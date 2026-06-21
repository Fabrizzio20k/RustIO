use crate::ui::theme::{ACCENT_SOFT, CODE_BG, CODE_FG, FAINT, MUTED, TEXT, wrap_text};
use ratatui::style::Style;
use ratatui::text::Span;

pub type Block = (Vec<Span<'static>>, usize);

pub fn render(text: &str, width: usize) -> Vec<Block> {
    let width = width.max(1);
    let mut out: Vec<Block> = Vec::new();
    let mut in_code = false;

    for raw in text.split('\n') {
        let t = raw.trim();

        if t.starts_with("```") {
            in_code = !in_code;
            let label = if in_code {
                let lang = t.trim_start_matches('`').trim();
                if lang.is_empty() {
                    "┌─".to_string()
                } else {
                    format!("┌ {lang}")
                }
            } else {
                "└─".to_string()
            };
            let deco = label.chars().count();
            out.push((vec![Span::styled(label, Style::default().fg(MUTED))], deco));
            continue;
        }

        if in_code {
            for chunk in hard_wrap(raw, width.saturating_sub(2)) {
                out.push((
                    vec![
                        Span::styled("│ ", Style::default().fg(MUTED)),
                        Span::styled(chunk, Style::default().fg(CODE_FG).bg(CODE_BG)),
                    ],
                    2,
                ));
            }
            continue;
        }

        if t == "---" || t == "***" || t == "___" {
            out.push((
                vec![Span::styled("─".repeat(width), Style::default().fg(FAINT))],
                0,
            ));
            continue;
        }

        if let Some(head) = heading(t) {
            for chunk in wrap_text(&head, width) {
                out.push((
                    vec![Span::styled(chunk, Style::default().fg(ACCENT_SOFT).bold())],
                    0,
                ));
            }
            continue;
        }

        for line in wrap_cells(inline_cells(raw), width) {
            out.push((group(line), 0));
        }
    }

    out
}

fn heading(t: &str) -> Option<String> {
    if !t.starts_with('#') {
        return None;
    }
    let rest = t.trim_start_matches('#');
    if rest.is_empty() || rest.starts_with(' ') {
        Some(rest.trim_start().to_string())
    } else {
        None
    }
}

fn hard_wrap(s: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    if s.is_empty() {
        return vec![String::new()];
    }
    let chars: Vec<char> = s.chars().collect();
    chars.chunks(width).map(|c| c.iter().collect()).collect()
}

fn inline_cells(s: &str) -> Vec<(char, Style)> {
    let base = Style::default().fg(TEXT);
    let code = Style::default().fg(CODE_FG).bg(CODE_BG);
    let bold = Style::default().fg(TEXT).bold();
    let chars: Vec<char> = s.chars().collect();
    let mut cells = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c == '`' {
            i += 1;
            while i < chars.len() && chars[i] != '`' {
                cells.push((chars[i], code));
                i += 1;
            }
            i += 1;
            continue;
        }
        if c == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '*') {
                cells.push((chars[i], bold));
                i += 1;
            }
            i += 2;
            continue;
        }
        cells.push((c, base));
        i += 1;
    }

    cells
}

fn wrap_cells(cells: Vec<(char, Style)>, width: usize) -> Vec<Vec<(char, Style)>> {
    let width = width.max(1);
    let mut lines: Vec<Vec<(char, Style)>> = Vec::new();
    let mut line: Vec<(char, Style)> = Vec::new();
    let mut last_space: Option<usize> = None;

    for (c, s) in cells {
        line.push((c, s));
        if c == ' ' {
            last_space = Some(line.len() - 1);
        }
        if line.len() > width {
            if let Some(sp) = last_space {
                let mut rest = line.split_off(sp + 1);
                line.pop();
                lines.push(std::mem::take(&mut line));
                last_space = rest.iter().rposition(|(cc, _)| *cc == ' ');
                line.append(&mut rest);
            } else {
                let tail = line.pop().unwrap();
                lines.push(std::mem::take(&mut line));
                line.push(tail);
                last_space = None;
            }
        }
    }

    lines.push(line);
    lines
}

fn group(cells: Vec<(char, Style)>) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut buf = String::new();
    let mut cur: Option<Style> = None;

    for (c, s) in cells {
        match cur {
            Some(st) if st == s => buf.push(c),
            _ => {
                if let Some(st) = cur {
                    spans.push(Span::styled(std::mem::take(&mut buf), st));
                }
                cur = Some(s);
                buf.push(c);
            }
        }
    }
    if let Some(st) = cur {
        spans.push(Span::styled(buf, st));
    }
    if spans.is_empty() {
        spans.push(Span::raw(""));
    }
    spans
}
