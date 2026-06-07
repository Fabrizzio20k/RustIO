use crate::core::state::{App, Status};
use crate::ui::theme::{ACCENT, BAR_BG, FAINT, MUTED, PILL_FG};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(
        Block::default().style(Style::default().bg(BAR_BG)),
        area,
    );

    let label = match app.status {
        Status::Idle => " listo ",
        Status::Thinking => " pensando ",
        Status::Streaming => " respondiendo ",
    };
    let pill = Line::from(vec![
        Span::styled(
            label,
            Style::default().fg(PILL_FG).bg(ACCENT).bold(),
        ),
        Span::styled(
            format!("  {} · {}", app.provider, app.model),
            Style::default().fg(MUTED).bg(BAR_BG),
        ),
    ]);

    let hints = Line::from(vec![
        Span::styled("enter", Style::default().fg(MUTED).bg(BAR_BG)),
        Span::styled(" enviar   ", Style::default().fg(FAINT).bg(BAR_BG)),
        Span::styled("esc", Style::default().fg(MUTED).bg(BAR_BG)),
        Span::styled(" salir ", Style::default().fg(FAINT).bg(BAR_BG)),
    ]);

    frame.render_widget(Paragraph::new(pill), area);
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Right), area);
}
