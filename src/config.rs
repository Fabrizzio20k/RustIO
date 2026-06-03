use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Groq,
    Local,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub provider: Provider,
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    pub prompts_dir: String,
    pub workspace_dir: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let provider = match std::env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "local".to_string())
            .to_lowercase()
            .as_str()
        {
            "groq" => Provider::Groq,
            "local" => Provider::Local,
            other => bail!("LLM_PROVIDER inválido: '{other}' (usa 'groq' o 'local')"),
        };

        let model = std::env::var("LLM_MODEL")
            .context("Falta LLM_MODEL en el entorno o en .env")?;

        let (api_key, base_url) = match provider {
            Provider::Groq => {
                let key = std::env::var("GROQ_API_KEY")
                    .context("Falta GROQ_API_KEY para LLM_PROVIDER=groq")?;
                (key, "https://api.groq.com/openai/v1".to_string())
            }
            Provider::Local => {
                let url = std::env::var("LOCAL_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:8080/v1".to_string());
                ("EMPTY".to_string(), url)
            }
        };

        let prompts_dir = std::env::var("PROMPTS_DIR").unwrap_or_else(|_| "prompts".to_string());
        let workspace_dir =
            std::env::var("WORKSPACE_DIR").unwrap_or_else(|_| "workspace".to_string());

        Ok(Self {
            provider,
            model,
            api_key,
            base_url,
            prompts_dir,
            workspace_dir,
        })
    }
}
