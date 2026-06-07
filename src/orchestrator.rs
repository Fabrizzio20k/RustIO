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
    ConsultReference, Dht11Reference, InstallPackages, ListFiles, MergeCode, Mq135Reference, ReadFile,
    RunPython, SetupVenv, WriteFile,
};

const CONTEXT_K: usize = 3;
const CONTEXT_CHARS: usize = 600;
const MAX_TURNS: usize = 12;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Chunk {
    pub file: String,
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Blueprint {
    pub chunks: Vec<Chunk>,
}

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
        let mut plan: Plan = Plan { subtasks: vec![] };
        let mut retries = 0;
        loop {
            match planner.prompt_typed::<Plan>(task).await {
                Ok(p) => {
                    plan = p;
                    break;
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if (err_str.contains("429") || err_str.contains("rate limit")) && retries < 5 {
                        println!("  [⏳ Planner] Rate limit (429). Esperando {}s...", 5 * (retries + 1));
                        tokio::time::sleep(std::time::Duration::from_secs(5 * (retries + 1))).await;
                        retries += 1;
                    } else {
                        return Err(e).context("el planner no pudo generar un plan válido");
                    }
                }
            }
        }

        println!("\n=== PLAN ({} subtareas) ===", plan.subtasks.len());
        for st in &plan.subtasks {
            println!("  [{}] {:?} — {}", st.id, st.role, st.title);
        }

        self.ctx.add(&task_id, "user", task)?;

        let mut results: Vec<String> = Vec::new();
        for st in &plan.subtasks {
            println!("\n--- [{}] {} ({:?}) ---", st.id, st.title, st.role);

            let memories = self.ctx.search(&st.description, CONTEXT_K)?;

            if st.role == Role::Coder {
                let blueprint_prompt = format!(
                    "Divide esta tarea en archivos para implementarse paso a paso. IMPORTANTE: Crea un ÚNICO chunk por cada archivo necesario. NO crees múltiples chunks para el mismo archivo.\n\
                     Tarea: {}\n\nResponde usando el siguiente formato XML estricto para cada archivo:\n\
                     <chunk>\n  <file>archivo.py</file>\n  <description>descripción detallada de todo el contenido del archivo</description>\n</chunk>",
                    st.description
                );

                let blueprint_agent = self.factory.agent()
                    .preamble("Eres el Arquitecto. Divide la tarea en partes.")
                    .temperature(0.1)
                    .build();

                let mut blueprint_raw = String::new();
                let mut retries = 0;
                loop {
                    match blueprint_agent.prompt(&blueprint_prompt).await {
                        Ok(ans) => {
                            blueprint_raw = ans;
                            break;
                        }
                        Err(e) => {
                            let err_str = e.to_string();
                            if (err_str.contains("429") || err_str.contains("rate limit")) && retries < 5 {
                                println!("  [⏳ Arquitecto] Rate limit (429). Esperando {}s...", 5 * (retries + 1));
                                tokio::time::sleep(std::time::Duration::from_secs(5 * (retries + 1))).await;
                                retries += 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
                
                let mut chunks = Vec::new();
                let mut search_idx = 0;
                while let Some(start) = blueprint_raw[search_idx..].find("<chunk>") {
                    let chunk_start = search_idx + start;
                    if let Some(end) = blueprint_raw[chunk_start..].find("</chunk>") {
                        let chunk_str = &blueprint_raw[chunk_start..chunk_start + end + 8];
                        
                        let file = if let (Some(fs), Some(fe)) = (chunk_str.find("<file>"), chunk_str.find("</file>")) {
                            chunk_str[fs + 6..fe].trim().to_string()
                        } else {
                            "main.py".to_string()
                        };
                        
                        let desc = if let (Some(ds), Some(de)) = (chunk_str.find("<description>"), chunk_str.find("</description>")) {
                            chunk_str[ds + 13..de].trim().to_string()
                        } else {
                            "Implementar lógica".to_string()
                        };
                        
                        chunks.push(Chunk { file, description: desc });
                        search_idx = chunk_start + end + 8;
                    } else {
                        break;
                    }
                }

                if chunks.is_empty() {
                    println!("  -> [Arquitecto] Error extrayendo XML. Respuesta cruda: {}", blueprint_raw);
                    chunks.push(Chunk {
                        file: "main.py".into(),
                        description: st.description.clone(),
                    });
                }

                println!("  -> [Arquitecto] Generó Blueprint con {} chunks", chunks.len());

                for (i, chunk) in chunks.iter().enumerate() {
                    println!("    -> Chunk {}/{}: {} ({})", i + 1, chunks.len(), chunk.description, chunk.file);

                    let chunk_prompt = format!(
                        "Tarea global: {}\n\n\
                         Implementa el archivo {}. Debes imprimir TODO el contenido del archivo en un ÚNICO bloque de código Markdown.\nInstrucciones del archivo: {}",
                        task,
                        chunk.file,
                        chunk.description
                    );

                    match self.run_subtask(st.role, &chunk_prompt).await {
                        Ok(answer) => {
                            self.process_answer(&answer, &task_id, st, Some(&chunk.file), &mut results)?;
                        }
                        Err(error) => {
                            eprintln!("subtarea chunk {} no completada: {error:#}", i+1);
                        }
                    }
                }
            } else {
                let prompt = build_prompt(task, st, &memories);
                match self.run_subtask(st.role, prompt.as_str()).await {
                    Ok(answer) => {
                        self.process_answer(&answer, &task_id, st, None, &mut results)?;
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
        let mut retries = 0;
        loop {
            match agent.prompt(prompt).max_turns(MAX_TURNS).await {
                Ok(ans) => return Ok(ans),
                Err(e) => {
                    let err_str = e.to_string();
                    if (err_str.contains("429") || err_str.contains("rate limit")) && retries < 5 {
                        println!("  [⏳ {}] Rate limit (429). Esperando {}s...", role.as_str(), 5 * (retries + 1));
                        tokio::time::sleep(std::time::Duration::from_secs(5 * (retries + 1))).await;
                        retries += 1;
                    } else {
                        return Err(e).context("el agente no completó la subtarea");
                    }
                }
            }
        }
    }

    fn process_answer(
        &mut self,
        answer: &str,
        task_id: &str,
        st: &SubTask,
        target_file: Option<&str>,
        results: &mut Vec<String>,
    ) -> Result<()> {
        if answer.contains("\"name\": \"write_file\"") || answer.contains("\"name\": \"merge_code\"") {
            let mut start_idx = 0;
            while let Some(pos) = answer[start_idx..].find("{\"name\":") {
                let json_start = start_idx + pos;
                let json_slice = &answer[json_start..];
                
                let mut stream = serde_json::Deserializer::from_str(json_slice).into_iter::<serde_json::Value>();
                if let Some(Ok(parsed)) = stream.next() {
                    if let Some(args) = parsed.get("arguments") {
                        if let (Some(path), Some(content)) = (
                            args.get("path").and_then(|v| v.as_str()),
                            args.get("content").or_else(|| args.get("chunk")).and_then(|v| v.as_str()),
                        ) {
                            let full_path = self.workspace.join(path);
                            if let Some(parent) = full_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            
                            use std::io::Write;
                            let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&full_path).unwrap();
                            match writeln!(file, "\n{}", content) {
                                Ok(_) => println!("\n[🔧 FALLBACK DE SEGURIDAD] ¡El fragmento fue rescatado y adjuntado en: {:?}!", full_path),
                                Err(e) => eprintln!("\n[🔧 FALLBACK ERROR] No se pudo guardar {:?}: {}", full_path, e),
                            }
                        }
                    }
                }
                start_idx = json_start + 1;
            }
        }

        let mut start_search = 0;
        let mut file_counter = 1;
        while let Some(start) = answer[start_search..].find("```") {
            let ext = target_file.map(|f| std::path::Path::new(f).extension().and_then(|e| e.to_str()).unwrap_or("")).unwrap_or("");
            let primary_lang = match ext {
                "py" => "```python",
                "sh" | "bash" => "```bash",
                "md" => "```markdown",
                "json" => "```json",
                "rs" => "```rust",
                "txt" => "```plaintext",
                _ => "```",
            };

            let mut code_start = start_search + start + 3; // Default to ` ``` `
            if answer[start_search + start..].starts_with(primary_lang) {
                code_start = start_search + start + primary_lang.len();
            } else if answer[start_search + start..].starts_with("```python") {
                code_start = start_search + start + 9;
            } else if answer[start_search + start..].starts_with("```bash") {
                code_start = start_search + start + 7;
            }

            if let Some(end_offset) = answer[code_start..].find("```") {
                let code = &answer[code_start..code_start + end_offset];
                let file_name = if let Some(t) = target_file { t.to_string() } else { format!("script_extraido_{}.py", file_counter) };
                let full_path = self.workspace.join(&file_name);
                if let Some(parent) = full_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&full_path).unwrap();
                if let Ok(_) = writeln!(file, "{}", code.trim()) {
                    println!("\n[🐍 EXTRACTOR PIPELINE] ¡Chunk guardado automáticamente en: {:?}!", full_path);
                }
                file_counter += 1;
                start_search = code_start + end_offset + 3;
            } else {
                break;
            }
        }

        println!("{answer}");
        self.ctx.add(task_id, st.role.as_str(), answer)?;
        results.push(format!("## [{}] {}\n{answer}", st.id, st.title));
        Ok(())
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
                .tool(ReadFile::new(ws.clone()))
                .tool(ListFiles::new(ws.clone()))
                .tool(RunPython::new(ws))
                .tool(ConsultReference::new(self.prompts.reference.clone()))
                .tool(Dht11Reference::new(self.prompts.dht11.clone()))
                .tool(Mq135Reference::new(self.prompts.mq135.clone()))
                .build(),
            Role::Reviewer => builder
                .tool(ReadFile::new(ws.clone()))
                .tool(ListFiles::new(ws.clone()))
                .build(),
            Role::Fixer => builder
                .tool(ReadFile::new(ws.clone()))
                .tool(WriteFile::new(ws.clone()))
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
