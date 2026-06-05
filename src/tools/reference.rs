use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde_json::{json, Value};

use super::ToolError;

/// Genera una tool de referencia que devuelve una guía fija (cableado, librería
/// y driver de ejemplo) para un dispositivo concreto. Añadir un sensor nuevo es
/// una sola línea más.
macro_rules! reference_tool {
    ($ty:ident, $name:literal, $desc:literal) => {
        pub struct $ty {
            content: String,
        }

        impl $ty {
            pub fn new(content: String) -> Self {
                Self { content }
            }
        }

        impl Tool for $ty {
            const NAME: &'static str = $name;
            type Error = ToolError;
            type Args = Value;
            type Output = String;

            async fn definition(&self, _prompt: String) -> ToolDefinition {
                ToolDefinition {
                    name: Self::NAME.to_string(),
                    description: $desc.to_string(),
                    parameters: json!({ "type": "object", "properties": {} }),
                }
            }

            async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
                Ok(self.content.clone())
            }
        }
    };
}

reference_tool!(
    ConsultReference,
    "consult_reference",
    "Devuelve una guía general con los comandos de uv y el uso de GPIO en la \
     Raspberry Pi 5. Consúltala cuando la tarea involucre el entorno de Python o GPIO."
);

reference_tool!(
    Dht11Reference,
    "dht11_reference",
    "Devuelve la guía del sensor DHT11 (temperatura y humedad): cableado, \
     dependencia y un driver de ejemplo listo para usar. Consúltala cuando la \
     tarea involucre un DHT11."
);

reference_tool!(
    Mq135Reference,
    "mq135_reference",
    "Devuelve la guía del sensor MQ135 (calidad del aire / gases): cableado con \
     el ADC MCP3008, dependencias y un driver de ejemplo. Consúltala cuando la \
     tarea involucre un MQ135."
);
