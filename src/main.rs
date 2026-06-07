mod agents;
mod config;
mod context;
mod llm;
mod orchestrator;
mod prompts;
mod tools;

use anyhow::Result;

use config::Config;
use context::ContextStore;
use llm::LlmFactory;
use orchestrator::Orchestrator;
use prompts::Prompts;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut cfg = Config::from_env()?;

    if cfg.provider == config::Provider::Local {
        if let Ok(output) = std::process::Command::new("ollama").arg("list").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let models: Vec<String> = stdout
                    .lines()
                    .skip(1)
                    .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
                    .collect();
                
                if !models.is_empty() {
                    println!("\n[⚙️  OLLAMA] Modelos locales detectados:");
                    for (i, model) in models.iter().enumerate() {
                        println!("  {}) {}", i + 1, model);
                    }
                    use std::io::Write;
                    print!("\nSelecciona el número del modelo a usar [1-{}] (Enter para default: {}): ", models.len(), cfg.model);
                    std::io::stdout().flush()?;
                    
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    let input = input.trim();
                    
                    if let Ok(idx) = input.parse::<usize>() {
                        if idx > 0 && idx <= models.len() {
                            cfg.model = models[idx - 1].clone();
                        }
                    }
                }
            }
        }
    } else if cfg.provider == config::Provider::OpenAI {
        let models = vec![
            "gpt-4o-mini".to_string(),
            "gpt-4o".to_string(),
            "o1-mini".to_string(),
            "o1-preview".to_string(),
            "gpt-3.5-turbo".to_string(),
            "gpt-4-turbo".to_string(),
        ];
        
        if !models.is_empty() {
            println!("\n[☁️  OPENAI] Modelos disponibles:");
            for (i, model) in models.iter().enumerate() {
                println!("  {}) {}", i + 1, model);
            }
            use std::io::Write;
            print!("\nSelecciona el número del modelo a usar [1-{}] (Enter para default: {}): ", models.len(), cfg.model);
            std::io::stdout().flush()?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if let Ok(idx) = input.parse::<usize>() {
                if idx > 0 && idx <= models.len() {
                    cfg.model = models[idx - 1].clone();
                }
            }
        }
    }

    println!("\n=================================");
    println!(" Proveedor: {:?}", cfg.provider);
    println!(" Modelo   : {}", cfg.model);
    println!("=================================\n");

    std::fs::create_dir_all(&cfg.workspace_dir)?;

    let factory = LlmFactory::new(&cfg)?;
    let prompts = Prompts::load(&cfg.prompts_dir)?;
    let ctx = ContextStore::new("context.db")?;
    let mut orchestrator = Orchestrator::new(factory, ctx, prompts, cfg.workspace_dir.into());

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let task = args[1..].join(" ");
        let result = orchestrator.run(&task).await?;
        println!("\n=== ENTREGABLE FINAL ===\n{result}");
    } else {
        println!("\nIniciando modo chat interactivo. Escribe 'salir' o 'exit' para terminar.");
        loop {
            use std::io::{self, Write};
            print!("\nUsuario> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let task = input.trim();
            
            if task.is_empty() {
                continue;
            }
            if task.eq_ignore_ascii_case("salir") || task.eq_ignore_ascii_case("exit") {
                println!("Saliendo del chat...");
                break;
            }
            
            match orchestrator.run(task).await {
                Ok(result) => {
                    println!("\n=== ENTREGABLE FINAL ===\n{result}");
                }
                Err(e) => {
                    eprintln!("Error al procesar la tarea: {e}");
                }
            }
        }
    }

    Ok(())
}
