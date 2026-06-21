use crate::core::state::{App, Role, Status};
use crate::ui::markdown;
use crate::ui::theme::{ACCENT, ACCENT_SOFT, FAINT, MUTED, SELECT_BG, spinner_frame, wrap_text};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
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

    if app.messages.is_empty() && app.notice.is_none() {
        app.set_chat_layout(inner, 0, Vec::new(), Vec::new());
        draw_splash(frame, inner);
        return;
    }

    let content_width = inner.width.saturating_sub(2).max(1) as usize;
    let (mut lines, plain, prefix) = build_lines(app, content_width);

    let viewport = inner.height as usize;
    let max_scroll = lines.len().saturating_sub(viewport);
    if app.scroll > max_scroll {
        app.scroll = max_scroll;
    }
    let offset = max_scroll - app.scroll;

    app.set_chat_layout(inner, offset, plain.clone(), prefix);
    apply_selection(&mut lines, &plain, app);

    let paragraph = Paragraph::new(lines).scroll((offset as u16, 0));
    frame.render_widget(paragraph, inner);
}

fn build_lines(app: &App, width: usize) -> (Vec<Line<'static>>, Vec<String>, Vec<usize>) {
    let mut lines: Vec<Line> = Vec::new();
    let mut plain: Vec<String> = Vec::new();
    let mut prefix: Vec<usize> = Vec::new();

    let mut push = |spans: Vec<Span<'static>>, pre: usize| {
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        prefix.push(pre.min(text.chars().count()));
        plain.push(text);
        lines.push(Line::from(spans));
    };

    for (index, message) in app.messages.iter().enumerate() {
        let (glyph_color, label, label_color) = match message.role {
            Role::User => (FAINT, "tú", MUTED),
            Role::Assistant => (ACCENT, "rustio", ACCENT_SOFT),
        };

        let label_spans = vec![
            Span::styled("▌ ", Style::default().fg(glyph_color)),
            Span::styled(label, Style::default().fg(label_color).bold()),
        ];
        push(label_spans, usize::MAX);

        let is_last = index + 1 == app.messages.len();
        if message.content.is_empty() && is_last && app.status == Status::Thinking {
            push(
                vec![
                    Span::raw("  "),
                    Span::styled(spinner_frame(app.spinner), Style::default().fg(ACCENT)),
                    Span::styled(" pensando…", Style::default().fg(MUTED)),
                ],
                usize::MAX,
            );
        } else {
            for (spans, deco) in markdown::render(&message.content, width.saturating_sub(2)) {
                let mut full = vec![Span::raw("  ")];
                full.extend(spans);
                push(full, 2 + deco);
            }
        }

        push(vec![Span::raw("")], 0);
    }

    if let Some(notice) = &app.notice {
        push(
            vec![Span::styled("▌ sistema", Style::default().fg(ACCENT_SOFT).bold())],
            usize::MAX,
        );
        for line in wrap_text(notice, width.saturating_sub(2)) {
            push(
                vec![Span::raw("  "), Span::styled(line, Style::default().fg(MUTED))],
                2,
            );
        }
        push(vec![Span::raw("")], 0);
    }

    (lines, plain, prefix)
}

fn apply_selection(lines: &mut [Line<'static>], plain: &[String], app: &App) {
    let Some(sel) = app.selection else { return };
    if sel.is_empty() {
        return;
    }
    let (start, end) = sel.ordered();
    let last = lines.len().saturating_sub(1);
    for li in start.0..=end.0.min(last) {
        let lo = if li == start.0 { start.1 } else { 0 };
        let hi = if li == end.0 {
            end.1
        } else {
            plain[li].chars().count()
        };
        lines[li] = highlight(&lines[li], lo, hi, SELECT_BG);
    }
}

fn highlight(line: &Line<'static>, lo: usize, hi: usize, bg: Color) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();
    let mut cur: Option<Style> = None;
    let mut col = 0usize;

    for span in &line.spans {
        for ch in span.content.chars() {
            let style = if col >= lo && col < hi {
                span.style.bg(bg)
            } else {
                span.style
            };
            match cur {
                Some(st) if st == style => buf.push(ch),
                _ => {
                    if let Some(st) = cur {
                        spans.push(Span::styled(std::mem::take(&mut buf), st));
                    }
                    cur = Some(style);
                    buf.push(ch);
                }
            }
            col += 1;
        }
    }
    if let Some(st) = cur {
        spans.push(Span::styled(buf, st));
    }
    Line::from(spans)
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
