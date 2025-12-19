use std::collections::VecDeque;

use ratatui::{
    Frame, layout::{Constraint, Layout}, style::Stylize, widgets::{Block, Borders, List, ListItem}
};

use crate::app::{App, message::ListType};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
        ])
        .split(frame.area());

    let App {
        left,
        right,
        mid,
        cursor,
        ..
    } = app;

    // ---- mid ---
    let mid_block = Block::default()
        .borders(Borders::ALL)
        .title("Mid list")
        .border_style(match cursor.focus {
            ListType::Mid => ratatui::style::Style::default().fg(ratatui::style::Color::Yellow).bold(),
            _ => ratatui::style::Style::default(),
        });

    let mid_list = List::new(
        left.items
            .iter()
            .map(|i: &crate::app::message::FileItem| ListItem::new(i.display_name.as_str()))
            .collect::<VecDeque<_>>(),
    )
    .block(mid_block)
    .highlight_symbol(">>");

    frame.render_stateful_widget(mid_list, chunks[1], &mut mid.state);

    // --- left ---
    let left_block = Block::default()
        .borders(Borders::ALL)
        .title("left list")
        .border_style(match cursor.focus {
            ListType::Left => ratatui::style::Style::default().fg(ratatui::style::Color::Yellow).bold(),
            _ => ratatui::style::Style::default(),
        });

    let left_list = List::new(
        left.items
            .iter()
            .map(|i: &crate::app::message::FileItem| ListItem::new(i.display_name.as_str()))
            .collect::<VecDeque<_>>(),
    )
    .block(left_block)
    .highlight_symbol(">>");

    frame.render_stateful_widget(left_list, chunks[0], &mut left.state);

    // ---- right ----
    let right_block = Block::default()
        .borders(Borders::ALL)
        .title("right list")
        .border_style(match cursor.focus {
            ListType::Right => ratatui::style::Style::default().fg(ratatui::style::Color::Yellow).bold(),
            _ => ratatui::style::Style::default(),
        });

    let right_list = List::new(
        left.items
            .iter()
            .map(|i: &crate::app::message::FileItem| ListItem::new(i.display_name.as_str()))
            .collect::<VecDeque<_>>(),
    )
    .block(right_block)
    .highlight_symbol(">>");

    frame.render_stateful_widget(right_list, chunks[2], &mut right.state);



}
