use crate::core::state::{App, Role, Status};
use crate::ui::theme::{ACCENT, ACCENT_SOFT, FAINT, MUTED, TEXT, spinner_frame, wrap_text};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let inner = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(3),
        height: area.height,
    };

    if app.messages.is_empty() {
        draw_splash(frame, inner);
        return;
    }

    let content_width = inner.width.saturating_sub(2).max(1) as usize;
    let lines = build_lines(app, content_width);

    let scroll = (lines.len() as u16).saturating_sub(inner.height);
    let paragraph = Paragraph::new(lines).scroll((scroll, 0));
    frame.render_widget(paragraph, inner);
}

fn build_lines(app: &App, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();

    for (index, message) in app.messages.iter().enumerate() {
        let (glyph_color, label, label_color) = match message.role {
            Role::User => (FAINT, "tú", MUTED),
            Role::Assistant => (ACCENT, "rustio", ACCENT_SOFT),
        };

        lines.push(Line::from(vec![
            Span::styled("▌ ", Style::default().fg(glyph_color)),
            Span::styled(label, Style::default().fg(label_color).bold()),
        ]));

        let is_last = index + 1 == app.messages.len();
        if message.content.is_empty() && is_last && app.status == Status::Thinking {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(spinner_frame(app.spinner), Style::default().fg(ACCENT)),
                Span::styled(" pensando…", Style::default().fg(MUTED)),
            ]));
        } else {
            for content_line in wrap_text(&message.content, width) {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(content_line, Style::default().fg(TEXT)),
                ]));
            }
        }

        lines.push(Line::from(""));
    }

    lines
}

fn draw_splash(frame: &mut Frame, area: Rect) {
    let splash = vec![
        Line::from(Span::styled(
            "▌ rustio",
            Style::default().fg(ACCENT).bold(),
        )),
        Line::from(Span::styled(
            "orquestador de agentes IoT",
            Style::default().fg(MUTED),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "escribe un mensaje y presiona enter",
            Style::default().fg(FAINT),
        )),
    ];

    let top = area.height.saturating_sub(splash.len() as u16) / 2;
    let mut lines: Vec<Line> = Vec::new();
    for _ in 0..top {
        lines.push(Line::from(""));
    }
    lines.extend(splash);

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
