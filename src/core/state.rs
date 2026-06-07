use crate::core::config::Config;
use crate::core::llm::Token;
use crate::core::memory::{self, Turn};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Idle,
    Thinking,
    Streaming,
}

pub struct App {
    pub provider: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub status: Status,
    pub spinner: usize,
    pub scroll: usize,
    pub should_quit: bool,
    summary: Option<String>,
    summarized_upto: usize,
    budget: usize,
}

impl App {
    pub fn new(config: &Config) -> Self {
        Self {
            provider: config.provider.label().to_string(),
            model: config.model.clone(),
            messages: Vec::new(),
            input: String::new(),
            status: Status::Idle,
            spinner: 0,
            scroll: 0,
            should_quit: false,
            summary: None,
            summarized_upto: 0,
            budget: config.history_budget_tokens,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.status != Status::Idle
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_add(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = usize::MAX / 2;
    }

    pub fn submit(&mut self) -> Option<Turn> {
        if self.is_busy() {
            return None;
        }
        let prompt = self.input.trim().to_string();
        if prompt.is_empty() {
            return None;
        }
        self.input.clear();

        let prior = &self.messages[self.summarized_upto..];
        let (overflow, recent) = memory::plan(prior, self.summary.as_deref(), self.budget);
        let new_upto = self.summarized_upto + overflow.len();

        let turn = Turn {
            summary: self.summary.clone(),
            overflow,
            recent,
            prompt: prompt.clone(),
            new_upto,
        };

        self.messages.push(ChatMessage {
            role: Role::User,
            content: prompt,
        });
        self.messages.push(ChatMessage {
            role: Role::Assistant,
            content: String::new(),
        });
        self.status = Status::Thinking;
        self.scroll = 0;
        Some(turn)
    }

    pub fn on_token(&mut self, token: Token) {
        match token {
            Token::Delta(delta) => {
                self.status = Status::Streaming;
                if let Some(last) = self.messages.last_mut() {
                    last.content.push_str(&delta);
                }
            }
            Token::Summary { text, upto } => {
                self.summary = Some(text);
                self.summarized_upto = upto;
            }
            Token::Done => {
                self.status = Status::Idle;
            }
            Token::Error(err) => {
                if let Some(last) = self.messages.last_mut() {
                    if last.content.is_empty() {
                        last.content = format!("[error] {err}");
                    } else {
                        last.content.push_str(&format!("\n[error] {err}"));
                    }
                }
                self.status = Status::Idle;
            }
        }
    }

    pub fn tick(&mut self) {
        if self.is_busy() {
            self.spinner = self.spinner.wrapping_add(1);
        }
    }
}
