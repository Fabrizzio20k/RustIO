mod core;
mod ui;

use anyhow::Result;
use crate::core::config::Config;
use crate::core::llm::{Llm, Token};
use crate::core::state::App;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind,
    KeyModifiers, MouseButton, MouseEventKind,
};
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
    let mut app = App::new(&config)?;

    let mut terminal = ratatui::init();
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = std::fs::write("rustio_panic.log", info.to_string());
        let _ = crossterm::execute!(std::io::stdout(), DisableMouseCapture);
        prev_hook(info);
    }));
    let _ = crossterm::execute!(std::io::stdout(), EnableMouseCapture);
    let result = run(&mut terminal, &mut app, llm).await;
    let _ = crossterm::execute!(std::io::stdout(), DisableMouseCapture);
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
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(app, key, llm, tx),
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => app.mouse_down(mouse.column, mouse.row),
            MouseEventKind::Drag(MouseButton::Left) => app.mouse_drag(mouse.column, mouse.row),
            MouseEventKind::Up(MouseButton::Left) => {
                if let Some(text) = app.mouse_up() {
                    set_clipboard(text);
                }
            }
            MouseEventKind::ScrollUp => app.scroll_up(3),
            MouseEventKind::ScrollDown => app.scroll_down(3),
            _ => {}
        },
        _ => {}
    }
}

// ponytail: nueva instancia por copia; cachear en App si copiar muchísimo molesta
fn set_clipboard(text: String) {
    if !text.is_empty() {
        if let Ok(mut clip) = arboard::Clipboard::new() {
            let _ = clip.set_text(text);
        }
    }
}

fn handle_key(
    app: &mut App,
    key: crossterm::event::KeyEvent,
    llm: &Arc<Llm>,
    tx: &UnboundedSender<Token>,
) {
    match key.code {
        KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            match app.copy_selection() {
                Some(text) => set_clipboard(text),
                None => app.should_quit = true,
            }
        }
        KeyCode::Enter => {
            if let Some(turn) = app.submit() {
                let future = llm.run_turn(turn, tx.clone());
                tokio::spawn(future);
            }
        }
        KeyCode::Up => app.scroll_up(1),
        KeyCode::Down => app.scroll_down(1),
        KeyCode::PageUp => app.scroll_up(10),
        KeyCode::PageDown => app.scroll_down(10),
        KeyCode::Home => app.scroll_to_top(),
        KeyCode::End => app.scroll_to_bottom(),
        KeyCode::Tab => app.complete_command(),
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char(c) => {
            app.input.push(c);
        }
        _ => {}
    }
}
