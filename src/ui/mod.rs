mod chat;
mod footer;
mod header;
mod input;
pub mod theme;

use crate::core::state::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

pub fn draw(frame: &mut Frame, app: &App) {
    let input_height = input::height(app, frame.area().width);
    let areas = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(input_height),
        Constraint::Length(1),
    ])
    .split(frame.area());

    header::draw(frame, areas[0], app);
    chat::draw(frame, areas[2], app);
    input::draw(frame, areas[4], app);
    footer::draw(frame, areas[5], app);
}
