use crate::core::state::App;
use crate::ui::theme::{ACCENT, ACCENT_DIM, FAINT, TEXT, wrap_text};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

const MAX_ROWS: u16 = 8;

fn text_width(total_width: u16) -> usize {
    total_width.saturating_sub(4).max(1) as usize
}

pub fn height(app: &App, total_width: u16) -> u16 {
    let rows = if app.input.is_empty() {
        1
    } else {
        wrap_text(&app.input, text_width(total_width)).len().max(1) as u16
    };
    rows.clamp(1, MAX_ROWS) + 2
}

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let border_color = if app.is_busy() { ACCENT_DIM } else { ACCENT };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = if app.input.is_empty() {
        vec![Line::from(vec![
            Span::styled("❯ ", Style::default().fg(ACCENT)),
            Span::styled("envía un mensaje a rustio…", Style::default().fg(FAINT)),
        ])]
    } else {
        let wrapped = wrap_text(&app.input, text_width(area.width));
        let last = wrapped.len().saturating_sub(1);
        wrapped
            .into_iter()
            .enumerate()
            .map(|(index, segment)| {
                let prefix = if index == 0 { "❯ " } else { "  " };
                let mut spans = vec![
                    Span::styled(prefix, Style::default().fg(ACCENT)),
                    Span::styled(segment, Style::default().fg(TEXT)),
                ];
                if index == last {
                    spans.push(Span::styled("▏", Style::default().fg(ACCENT)));
                }
                Line::from(spans)
            })
            .collect()
    };

    let scroll = (lines.len() as u16).saturating_sub(inner.height);
    frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), inner);
}
