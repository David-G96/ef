use core::fmt;

use ratatui::{
    style::{Color, Style},
    widgets::Paragraph,
};

#[derive(Debug, Default, Clone)]
pub struct InputBox {
    input: String,
    char_index: usize,
}

impl InputBox {
    pub fn new() -> Self {
        Self::default()
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_index)
            .unwrap_or(self.input.len())
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_index.saturating_sub(1);
        self.char_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_index.saturating_add(1);
        self.char_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        if self.char_index > 0 {
            let byte_idx = self.byte_index();
            let prev_byte_idx = self
                .input
                .char_indices()
                .rfind(|(i, _)| *i < byte_idx)
                .map(|(i, _)| i)
                .unwrap_or(0);

            self.input.replace_range(prev_byte_idx..byte_idx, "");
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn as_paragraph(&'_ self, is_editing: bool) -> Paragraph<'_> {
        Paragraph::new(self.input.as_str()).style(if is_editing {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        })
        // .block(Block::bordered().title("Input"))
    }
}

impl fmt::Display for InputBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}
