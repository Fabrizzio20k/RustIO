use anyhow::{Result, anyhow, bail};
use std::env;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Local,
    Groq,
    OpenAI,
}

impl Provider {
    pub fn label(&self) -> &'static str {
        match self {
            Provider::Local => "local",
            Provider::Groq => "groq",
            Provider::OpenAI => "openai",
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
    pub workspace_dir: PathBuf,
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
            "openai" => Provider::OpenAI,
            other => bail!("LLM_PROVIDER no soportado: {other}"),
        };

        let model = env::var("LLM_MODEL").map_err(|_| anyhow!("LLM_MODEL es requerido"))?;
        let base_url =
            env::var("LOCAL_BASE_URL").unwrap_or_else(|_| "http://localhost:8080/v1".into());

        // Selecciona la api_key según el provider activo
        let api_key = match provider {
            Provider::Groq => env::var("GROQ_API_KEY").ok(),
            Provider::OpenAI => env::var("OPENAI_API_KEY").ok(),
            Provider::Local => env::var("OPENAI_API_KEY")
                .ok()
                .or_else(|| env::var("GROQ_API_KEY").ok()),
        };

        let history_budget_tokens = env::var("HISTORY_BUDGET_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);

        let system_prompt = format!(
            "Eres RustIO, un asistente para la gestion de dispositivos IoT en una Raspberry Pi. Te ejecutas en un sistema operativo '{}'; usa siempre comandos nativos de ese sistema (en windows: netsh, powershell; en linux: nmcli, iw, ip). Tienes herramientas para ejecutar codigo (run_python) y comandos de shell (run_shell): SIEMPRE intenta resolver con ellas (leer hardware, specs del sistema, redes, sensores) antes de decir que no puedes. Si una ejecucion falla, corrige segun el error y reintenta. Responde de forma clara, concisa y en espanol y sin emojis.",
            std::env::consts::OS
        );

        let workspace_dir = env::var("WORKSPACE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("workspace"));
        let workspace_dir = if workspace_dir.is_absolute() {
            workspace_dir
        } else {
            env::current_dir().unwrap_or_default().join(workspace_dir)
        };

        Ok(Self {
            provider,
            model,
            base_url,
            api_key,
            system_prompt,
            history_budget_tokens,
            workspace_dir,
        })
    }
}
