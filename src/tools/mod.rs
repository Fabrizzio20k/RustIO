mod files;
mod python;
mod reference;

pub use files::{ListFiles, ReadFile, WriteFile};
pub use python::{InstallPackages, RunPython, SetupVenv};
pub use reference::{ConsultReference, Dht11Reference, Mq135Reference};

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

const MAX_OUTPUT: usize = 4000;

/// Deserializa un `Vec<String>` tolerando las manías de los modelos pequeños:
/// acepta un array real (`["a","b"]`), un array JSON "stringificado"
/// (`"[\"a\",\"b\"]"`) o un único string (`"a"`).
pub fn de_string_or_seq<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(value_to_vec(Value::deserialize(deserializer)?))
}

fn value_to_vec(value: Value) -> Vec<String> {
    match value {
        Value::Array(items) => items.into_iter().map(value_to_string).collect(),
        Value::String(s) => match serde_json::from_str::<Value>(&s) {
            Ok(Value::Array(items)) => items.into_iter().map(value_to_string).collect(),
            _ => vec![s],
        },
        Value::Null => Vec::new(),
        other => vec![value_to_string(other)],
    }
}

fn value_to_string(v: Value) -> String {
    match v {
        Value::String(s) => s,
        other => other.to_string(),
    }
}

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
