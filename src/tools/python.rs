use std::path::PathBuf;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use serde_json::{json, Value};

use super::{run_command, ToolError};

pub struct SetupVenv {
    root: PathBuf,
}

impl SetupVenv {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl Tool for SetupVenv {
    const NAME: &'static str = "setup_venv";
    type Error = ToolError;
    type Args = Value;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Crea el entorno virtual .venv en el workspace con `uv venv`.".to_string(),
            parameters: json!({ "type": "object", "properties": {} }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        run_command(&self.root, "uv", &["venv".to_string()])
    }
}

pub struct InstallPackages {
    root: PathBuf,
}

impl InstallPackages {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Deserialize)]
pub struct InstallArgs {
    pub packages: Vec<String>,
}

impl Tool for InstallPackages {
    const NAME: &'static str = "install_packages";
    type Error = ToolError;
    type Args = InstallArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Instala paquetes de Python en el .venv con `uv pip install`."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "packages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Nombres de los paquetes a instalar"
                    }
                },
                "required": ["packages"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut cmd = vec!["pip".to_string(), "install".to_string()];
        cmd.extend(args.packages);
        run_command(&self.root, "uv", &cmd)
    }
}

pub struct RunPython {
    root: PathBuf,
}

impl RunPython {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Deserialize)]
pub struct RunArgs {
    pub file: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl Tool for RunPython {
    const NAME: &'static str = "run_python";
    type Error = ToolError;
    type Args = RunArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Ejecuta un archivo Python con el intérprete del .venv del workspace."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file": { "type": "string", "description": "Ruta del archivo .py relativa al workspace" },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Argumentos opcionales para el script"
                    }
                },
                "required": ["file"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let python = self.root.join(".venv/bin/python");
        let mut cmd = vec![args.file];
        cmd.extend(args.args);
        run_command(&self.root, python, &cmd)
    }
}
