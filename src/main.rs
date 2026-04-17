use rig::completion::{CompletionModel, Message, ToolDefinition};
use rig::providers::openai;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = openai::Client::from_url("EMPTY", "http://localhost:8080/v1");
    let model = client.completion_model("qwen2.5-3b");

    // --- Turno 1: el modelo decide llamar el tool ---
    let response = model
        .completion(rig::completion::CompletionRequest {
            preamble: Some("Eres un asistente de red.".to_string()),
            chat_history: vec![],
            prompt: Message::user("¿Qué dispositivos de red tengo?"),
            documents: vec![],
            tools: vec![ToolDefinition {
                name: "listar_dispositivos".to_string(),
                description: "Lista los dispositivos de red conectados".to_string(),
                parameters: json!({"type": "object", "properties": {}}),
            }],
            temperature: None,
            additional_params: Some(json!({"tool_choice": "required"})),
            max_tokens: None,
        })
        .await?;

    // --- Detectar si hubo un tool call ---
    let tool_call = response.choice.iter().find_map(|c| {
        if let rig::completion::AssistantContent::ToolCall(tc) = c {
            Some(tc.clone())
        } else {
            None
        }
    });

    if let Some(tc) = tool_call {
        println!("Tool llamado: {}", tc.function.name);

        // --- Ejecutar la función Rust ---
        let resultado = json!([
            { "nombre": "eth0",  "ip": "192.168.1.1", "tipo": "ethernet" },
            { "nombre": "wlan0", "ip": "192.168.1.5", "tipo": "wifi" },
            { "nombre": "lo",    "ip": "127.0.0.1",   "tipo": "loopback" }
        ]);

        // --- Turno 2: devolver el resultado al modelo ---
        // --- Turno 2: devolver el resultado al modelo ---
        let respuesta_final = model
            .completion(rig::completion::CompletionRequest {
                preamble: Some("Eres un asistente de red.".to_string()),
                chat_history: vec![
                    Message::user("¿Qué dispositivos de red tengo?"),
                    // Simular el resultado del tool como contexto para el modelo
                    Message::assistant(format!(
                        "Resultado de listar_dispositivos: {}",
                        resultado.to_string()
                    )),
                ],
                prompt: Message::user(
                    "Con base en esos datos, resume los dispositivos encontrados.",
                ),
                documents: vec![],
                tools: vec![],
                temperature: None,
                additional_params: None,
                max_tokens: None,
            })
            .await?;
        // Extraer texto de la respuesta
        if let Some(rig::completion::AssistantContent::Text(t)) =
            respuesta_final.choice.iter().next()
        {
            println!("Agente: {}", t.text);
        }
    }

    Ok(())
}
