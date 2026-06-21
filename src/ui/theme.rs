use ratatui::style::Color;

pub const ACCENT: Color = Color::Rgb(56, 189, 248);
pub const ACCENT_SOFT: Color = Color::Rgb(125, 211, 252);
pub const ACCENT_DIM: Color = Color::Rgb(30, 90, 120);
pub const TEXT: Color = Color::Rgb(222, 228, 234);
pub const MUTED: Color = Color::Rgb(120, 132, 144);
pub const FAINT: Color = Color::Rgb(78, 86, 96);
pub const BAR_BG: Color = Color::Rgb(20, 26, 33);
pub const PILL_FG: Color = Color::Rgb(8, 12, 16);
pub const CODE_FG: Color = Color::Rgb(152, 195, 121);
pub const CODE_BG: Color = Color::Rgb(28, 34, 42);
pub const SELECT_BG: Color = Color::Rgb(45, 72, 96);

pub const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn spinner_frame(tick: usize) -> &'static str {
    SPINNER[tick % SPINNER.len()]
}

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out: Vec<String> = Vec::new();

    for raw in text.split('\n') {
        if raw.is_empty() {
            out.push(String::new());
            continue;
        }

        let mut line = String::new();
        let mut len = 0usize;

        for word in raw.split(' ') {
            let word_len = word.chars().count();

            if word_len > width {
                if !line.is_empty() {
                    out.push(std::mem::take(&mut line));
                }
                let mut chunk = String::new();
                let mut chunk_len = 0usize;
                for ch in word.chars() {
                    if chunk_len == width {
                        out.push(std::mem::take(&mut chunk));
                        chunk_len = 0;
                    }
                    chunk.push(ch);
                    chunk_len += 1;
                }
                line = chunk;
                len = chunk_len;
                continue;
            }

            let extra = if line.is_empty() { word_len } else { word_len + 1 };
            if len + extra > width {
                out.push(std::mem::take(&mut line));
                line = word.to_string();
                len = word_len;
            } else {
                if !line.is_empty() {
                    line.push(' ');
                    len += 1;
                }
                line.push_str(word);
                len += word_len;
            }
        }

        out.push(line);
    }

    out
}
