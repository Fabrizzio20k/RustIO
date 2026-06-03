use anyhow::{Context, Result};
use rig::agent::Agent;
use rig::completion::{Completion, Prompt};
use rig::message::{AssistantContent, Message, ToolResultContent, UserContent};
use rig::OneOrMany;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use crate::agents::{Plan, Role, SubTask};
use crate::context::{ContextStore, Memory};
use crate::llm::{LlmFactory, Model};
use crate::prompts::Prompts;
use crate::tools::{
    ConsultReference, InstallPackages, ListFiles, ReadFile, RunPython, SetupVenv, WriteFile,
};

const CONTEXT_K: usize = 3;
const CONTEXT_CHARS: usize = 600;
const MAX_ATTEMPTS: usize = 4;
const MAX_TURNS: usize = 12;
const RATE_LIMIT_BACKOFF: u64 = 12;

pub struct Orchestrator {
    factory: LlmFactory,
    ctx: ContextStore,
    prompts: Prompts,
    workspace: PathBuf,
}

impl Orchestrator {
    pub fn new(
        factory: LlmFactory,
        ctx: ContextStore,
        prompts: Prompts,
        workspace: PathBuf,
    ) -> Self {
        Self {
            factory,
            ctx,
            prompts,
            workspace,
        }
    }

    pub async fn run(&mut self, task: &str) -> Result<String> {
        let task_id = short_id(task);

        let planner = self
            .factory
            .extractor::<Plan>()
            .preamble(self.prompts.planner.as_str())
            .build();
        let plan = planner
            .extract(task)
            .await
            .context("el planner no pudo generar un plan válido")?;

        println!("\n=== PLAN ({} subtareas) ===", plan.subtasks.len());
        for st in &plan.subtasks {
            println!("  [{}] {:?} — {}", st.id, st.role, st.title);
        }

        self.ctx.add(&task_id, "user", task)?;

        let mut results: Vec<String> = Vec::new();
        for st in &plan.subtasks {
            println!("\n--- [{}] {} ({:?}) ---", st.id, st.title, st.role);

            let memories = self.ctx.search(&st.description, CONTEXT_K)?;
            let prompt = build_prompt(task, st, &memories);

            match self.run_subtask(st.role, prompt.as_str()).await {
                Ok(answer) => {
                    println!("{answer}");
                    self.ctx.add(&task_id, st.role.as_str(), &answer)?;
                    results.push(format!("## [{}] {}\n{answer}", st.id, st.title));
                }
                Err(error) => {
                    eprintln!("subtarea {} no completada: {error}", st.id);
                    let note = format!("La subtarea no se completó: {error}");
                    self.ctx.add(&task_id, st.role.as_str(), &note)?;
                    results.push(format!(
                        "## [{}] {} (no completada)\n{error}",
                        st.id, st.title
                    ));
                }
            }
        }

        let synth = self
            .factory
            .agent()
            .preamble(self.prompts.synthesizer.as_str())
            .temperature(0.2)
            .build();
        let joined = results.join("\n\n");
        let summary = synth
            .prompt(format!(
                "Tarea original:\n{task}\n\nResultados de las subtareas:\n{joined}\n\n\
                 Sintetiza el entregable final."
            ))
            .await
            .context("falló la síntesis final")?;

        Ok(summary)
    }

    async fn run_subtask(&self, role: Role, prompt: &str) -> Result<String> {
        let agent = self.build_worker(role);
        let mut history: Vec<Message> = Vec::new();
        let mut pending = Message::user(prompt);

        for _ in 0..MAX_TURNS {
            let choice = complete(&agent, &pending, &history).await?;

            let mut tool_calls = Vec::new();
            let mut texts = Vec::new();
            for content in choice.iter() {
                match content {
                    AssistantContent::Text(text) => texts.push(text.text.clone()),
                    AssistantContent::ToolCall(call) => tool_calls.push(call.clone()),
                }
            }

            if tool_calls.is_empty() {
                return Ok(texts.join("\n"));
            }

            history.push(pending);
            history.push(Message::Assistant {
                content: choice.clone(),
            });

            let mut results = Vec::new();
            for call in &tool_calls {
                let output = agent
                    .tools
                    .call(&call.function.name, call.function.arguments.to_string())
                    .await
                    .unwrap_or_else(|error| format!("ERROR: {error}"));
                println!("    · {} -> {}", call.function.name, first_line(&output));
                results.push(UserContent::tool_result(
                    call.id.clone(),
                    OneOrMany::one(ToolResultContent::text(output)),
                ));
            }

            pending = Message::User {
                content: OneOrMany::many(results).expect("hay al menos un resultado de tool"),
            };
        }

        anyhow::bail!("el agente no terminó en {MAX_TURNS} turnos")
    }

    fn build_worker(&self, role: Role) -> Agent<Model> {
        let ws = self.workspace.clone();
        let builder = self
            .factory
            .agent()
            .preamble(self.prompts.worker(role))
            .temperature(0.2);

        match role {
            Role::Coder => builder
                .tool(SetupVenv::new(ws.clone()))
                .tool(InstallPackages::new(ws.clone()))
                .tool(WriteFile::new(ws.clone()))
                .tool(ReadFile::new(ws.clone()))
                .tool(ListFiles::new(ws.clone()))
                .tool(RunPython::new(ws))
                .tool(ConsultReference::new(self.prompts.reference.clone()))
                .build(),
            Role::Reviewer => builder
                .tool(ReadFile::new(ws.clone()))
                .tool(ListFiles::new(ws))
                .build(),
        }
    }
}

fn build_prompt(task: &str, st: &SubTask, memories: &[Memory]) -> String {
    let context_block = if memories.is_empty() {
        "(sin contexto previo relevante)".to_string()
    } else {
        memories
            .iter()
            .map(|m| format!("- ({}) {}", m.role, truncate(&m.content, CONTEXT_CHARS)))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "Tarea global:\n{task}\n\n\
         Contexto relevante de pasos previos:\n{context_block}\n\n\
         Subtarea actual — {title}:\n{desc}\n\n\
         Completa únicamente esta subtarea.",
        title = st.title,
        desc = st.description,
    )
}

async fn complete(
    agent: &Agent<Model>,
    pending: &Message,
    history: &[Message],
) -> Result<OneOrMany<AssistantContent>> {
    let mut last_error = String::new();
    for attempt in 1..=MAX_ATTEMPTS {
        let sent = match agent.completion(pending.clone(), history.to_vec()).await {
            Ok(builder) => builder.send().await,
            Err(error) => Err(error),
        };
        match sent {
            Ok(response) => return Ok(response.choice),
            Err(error) => {
                last_error = error.to_string();
                eprintln!("  reintento {attempt}/{MAX_ATTEMPTS} (modelo): {last_error}");
                if attempt < MAX_ATTEMPTS {
                    let backoff = if is_rate_limit(&last_error) {
                        RATE_LIMIT_BACKOFF
                    } else {
                        1
                    };
                    sleep(Duration::from_secs(backoff)).await;
                }
            }
        }
    }
    anyhow::bail!("fallo del modelo tras {MAX_ATTEMPTS} intentos: {last_error}")
}

fn first_line(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    truncate(line, 120)
}

fn is_rate_limit(error: &str) -> bool {
    error.contains("rate_limit") || error.contains("Rate limit")
}

fn short_id(task: &str) -> String {
    let mut hasher = DefaultHasher::new();
    task.hash(&mut hasher);
    format!("task-{:x}", hasher.finish())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect::<String>() + "…"
    }
}
