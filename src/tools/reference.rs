use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde_json::{json, Value};

use super::ToolError;

pub struct ConsultReference {
    content: String,
}

impl ConsultReference {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl Tool for ConsultReference {
    const NAME: &'static str = "consult_reference";
    type Error = ToolError;
    type Args = Value;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Devuelve una guía con los comandos de uv, el uso de GPIO en la Raspberry \
                          Pi 5 y un ejemplo para leer un sensor DHT11. Consúltala solo cuando la \
                          tarea involucre hardware, sensores o GPIO."
                .to_string(),
            parameters: json!({ "type": "object", "properties": {} }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(self.content.clone())
    }
}
