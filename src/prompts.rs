use std::path::Path;

use anyhow::{Context, Result};

use crate::agents::Role;

pub struct Prompts {
    pub planner: String,
    pub synthesizer: String,
    pub reference: String,
    pub dht11: String,
    pub mq135: String,
    coder: String,
    reviewer: String,
}

impl Prompts {
    pub fn load(dir: &str) -> Result<Self> {
        let read = |name: &str| -> Result<String> {
            let path = Path::new(dir).join(name);
            std::fs::read_to_string(&path)
                .with_context(|| format!("no se pudo leer el prompt {}", path.display()))
        };

        Ok(Self {
            planner: read("planner.md")?,
            synthesizer: read("synthesizer.md")?,
            reference: read("reference.md")?,
            dht11: read("dht11.md")?,
            mq135: read("mq135.md")?,
            coder: read("coder.md")?,
            reviewer: read("reviewer.md")?,
        })
    }

    pub fn worker(&self, role: Role) -> &str {
        match role {
            Role::Coder => &self.coder,
            Role::Reviewer => &self.reviewer,
        }
    }
}
