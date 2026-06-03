mod files;
mod python;
mod reference;

pub use files::{ListFiles, ReadFile, WriteFile};
pub use python::{InstallPackages, RunPython, SetupVenv};
pub use reference::ConsultReference;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_OUTPUT: usize = 4000;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),
}

fn resolve(root: &Path, rel: &str) -> PathBuf {
    root.join(rel)
}

fn run_command(
    root: &Path,
    program: impl AsRef<OsStr>,
    args: &[String],
) -> Result<String, ToolError> {
    let output = Command::new(program).args(args).current_dir(root).output()?;
    let status = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(format!(
        "exit={status}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        truncate(&stdout),
        truncate(&stderr)
    ))
}

fn truncate(s: &str) -> String {
    if s.chars().count() <= MAX_OUTPUT {
        s.to_string()
    } else {
        s.chars().take(MAX_OUTPUT).collect::<String>() + "…"
    }
}
