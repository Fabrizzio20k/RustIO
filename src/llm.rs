use rig::agent::AgentBuilder;
use rig::extractor::ExtractorBuilder;
use rig::providers::openai;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::config::Config;

pub type Model = openai::CompletionModel;

pub struct LlmFactory {
    client: openai::Client,
    model: String,
}

impl LlmFactory {
    pub fn new(cfg: &Config) -> Self {
        let client = openai::Client::from_url(&cfg.api_key, &cfg.base_url);
        Self {
            client,
            model: cfg.model.clone(),
        }
    }

    pub fn agent(&self) -> AgentBuilder<Model> {
        self.client.agent(&self.model)
    }

    pub fn extractor<T>(&self) -> ExtractorBuilder<T, Model>
    where
        T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static,
    {
        self.client.extractor::<T>(&self.model)
    }
}
