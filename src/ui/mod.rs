mod chat;
mod footer;
mod header;
mod input;
mod markdown;
pub mod theme;

use crate::core::state::{App, ModelSetupStep};
use crate::ui::theme::{ACCENT, ACCENT_SOFT, BAR_BG, MUTED};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, List, ListItem, Borders};

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
    
    if app.model_setup.is_some() {
        draw_model_setup(frame, frame.area(), app);
    }
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

fn draw_model_setup(frame: &mut Frame, area: Rect, app: &App) {
    let Some(setup) = &app.model_setup else { return };
    
    let popup_layout = Layout::vertical([
        Constraint::Percentage(20),
        Constraint::Length(12),
        Constraint::Percentage(20),
    ])
    .split(area)[1];
    
    let popup_area = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Min(40),
        Constraint::Percentage(20),
    ])
    .split(popup_layout)[1];

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Selección de Modelo ")
        .borders(Borders::ALL)
        .style(Style::default().bg(BAR_BG).fg(ACCENT));

    if let ModelSetupStep::CategoryMenu | ModelSetupStep::PresetMenu { .. } = setup.step {
        let items: Vec<ListItem> = setup.menu_items.iter().enumerate().map(|(i, s)| {
            let style = if i == setup.selected {
                Style::default().bg(ACCENT).fg(ratatui::style::Color::Black)
            } else {
                Style::default()
            };
            ListItem::new(s.clone()).style(style)
        }).collect();
        
        let list = List::new(items).block(block);
        frame.render_widget(list, popup_area);
    } else {
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);
        
        let notice = setup.notice.as_deref().unwrap_or("");
        let input_line = format!("> {}_{}", setup.input, " ");
        
        let color = if notice.starts_with("Error") {
            ratatui::style::Color::Red
        } else {
            MUTED
        };
        
        let mut lines: Vec<Line> = notice
            .split('\n')
            .map(|s| Line::from(Span::styled(s, Style::default().fg(color))))
            .collect();
            
        lines.push(Line::from(""));
        lines.push(Line::from(input_line));
        
        let p = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(p, inner);
    }
}
