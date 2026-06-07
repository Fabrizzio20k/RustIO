use crate::core::state::App;
use crate::ui::theme::{ACCENT, ACCENT_SOFT, FAINT, MUTED};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let brand = Line::from(vec![
        Span::styled(" ▌ ", Style::default().fg(ACCENT)),
        Span::styled("rustio", Style::default().fg(ACCENT_SOFT).bold()),
        Span::styled("  orquestador IoT", Style::default().fg(MUTED)),
    ]);

    let model = Line::from(vec![
        Span::styled(format!("{} ", app.provider), Style::default().fg(MUTED)),
        Span::styled("· ", Style::default().fg(FAINT)),
        Span::styled(format!("{} ", app.model), Style::default().fg(ACCENT)),
    ]);

    frame.render_widget(Paragraph::new(brand), area);
    frame.render_widget(Paragraph::new(model).alignment(Alignment::Right), area);
}
