use crate::core::config::Config;
use crate::core::llm::Token;
use crate::core::memory::{self, Turn};
use crate::core::store::Store;
use anyhow::Result;
use ratatui::layout::Rect;
use std::time::Instant;

const HELP: &str = "Comandos:\n/help    muestra esta ayuda\n/resume  describe la memoria actual (resumen + estado)\n/clear   borra la conversación y la memoria guardada";

#[derive(Clone, Copy)]
pub struct Selection {
    pub anchor: (usize, usize),
    pub cursor: (usize, usize),
    pub dragging: bool,
}

impl Selection {
    pub fn ordered(&self) -> ((usize, usize), (usize, usize)) {
        if self.anchor <= self.cursor {
            (self.anchor, self.cursor)
        } else {
            (self.cursor, self.anchor)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.cursor
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub activity: Vec<String>,
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
    pub selection: Option<Selection>,
    pub notice: Option<String>,
    pub tok_per_sec: f64,
    store: Store,
    drag_point: Option<(u16, u16)>,
    chat_area: Rect,
    chat_offset: usize,
    chat_plain: Vec<String>,
    chat_prefix: Vec<usize>,
    stream_start: Option<Instant>,
    stream_count: usize,
    summary: Option<String>,
    summarized_upto: usize,
    budget: usize,
}

impl App {
    pub fn new(config: &Config) -> Result<Self> {
        let db = std::env::var("RUSTIO_DB").unwrap_or_else(|_| "rustio.db".into());
        let store = Store::open(&db)?;
        let loaded = store.load()?;
        Ok(Self {
            provider: config.provider.label().to_string(),
            model: config.model.clone(),
            messages: loaded.messages,
            input: String::new(),
            status: Status::Idle,
            spinner: 0,
            scroll: 0,
            should_quit: false,
            selection: None,
            notice: None,
            tok_per_sec: 0.0,
            store,
            drag_point: None,
            chat_area: Rect::default(),
            chat_offset: 0,
            chat_plain: Vec::new(),
            chat_prefix: Vec::new(),
            stream_start: None,
            stream_count: 0,
            summary: loaded.summary,
            summarized_upto: loaded.summarized_upto,
            budget: config.history_budget_tokens,
        })
    }

    fn persist(&self) {
        let _ = self
            .store
            .save(&self.messages, self.summary.as_deref(), self.summarized_upto);
    }

    fn run_command(&mut self, cmd: &str) {
        match cmd.split_whitespace().next().unwrap_or("") {
            "/help" => self.notice = Some(HELP.to_string()),
            "/resume" | "/resumen" => self.notice = Some(self.memory_overview()),
            "/clear" => {
                self.messages.clear();
                self.summary = None;
                self.summarized_upto = 0;
                self.scroll = 0;
                self.selection = None;
                let _ = self.store.clear();
                self.notice = Some("Conversación y memoria borradas.".to_string());
            }
            other => self.notice = Some(format!("Comando desconocido: {other}. Usa /help.")),
        }
    }

    fn memory_overview(&self) -> String {
        let mut out = String::new();
        match &self.summary {
            Some(sum) if !sum.is_empty() => {
                out.push_str("Resumen de memoria:\n");
                out.push_str(sum);
                out.push_str("\n\n");
            }
            _ => out.push_str("Aún no hay resumen comprimido (conversación corta).\n\n"),
        }
        out.push_str(&format!(
            "{} mensajes · {} ya comprimidos · budget {} tokens",
            self.messages.len(),
            self.summarized_upto,
            self.budget
        ));
        out
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

    pub fn set_chat_layout(
        &mut self,
        area: Rect,
        offset: usize,
        plain: Vec<String>,
        prefix: Vec<usize>,
    ) {
        self.chat_area = area;
        self.chat_offset = offset;
        self.chat_plain = plain;
        self.chat_prefix = prefix;
    }

    fn in_chat(&self, col: u16, row: u16) -> bool {
        let a = self.chat_area;
        col >= a.x && col < a.x + a.width && row >= a.y && row < a.y + a.height
    }

    fn cell_clamped(&self, col: u16, row: u16) -> (usize, usize) {
        if self.chat_plain.is_empty() {
            return (0, 0);
        }
        let a = self.chat_area;
        let row = row.clamp(a.y, a.y + a.height.saturating_sub(1));
        let line = (self.chat_offset + (row - a.y) as usize).min(self.chat_plain.len() - 1);
        let max_col = self.chat_plain[line].chars().count();
        let col = (col.saturating_sub(a.x) as usize).min(max_col);
        (line, col)
    }

    pub fn mouse_down(&mut self, col: u16, row: u16) {
        if self.chat_plain.is_empty() || !self.in_chat(col, row) {
            self.selection = None;
            self.drag_point = None;
            return;
        }
        let cell = self.cell_clamped(col, row);
        self.selection = Some(Selection {
            anchor: cell,
            cursor: cell,
            dragging: true,
        });
        self.drag_point = Some((col, row));
    }

    pub fn mouse_drag(&mut self, col: u16, row: u16) {
        if self.chat_plain.is_empty() || !self.selection.map_or(false, |s| s.dragging) {
            return;
        }
        self.drag_point = Some((col, row));
        let cell = self.cell_clamped(col, row);
        if let Some(sel) = &mut self.selection {
            sel.cursor = cell;
        }
    }

    // Scroll continuo mientras se sostiene el arrastre en un borde (crossterm no
    // emite eventos de drag si el mouse no se mueve, así que lo bombea el ticker).
    fn autoscroll_tick(&mut self) {
        if self.chat_plain.is_empty() || !self.selection.map_or(false, |s| s.dragging) {
            return;
        }
        let Some((col, row)) = self.drag_point else {
            return;
        };
        let a = self.chat_area;
        let edge_row = if row < a.y {
            self.scroll_up(1);
            a.y
        } else if a.height > 0 && row >= a.y + a.height {
            self.scroll_down(1);
            a.y + a.height - 1
        } else {
            return;
        };
        let cell = self.cell_clamped(col, edge_row);
        if let Some(sel) = &mut self.selection {
            sel.cursor = cell;
        }
    }

    pub fn copy_selection(&self) -> Option<String> {
        let sel = self.selection?;
        if sel.is_empty() {
            return None;
        }
        Some(self.selection_text())
    }

    pub fn mouse_up(&mut self) -> Option<String> {
        let sel = self.selection.as_mut()?;
        sel.dragging = false;
        self.drag_point = None;
        if sel.is_empty() {
            self.selection = None;
            return None;
        }
        Some(self.selection_text())
    }

    fn selection_text(&self) -> String {
        let Some(sel) = self.selection else {
            return String::new();
        };
        let (start, end) = sel.ordered();
        let last = self.chat_plain.len().saturating_sub(1);
        let mut parts = Vec::new();
        for li in start.0..=end.0.min(last) {
            let chars: Vec<char> = self.chat_plain[li].chars().collect();
            let pre = self.chat_prefix[li];
            let a = if li == start.0 { start.1 } else { 0 };
            let b = if li == end.0 { end.1 } else { chars.len() };
            let a = a.max(pre).min(chars.len());
            let b = b.max(pre).min(chars.len());
            parts.push(if a < b {
                chars[a..b].iter().collect::<String>()
            } else {
                String::new()
            });
        }
        parts.join("\n")
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

        if prompt.starts_with('/') {
            self.run_command(&prompt);
            return None;
        }
        self.notice = None;

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
            activity: Vec::new(),
        });
        self.messages.push(ChatMessage {
            role: Role::Assistant,
            content: String::new(),
            activity: Vec::new(),
        });
        self.status = Status::Thinking;
        self.scroll = 0;
        self.selection = None;
        self.stream_start = None;
        self.stream_count = 0;
        self.tok_per_sec = 0.0;
        self.persist();
        Some(turn)
    }

    pub fn on_token(&mut self, token: Token) {
        match token {
            Token::Delta(delta) => {
                self.status = Status::Streaming;
                let start = *self.stream_start.get_or_insert_with(Instant::now);
                self.stream_count += 1;
                let dt = start.elapsed().as_secs_f64();
                if dt > 0.0 {
                    self.tok_per_sec = self.stream_count as f64 / dt;
                }
                if let Some(last) = self.messages.last_mut() {
                    last.content.push_str(&delta);
                }
            }
            Token::ToolCall { name, preview } => {
                self.status = Status::Streaming;
                if let Some(last) = self.messages.last_mut() {
                    let line = if preview.is_empty() {
                        name
                    } else {
                        format!("{name}  {preview}")
                    };
                    last.activity.push(line);
                }
            }
            Token::ToolResult { summary } => {
                if let Some(last) = self.messages.last_mut() {
                    if let Some(line) = last.activity.last_mut() {
                        line.push_str(&format!("  →  {summary}"));
                    }
                }
            }
            Token::Summary { text, upto } => {
                self.summary = Some(text);
                self.summarized_upto = upto;
                self.persist();
            }
            Token::Done => {
                self.status = Status::Idle;
                self.persist();
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
        self.autoscroll_tick();
    }
}
