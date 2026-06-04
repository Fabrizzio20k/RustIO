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
    let cfg = Config::from_env()?;
    println!("Proveedor: {:?} | Modelo: {}", cfg.provider, cfg.model);

    std::fs::create_dir_all(&cfg.workspace_dir)?;

    let factory = LlmFactory::new(&cfg)?;
    let prompts = Prompts::load(&cfg.prompts_dir)?;
    let ctx = ContextStore::new("context.db")?;
    let mut orchestrator = Orchestrator::new(factory, ctx, prompts, cfg.workspace_dir.into());

    let task = std::env::args().nth(1).unwrap_or_else(|| {
        "Crea un script python que use requests para consumir una api publica de perros, la de dogs api y que imprima el resultado en consola"
            .to_string()
    });

    let result = orchestrator.run(&task).await?;
    println!("\n=== ENTREGABLE FINAL ===\n{result}");
    Ok(())
}
