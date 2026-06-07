use crate::core::config::{Config, Provider};
use anyhow::{Result, anyhow};
use futures::StreamExt;
use futures::future::BoxFuture;
use rig::agent::{Agent, MultiTurnStreamItem};
use rig::client::BearerAuth;
use rig::completion::{CompletionModel, GetTokenUsage, Message};
use rig::prelude::{CompletionClient, ProviderClient};
use rig::providers::{groq, openai};
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub enum Token {
    Delta(String),
    Done,
    Error(String),
}

trait Chat: Send + Sync {
    fn run(
        &self,
        history: Vec<Message>,
        prompt: String,
        tx: UnboundedSender<Token>,
    ) -> BoxFuture<'static, ()>;
}

struct AgentChat<M: CompletionModel>(Arc<Agent<M>>);

impl<M> Chat for AgentChat<M>
where
    M: CompletionModel + 'static,
    M::StreamingResponse: GetTokenUsage,
{
    fn run(
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
    inner: Box<dyn Chat>,
}

impl Llm {
    pub fn new(config: &Config) -> Result<Self> {
        let inner: Box<dyn Chat> = match config.provider {
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
                Box::new(AgentChat(Arc::new(agent)))
            }
            Provider::Groq => {
                let client = groq::Client::from_env()
                    .map_err(|e| anyhow!("no se pudo crear el cliente groq: {e:?}"))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
                    .build();
                Box::new(AgentChat(Arc::new(agent)))
            }
        };

        Ok(Self { inner })
    }

    pub fn run(
        &self,
        history: Vec<Message>,
        prompt: String,
        tx: UnboundedSender<Token>,
    ) -> BoxFuture<'static, ()> {
        self.inner.run(history, prompt, tx)
    }
}
