use anyhow::{Result, anyhow, bail};
use std::env;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Local,
    Groq,
}

impl Provider {
    pub fn label(&self) -> &'static str {
        match self {
            Provider::Local => "local",
            Provider::Groq => "groq",
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub provider: Provider,
    pub model: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub system_prompt: String,
    pub history_budget_tokens: usize,
}

impl Config {
    pub fn load() -> Result<Self> {
        let provider = match env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "local".into())
            .to_lowercase()
            .as_str()
        {
            "local" => Provider::Local,
            "groq" => Provider::Groq,
            other => bail!("LLM_PROVIDER no soportado: {other}"),
        };

        let model = env::var("LLM_MODEL").map_err(|_| anyhow!("LLM_MODEL es requerido"))?;
        let base_url =
            env::var("LOCAL_BASE_URL").unwrap_or_else(|_| "http://localhost:8080/v1".into());
        let api_key = env::var("GROQ_API_KEY").ok();

        let history_budget_tokens = env::var("HISTORY_BUDGET_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);

        let system_prompt = "Eres RustIO, un asistente para la gestion de dispositivos IoT en una Raspberry Pi. Responde de forma clara, concisa y en espanol y sin emojis.".into();

        Ok(Self {
            provider,
            model,
            base_url,
            api_key,
            system_prompt,
            history_budget_tokens,
        })
    }
}
