use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ToolFail(String);

#[derive(Serialize)]
pub struct RunOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

// ponytail: tope de salida para no inundar el contexto del modelo
fn cap(s: &str) -> String {
    const MAX: usize = 4000;
    let cut: String = s.chars().take(MAX).collect();
    if cut.len() < s.len() {
        format!("{cut}\n...[salida recortada]")
    } else {
        cut
    }
}

fn captured(out: std::process::Output) -> RunOutput {
    RunOutput {
        exit_code: out.status.code().unwrap_or(-1),
        stdout: cap(&String::from_utf8_lossy(&out.stdout)),
        stderr: cap(&String::from_utf8_lossy(&out.stderr)),
    }
}

pub struct RunPython {
    workspace: PathBuf,
}

#[derive(Deserialize)]
pub struct RunPythonArgs {
    pub code: String,
    #[serde(default)]
    pub pip: Vec<String>,
}

impl RunPython {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }

    fn venv_python(&self) -> PathBuf {
        if cfg!(windows) {
            self.workspace.join(".venv/Scripts/python.exe")
        } else {
            self.workspace.join(".venv/bin/python")
        }
    }

    // El venv ya creado es su propia memoria: si existe, no se recrea.
    async fn ensure_venv(&self) -> Result<(), ToolFail> {
        if self.venv_python().exists() {
            return Ok(());
        }
        tokio::fs::create_dir_all(&self.workspace)
            .await
            .map_err(|e| ToolFail(format!("no se pudo crear el workspace: {e}")))?;
        let out = Command::new("uv")
            .args(["venv", "--python", "3.13", ".venv"])
            .current_dir(&self.workspace)
            .output()
            .await
            .map_err(|e| ToolFail(format!("no se pudo invocar uv (¿está instalado?): {e}")))?;
        if !out.status.success() {
            return Err(ToolFail(format!(
                "uv venv falló: {}",
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        Ok(())
    }
}

impl Tool for RunPython {
    const NAME: &'static str = "run_python";
    type Error = ToolFail;
    type Args = RunPythonArgs;
    type Output = RunOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Ejecuta código Python en un entorno virtual gestionado por uv (Python 3.13) y devuelve exit_code, stdout y stderr. El código DEBE imprimir los resultados con print(). Usa 'pip' para instalar dependencias antes de ejecutar. Si exit_code no es 0, corrige el código según stderr y vuelve a llamar.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Código Python a ejecutar. Imprime los resultados con print()."
                    },
                    "pip": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Paquetes pip a instalar en el venv antes de ejecutar (opcional)."
                    }
                },
                "required": ["code"]
            }),
        }
    }

    async fn call(&self, args: RunPythonArgs) -> Result<RunOutput, ToolFail> {
        self.ensure_venv().await?;
        let python = self.venv_python();

        if !args.pip.is_empty() {
            let out = Command::new("uv")
                .args(["pip", "install", "--python"])
                .arg(&python)
                .args(&args.pip)
                .current_dir(&self.workspace)
                .output()
                .await
                .map_err(|e| ToolFail(format!("uv pip falló al invocarse: {e}")))?;
            if !out.status.success() {
                return Ok(RunOutput {
                    exit_code: out.status.code().unwrap_or(-1),
                    stdout: String::new(),
                    stderr: cap(&format!(
                        "uv pip install falló:\n{}",
                        String::from_utf8_lossy(&out.stderr)
                    )),
                });
            }
        }

        let file = self.workspace.join(".rustio_run.py");
        tokio::fs::write(&file, &args.code)
            .await
            .map_err(|e| ToolFail(format!("no se pudo escribir el script: {e}")))?;

        let out = Command::new(&python)
            .arg(&file)
            .current_dir(&self.workspace)
            .output()
            .await
            .map_err(|e| ToolFail(format!("no se pudo ejecutar python: {e}")))?;
        Ok(captured(out))
    }
}

pub struct RunShell {
    workspace: PathBuf,
}

#[derive(Deserialize)]
pub struct RunShellArgs {
    pub command: String,
}

impl RunShell {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

impl Tool for RunShell {
    const NAME: &'static str = "run_shell";
    type Error = ToolFail;
    type Args = RunShellArgs;
    type Output = RunOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Ejecuta un comando de shell en la máquina y devuelve exit_code, stdout y stderr. Úsalo para instalar paquetes del sistema, listar archivos o tareas de terminal. Si falla, corrige según stderr y reintenta.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Comando de shell a ejecutar."
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: RunShellArgs) -> Result<RunOutput, ToolFail> {
        let (shell, flag) = if cfg!(windows) {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };
        let out = Command::new(shell)
            .arg(flag)
            .arg(&args.command)
            .current_dir(&self.workspace)
            .output()
            .await
            .map_err(|e| ToolFail(format!("no se pudo ejecutar el comando: {e}")))?;
        Ok(captured(out))
    }
}

#[cfg(test)]
mod tests {
    use super::cap;

    #[test]
    fn cap_handles_multibyte_boundaries() {
        let short = "café ñ 23.4°C";
        assert_eq!(cap(short), short);

        let long: String = "ñ".repeat(5000);
        let out = cap(&long);
        assert!(out.ends_with("[salida recortada]"));
        assert!(out.is_char_boundary(out.len()));
    }
}
