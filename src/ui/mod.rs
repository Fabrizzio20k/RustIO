mod chat;
mod footer;
mod header;
mod input;
mod markdown;
pub mod theme;

use crate::core::state::App;
use crate::ui::theme::{ACCENT, ACCENT_SOFT, BAR_BG, MUTED};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

pub fn draw(frame: &mut Frame, app: &mut App) {
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
    draw_command_menu(frame, areas[4], app);
}

fn draw_command_menu(frame: &mut Frame, input_area: Rect, app: &App) {
    let items = app.command_suggestions();
    if items.is_empty() {
        return;
    }

    let height = items.len() as u16;
    let width = 44.min(frame.area().width.saturating_sub(input_area.x).max(1));
    let area = Rect {
        x: input_area.x,
        y: input_area.y.saturating_sub(height),
        width,
        height,
    };

    frame.render_widget(Clear, area);
    frame.render_widget(Block::default().style(Style::default().bg(BAR_BG)), area);

    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let marker = if i == 0 { "› " } else { "  " };
            Line::from(vec![
                Span::styled(marker, Style::default().fg(ACCENT).bg(BAR_BG)),
                Span::styled(format!("{name}  "), Style::default().fg(ACCENT_SOFT).bg(BAR_BG)),
                Span::styled(*desc, Style::default().fg(MUTED).bg(BAR_BG)),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), area);
}
