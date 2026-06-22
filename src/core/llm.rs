use crate::core::config::{Config, Provider};
use crate::core::memory::{self, Turn};
use crate::core::tools::{RunPython, RunShell};
use anyhow::{Result, anyhow};
use futures::StreamExt;
use futures::future::BoxFuture;
use rig::agent::{Agent, MultiTurnStreamItem};
use rig::client::BearerAuth;
use rig::completion::message::{ToolResult, ToolResultContent};
use rig::completion::{CompletionModel, GetTokenUsage, Message, Prompt};
use rig::prelude::{CompletionClient, ProviderClient};
use rig::providers::{groq, openai};
use rig::streaming::{StreamedAssistantContent, StreamedUserContent, StreamingChat};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub enum Token {
    Delta(String),
    ToolCall { name: String, preview: String },
    ToolResult { summary: String },
    Summary { text: String, upto: usize },
    Done,
    Error(String),
}

fn tool_preview(args: &serde_json::Value) -> String {
    let raw = args
        .get("command")
        .or_else(|| args.get("code"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let first = raw.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    first.chars().take(60).collect()
}

fn result_summary(result: &ToolResult) -> String {
    let text = result
        .content
        .iter()
        .find_map(|c| match c {
            ToolResultContent::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .unwrap_or_default();

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
        let code = v.get("exit_code").and_then(|x| x.as_i64());
        let stdout = v.get("stdout").and_then(|x| x.as_str()).unwrap_or("");
        let stderr = v.get("stderr").and_then(|x| x.as_str()).unwrap_or("");
        let src = match code {
            Some(0) => stdout,
            _ if !stderr.is_empty() => stderr,
            _ => stdout,
        };
        let snippet: String = src
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .chars()
            .take(60)
            .collect();
        return match code {
            Some(c) if snippet.is_empty() => format!("exit {c}"),
            Some(c) => format!("exit {c} · {snippet}"),
            None => snippet,
        };
    }

    text.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .chars()
        .take(60)
        .collect()
}

trait Backend: Send + Sync {
    fn complete(&self, prompt: String) -> BoxFuture<'static, Result<String, String>>;
    fn stream(
        &self,
        history: Vec<Message>,
        prompt: String,
        tx: UnboundedSender<Token>,
    ) -> BoxFuture<'static, ()>;
}

struct AgentBackend<M: CompletionModel>(Arc<Agent<M>>);

impl<M> Backend for AgentBackend<M>
where
    M: CompletionModel + 'static,
    M::StreamingResponse: GetTokenUsage,
{
    fn complete(&self, prompt: String) -> BoxFuture<'static, Result<String, String>> {
        let agent = self.0.clone();
        Box::pin(async move { agent.prompt(prompt).await.map_err(|e| e.to_string()) })
    }

    fn stream(
        &self,
        history: Vec<Message>,
        prompt: String,
        tx: UnboundedSender<Token>,
    ) -> BoxFuture<'static, ()> {
        let agent = self.0.clone();
        Box::pin(async move {
            let mut stream = agent.stream_chat(prompt, history).await;
            while let Some(item) = stream.next().await {
                match item {
                    Ok(MultiTurnStreamItem::StreamAssistantItem(
                        StreamedAssistantContent::Text(text),
                    )) => {
                        let _ = tx.send(Token::Delta(text.text));
                    }
                    Ok(MultiTurnStreamItem::StreamAssistantItem(
                        StreamedAssistantContent::ToolCall { tool_call, .. },
                    )) => {
                        let _ = tx.send(Token::ToolCall {
                            name: tool_call.function.name.clone(),
                            preview: tool_preview(&tool_call.function.arguments),
                        });
                    }
                    Ok(MultiTurnStreamItem::StreamUserItem(
                        StreamedUserContent::ToolResult { tool_result, .. },
                    )) => {
                        let _ = tx.send(Token::ToolResult {
                            summary: result_summary(&tool_result),
                        });
                    }
                    Ok(_) => {}
                    Err(err) => {
                        let _ = tx.send(Token::Error(err.to_string()));
                        return;
                    }
                }
            }
            let _ = tx.send(Token::Done);
        })
    }
}

pub struct Llm {
    inner: Arc<dyn Backend>,
}

impl Llm {
    pub fn new(config: &Config) -> Result<Self> {
        let ws = &config.workspace_dir;
        let _ = std::fs::create_dir_all(ws);
        let inner: Arc<dyn Backend> = match config.provider {
            Provider::Local => {
                let key = config.api_key.clone().unwrap_or_else(|| "local".into());
                let client = openai::CompletionsClient::builder()
                    .api_key(BearerAuth::from(key))
                    .base_url(&config.base_url)
                    .build()
                    .map_err(|e| anyhow!("no se pudo crear el cliente local: {e:?}"))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
                    .default_max_turns(8)
                    .tool(RunPython::new(ws.clone()))
                    .tool(RunShell::new(ws.clone()))
                    .build();
                Arc::new(AgentBackend(Arc::new(agent)))
            }
            Provider::Groq => {
                let client = groq::Client::from_env()
                    .map_err(|e| anyhow!("no se pudo crear el cliente groq: {e:?}"))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
                    .default_max_turns(8)
                    .tool(RunPython::new(ws.clone()))
                    .tool(RunShell::new(ws.clone()))
                    .build();
                Arc::new(AgentBackend(Arc::new(agent)))
            }
        };

        Ok(Self { inner })
    }

    pub fn run_turn(&self, turn: Turn, tx: UnboundedSender<Token>) -> BoxFuture<'static, ()> {
        let inner = self.inner.clone();
        Box::pin(async move {
            let overflow = turn.overflow;
            let mut recent = turn.recent;
            let mut summary = turn.summary;

            if !overflow.is_empty() {
                let prompt = memory::summarize_prompt(summary.as_deref(), &overflow);
                match inner.complete(prompt).await {
                    Ok(text) => {
                        let text = text.trim().to_string();
                        summary = Some(text.clone());
                        let _ = tx.send(Token::Summary {
                            text,
                            upto: turn.new_upto,
                        });
                    }
                    Err(_) => {
                        let mut merged = overflow;
                        merged.extend(recent);
                        recent = merged;
                    }
                }
            }

            let history = memory::build_history(summary.as_deref(), &recent);
            inner.stream(history, turn.prompt, tx).await;
        })
    }
}
