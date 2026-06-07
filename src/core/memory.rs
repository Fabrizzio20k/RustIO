use crate::core::state::{ChatMessage, Role};
use rig::completion::Message;

pub struct Turn {
    pub summary: Option<String>,
    pub overflow: Vec<(Role, String)>,
    pub recent: Vec<(Role, String)>,
    pub prompt: String,
    pub new_upto: usize,
}

pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4 + 1
}

pub fn plan(
    messages: &[ChatMessage],
    summary: Option<&str>,
    budget: usize,
) -> (Vec<(Role, String)>, Vec<(Role, String)>) {
    let reserve = summary.map(estimate_tokens).unwrap_or(0);
    let available = budget.saturating_sub(reserve);

    let mut split = messages.len();
    let mut used = 0;
    for i in (0..messages.len()).rev() {
        let cost = estimate_tokens(&messages[i].content) + 4;
        if used + cost > available && split != messages.len() {
            break;
        }
        used += cost;
        split = i;
    }

    let overflow = messages[..split]
        .iter()
        .map(|m| (m.role, m.content.clone()))
        .collect();
    let recent = messages[split..]
        .iter()
        .map(|m| (m.role, m.content.clone()))
        .collect();
    (overflow, recent)
}

pub fn build_history(summary: Option<&str>, recent: &[(Role, String)]) -> Vec<Message> {
    let mut history = Vec::new();
    if let Some(text) = summary {
        if !text.is_empty() {
            history.push(Message::user(format!(
                "Resumen de la conversacion previa (contexto, no respondas a esto directamente):\n{text}"
            )));
        }
    }
    for (role, content) in recent {
        history.push(match role {
            Role::User => Message::user(content.clone()),
            Role::Assistant => Message::assistant(content.clone()),
        });
    }
    history
}

pub fn summarize_prompt(existing: Option<&str>, messages: &[(Role, String)]) -> String {
    let mut out = String::new();
    out.push_str("Eres un compresor de memoria de conversaciones. Resume de forma breve y en espanol, conservando datos concretos, decisiones y preferencias del usuario. Devuelve solo el resumen, sin preambulos.\n\n");
    if let Some(prev) = existing {
        if !prev.is_empty() {
            out.push_str("Resumen previo:\n");
            out.push_str(prev);
            out.push_str("\n\n");
        }
    }
    out.push_str("Mensajes a integrar:\n");
    for (role, content) in messages {
        let who = match role {
            Role::User => "Usuario",
            Role::Assistant => "Asistente",
        };
        out.push_str(who);
        out.push_str(": ");
        out.push_str(content);
        out.push('\n');
    }
    out.push_str("\nResumen actualizado:");
    out
}
