use anyhow::{Context, Result};
use rig::agent::Agent;
use rig::completion::{Prompt, TypedPrompt};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::agents::{Plan, Role, SubTask};
use crate::context::{ContextStore, Memory};
use crate::llm::{LlmFactory, Model};
use crate::prompts::Prompts;
use crate::tools::{
    ConsultReference, Dht11Reference, InstallPackages, ListFiles, Mq135Reference, ReadFile,
    RunPython, SetupVenv, WriteFile,
};

const CONTEXT_K: usize = 3;
const CONTEXT_CHARS: usize = 600;
const MAX_TURNS: usize = 12;

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
            .agent()
            .preamble(self.prompts.planner.as_str())
            .build();
        let plan: Plan = planner
            .prompt_typed::<Plan>(task)
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
                    eprintln!("subtarea {} no completada: {error:#}", st.id);
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
        let answer = agent
            .prompt(prompt)
            .max_turns(MAX_TURNS)
            .await
            .context("el agente no completó la subtarea")?;
        Ok(answer)
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
                .tool(Dht11Reference::new(self.prompts.dht11.clone()))
                .tool(Mq135Reference::new(self.prompts.mq135.clone()))
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
