use crate::core::state::{App, Role, Status};
use crate::ui::theme::{ACCENT, ACCENT_SOFT, FAINT, MUTED, TEXT, spinner_frame, wrap_text};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

const BANNER: [&str; 6] = [
    "██████╗ ██╗   ██╗███████╗████████╗██╗ ██████╗ ",
    "██╔══██╗██║   ██║██╔════╝╚══██╔══╝██║██╔═══██╗",
    "██████╔╝██║   ██║███████╗   ██║   ██║██║   ██║",
    "██╔══██╗██║   ██║╚════██║   ██║   ██║██║   ██║",
    "██║  ██║╚██████╔╝███████║   ██║   ██║╚██████╔╝",
    "╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   ╚═╝ ╚═════╝ ",
];

const BANNER_WIDTH: u16 = 46;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
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

    let viewport = inner.height as usize;
    let max_scroll = lines.len().saturating_sub(viewport);
    if app.scroll > max_scroll {
        app.scroll = max_scroll;
    }
    let offset = (max_scroll - app.scroll) as u16;

    let paragraph = Paragraph::new(lines).scroll((offset, 0));
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
    let mut lines: Vec<Line> = Vec::new();

    if area.width >= BANNER_WIDTH {
        let colors = [ACCENT_SOFT, ACCENT_SOFT, ACCENT, ACCENT, ACCENT, ACCENT];
        for (row, glyphs) in BANNER.iter().enumerate() {
            lines.push(Line::from(Span::styled(
                *glyphs,
                Style::default().fg(colors[row]).bold(),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "▌ rustio",
            Style::default().fg(ACCENT).bold(),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "orquestador de agentes IoT · raspberry pi",
        Style::default().fg(MUTED),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "escribe un mensaje y presiona enter   ·   ↑ ↓ para desplazar",
        Style::default().fg(FAINT),
    )));

    let top = area.height.saturating_sub(lines.len() as u16) / 2;
    let mut padded: Vec<Line> = Vec::new();
    for _ in 0..top {
        padded.push(Line::from(""));
    }
    padded.extend(lines);

    let paragraph = Paragraph::new(padded).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
