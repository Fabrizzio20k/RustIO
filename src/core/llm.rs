use crate::core::config::{Config, Provider};
use crate::core::memory::{self, Turn};
use anyhow::{Result, anyhow};
use futures::StreamExt;
use futures::future::BoxFuture;
use rig::agent::{Agent, MultiTurnStreamItem};
use rig::client::BearerAuth;
use rig::completion::{CompletionModel, GetTokenUsage, Message, Prompt};
use rig::prelude::{CompletionClient, ProviderClient};
use rig::providers::{groq, openai};
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub enum Token {
    Delta(String),
    Summary { text: String, upto: usize },
    Done,
    Error(String),
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
                    .build();
                Arc::new(AgentBackend(Arc::new(agent)))
            }
            Provider::Groq => {
                let client = groq::Client::from_env()
                    .map_err(|e| anyhow!("no se pudo crear el cliente groq: {e:?}"))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
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
