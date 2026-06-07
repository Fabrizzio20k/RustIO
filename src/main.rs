mod core;
mod ui;

use anyhow::Result;
use crate::core::config::Config;
use crate::core::llm::{Llm, Token};
use crate::core::state::App;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedSender};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::load()?;
    let llm = Arc::new(Llm::new(&config)?);
    let mut app = App::new(&config);

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app, llm).await;
    ratatui::restore();
    result
}

async fn run(terminal: &mut DefaultTerminal, app: &mut App, llm: Arc<Llm>) -> Result<()> {
    let mut events = EventStream::new();
    let (tx, mut rx) = mpsc::unbounded_channel::<Token>();
    let mut ticker = tokio::time::interval(Duration::from_millis(90));

    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;
        if app.should_quit {
            return Ok(());
        }

        tokio::select! {
            maybe_event = events.next() => {
                if let Some(Ok(event)) = maybe_event {
                    handle_event(app, event, &llm, &tx);
                }
            }
            Some(token) = rx.recv() => {
                app.on_token(token);
            }
            _ = ticker.tick() => {
                app.tick();
            }
        }
    }
}

fn handle_event(app: &mut App, event: Event, llm: &Arc<Llm>, tx: &UnboundedSender<Token>) {
    let Event::Key(key) = event else {
        return;
    };
    if key.kind != KeyEventKind::Press {
        return;
    }

    match key.code {
        KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Enter => {
            if let Some((history, prompt)) = app.submit() {
                let future = llm.run(history, prompt, tx.clone());
                tokio::spawn(future);
            }
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char(c) => {
            app.input.push(c);
        }
        _ => {}
    }
}
