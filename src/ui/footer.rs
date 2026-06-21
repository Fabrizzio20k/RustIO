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
    let mut pill = vec![
        Span::styled(
            label,
            Style::default().fg(PILL_FG).bg(ACCENT).bold(),
        ),
        Span::styled(
            format!("  {} · {}", app.provider, app.model),
            Style::default().fg(MUTED).bg(BAR_BG),
        ),
    ];
    if app.tok_per_sec >= 1.0 {
        pill.push(Span::styled(
            format!("  ~{:.0} tok/s", app.tok_per_sec),
            Style::default().fg(ACCENT).bg(BAR_BG),
        ));
    }
    let pill = Line::from(pill);

    let hints = Line::from(vec![
        Span::styled("↑↓", Style::default().fg(MUTED).bg(BAR_BG)),
        Span::styled(" scroll   ", Style::default().fg(FAINT).bg(BAR_BG)),
        Span::styled("enter", Style::default().fg(MUTED).bg(BAR_BG)),
        Span::styled(" enviar   ", Style::default().fg(FAINT).bg(BAR_BG)),
        Span::styled("esc", Style::default().fg(MUTED).bg(BAR_BG)),
        Span::styled(" salir ", Style::default().fg(FAINT).bg(BAR_BG)),
    ]);

    frame.render_widget(Paragraph::new(pill), area);
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Right), area);
}
