use std::{collections::VecDeque, ops::Deref};

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Stylize as _,
    symbols::border,
    text::Line,
    widgets::{Block, Borders, List, ListItem, StatefulWidget, Widget},
};

use crate::core::{
    App,
    component::{Focus, WorkSpace},
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(frame.area());

    let App {
        workspace, cursor, ..
    } = app;

    let WorkSpace {
        mid, left, right, ..
    } = workspace;

    // ---- mid --sl
    let mid_block = Block::default()
        .borders(Borders::ALL)
        .title("Mid list")
        .border_style(match cursor.focus {
            Focus::Mid => ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold(),
            _ => ratatui::style::Style::default(),
        });

    let mid_list = List::new(
        mid.items
            .iter()
            .map(|i: &crate::core::component::FileItem| ListItem::new(i.display_name.as_str()))
            .collect::<VecDeque<_>>(),
    )
    .block(mid_block)
    .highlight_symbol(">>");

    frame.render_stateful_widget(mid_list, chunks[1], &mut mid.state); // mid is at index 0 in left_and_mid

    // --- left ---
    let left_block = Block::default()
        .borders(Borders::ALL)
        .title("left list")
        .border_style(match cursor.focus {
            Focus::Left => ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold(),
            _ => ratatui::style::Style::default(),
        });

    let left_list = List::new(
        left.items
            .iter()
            .map(|i: &crate::core::component::FileItem| ListItem::new(i.display_name.as_str()))
            .collect::<VecDeque<_>>(),
    )
    .block(left_block)
    .highlight_symbol(">>");

    frame.render_stateful_widget(left_list, chunks[0], &mut left.state); // left is at index 0 in left_and_mid

    // ---- right ----
    let right_block = Block::default()
        .borders(Borders::ALL)
        .title("right list")
        .border_style(match cursor.focus {
            Focus::Right => ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold(),
            _ => ratatui::style::Style::default(),
        });

    let right_list = List::new(
        right
            .items // right is at index 0 in the `right` slice
            .iter()
            .map(|i: &crate::core::component::FileItem| ListItem::new(i.display_name.as_str()))
            .collect::<VecDeque<_>>(),
    )
    .block(right_block)
    .highlight_symbol(">>");

    frame.render_stateful_widget(right_list, chunks[2], &mut right.state);
}

pub struct AppWrapper<'a>(pub &'a App);

impl<'a> Deref for AppWrapper<'a> {
    type Target = App;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Widget for AppWrapper<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // let title = Line::from("This is the placeholder for the current path")
        //     .blue()
        //     .bold();
        let instructions = Line::from(vec![
            " Move Left ".into(),
            "<LEFT>".blue().bold(),
            " Move Right ".into(),
            "<RIGHT>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            // .title(title.centered())
            .title_bottom(instructions.centered())
            // .border_set(border::THICK)
            ;

        let inner_area = block.inner(area);
        block.render(area, buf);

        self.0
            .workspace
            .render_as_stateful(inner_area, buf, &self.0.cursor);
    }
}
