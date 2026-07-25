use anyhow::{Result, bail};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
pub struct ModelPreset {
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

pub fn load_presets() -> HashMap<String, ModelPreset> {
    let content = std::fs::read_to_string("models.json").unwrap_or_else(|_| "{}".into());
    serde_json::from_str(&content).unwrap_or_default()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Local,
    Groq,
    OpenAI,
    Anthropic,
    DeepSeek,
}

impl Provider {
    pub fn label(&self) -> &'static str {
        match self {
            Provider::Local => "local",
            Provider::Groq => "groq",
            Provider::OpenAI => "openai",
            Provider::Anthropic => "anthropic",
            Provider::DeepSeek => "deepseek",
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
    pub is_configured: bool,
}

impl Config {
    pub fn load(store: Option<&crate::core::store::Store>) -> Result<Self> {
        let provider_str = if let Some(s) = store {
            s.get_meta("llm_provider")?
                .unwrap_or_else(|| "local".into())
        } else {
            "local".into()
        };

        let provider = match provider_str.to_lowercase().as_str() {
            "local" => Provider::Local,
            "groq" => Provider::Groq,
            "openai" => Provider::OpenAI,
            "anthropic" => Provider::Anthropic,
            "deepseek" => Provider::DeepSeek,
            other => bail!("LLM_PROVIDER no soportado: {other}"),
        };

        let model = if let Some(s) = store {
            s.get_meta("llm_model")?.unwrap_or_else(|| "llama3".into())
        } else {
            "llama3".into()
        };

        let base_url = if let Some(s) = store {
            s.get_meta("llm_base_url")?
                .unwrap_or_else(|| "http://localhost:8080/v1".into())
        } else {
            "http://localhost:8080/v1".into()
        };

        let api_key = if let Some(s) = store {
            s.get_meta("llm_api_key")?
        } else {
            None
        };

        let history_budget_tokens = if let Some(s) = store {
            s.get_meta("history_budget_tokens")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000)
        } else {
            3000
        };

        let system_prompt = format!(
            "Eres RustIO, un asistente para la gestion de dispositivos IoT en una Raspberry Pi. \
Te ejecutas en un sistema operativo '{}'; usa siempre comandos nativos de ese sistema \
(en windows: netsh, powershell; en linux: nmcli, iw, ip). \
Tienes herramientas para ejecutar codigo (run_python) y comandos de shell (run_shell): \
SIEMPRE intenta resolver con ellas (leer hardware, specs del sistema, redes, sensores) \
antes de decir que no puedes. \
Para codigo Python: el entorno virtual se gestiona automaticamente con uv; \
si necesitas un paquete externo, incluye su nombre en el campo 'pip' de run_python \
(ej: pip: ['numpy', 'psutil']); si obtienes ModuleNotFoundError, reintenta con el paquete en 'pip'; \
NUNCA uses pip ni uv en run_shell para instalar paquetes Python. \
Si una ejecucion falla, corrige segun el error y reintenta. \
Responde de forma clara, concisa y en espanol y sin emojis.",
            std::env::consts::OS
        );

        let workspace_dir = if let Some(s) = store {
            s.get_meta("workspace_dir")?
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("workspace"))
        } else {
            PathBuf::from("workspace")
        };
        let is_configured = if let Some(s) = store {
            s.get_meta("llm_model")?.is_some()
        } else {
            false
        };

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
            is_configured,
        })
    }
}
