use crate::core::config::Config;
use crate::core::llm::Token;
use rig::completion::Message;

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
    pub should_quit: bool,
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
            should_quit: false,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.status != Status::Idle
    }

    pub fn history(&self) -> Vec<Message> {
        self.messages
            .iter()
            .map(|m| match m.role {
                Role::User => Message::user(m.content.clone()),
                Role::Assistant => Message::assistant(m.content.clone()),
            })
            .collect()
    }

    pub fn submit(&mut self) -> Option<(Vec<Message>, String)> {
        if self.is_busy() {
            return None;
        }
        let prompt = self.input.trim().to_string();
        if prompt.is_empty() {
            return None;
        }
        let history = self.history();
        self.input.clear();
        self.messages.push(ChatMessage {
            role: Role::User,
            content: prompt.clone(),
        });
        self.messages.push(ChatMessage {
            role: Role::Assistant,
            content: String::new(),
        });
        self.status = Status::Thinking;
        Some((history, prompt))
    }

    pub fn on_token(&mut self, token: Token) {
        match token {
            Token::Delta(delta) => {
                self.status = Status::Streaming;
                if let Some(last) = self.messages.last_mut() {
                    last.content.push_str(&delta);
                }
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
