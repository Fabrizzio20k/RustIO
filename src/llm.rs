use anyhow::Result;
use rig::agent::AgentBuilder;
use rig::client::CompletionClient;
use rig::providers::openai;

use crate::config::Config;

pub type Model = openai::CompletionModel;

pub struct LlmFactory {
    client: openai::CompletionsClient,
    model: String,
}

impl LlmFactory {
    pub fn new(cfg: &Config) -> Result<Self> {
        let client = openai::CompletionsClient::builder()
            .api_key(cfg.api_key.as_str())
            .base_url(&cfg.base_url)
            .build()?;
        Ok(Self {
            client,
            model: cfg.model.clone(),
        })
    }

    pub fn agent(&self) -> AgentBuilder<Model> {
        self.client.agent(&self.model)
    }
}
