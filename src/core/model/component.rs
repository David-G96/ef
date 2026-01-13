//! Reusable common components for models

pub mod fps;
pub mod input;
pub mod popup;
use std::collections::VecDeque;

use ratatui::{
    style::{self, Style, Stylize as _},
    text::Line,
    widgets::{Block, ListState, Paragraph, Widget},
};

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileItem {
    pub id: u64,
    pub path: PathBuf,
    /// dir后面加斜杠以示区分，并使用蓝色 e.g. `lib/
    pub display_name: String,
    pub is_dir: bool,
}

impl FileItem {
    pub fn as_line(&self) -> Line<'static> {
        if self.is_dir {
            Line::from(format!("{}/", &self.display_name).blue())
        } else {
            Line::from(self.display_name.clone())
        }
    }
}

impl Widget for &FileItem {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.as_line().render(area, buf)
    }
}

impl std::fmt::Display for FileItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(core::format_args!(
            "File<{}>: path={}, display_name={}, is_dir={}",
            self.id,
            self.path.to_string_lossy(),
            self.display_name,
            self.is_dir
        ))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ListType {
    /// The middle list, used as pending/register
    #[default]
    Mid,
    Left,
    Right,
}

impl ListType {
    pub fn left(self) -> Self {
        match self {
            Self::Mid => Self::Left,
            Self::Left => Self::Right,
            Self::Right => Self::Mid,
        }
    }
    pub fn right(self) -> Self {
        match self {
            Self::Mid => Self::Right,
            Self::Left => Self::Mid,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct History<T> {
    history: Vec<T>,
    /// points to the next command
    top: usize,
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            top: 0,
        }
    }
}

impl<T> History<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&mut self, cmd: T) {
        if self.history.len() <= self.top {
            self.history.push(cmd);
        } else {
            self.history[self.top] = cmd;
        }
        self.top = self.top.saturating_add(1);
    }

    pub fn undo(&mut self) {
        self.top = self.top.saturating_sub(1);
    }

    pub fn redo(&mut self) {
        if self.top < self.history.len() - 1 {
            self.top = self.top.saturating_add(1);
        }
    }

    pub fn last(&self) -> Option<&T> {
        if self.top == 0 {
            None
        } else {
            Some(&self.history[self.top - 1])
        }
    }

    pub fn count(&self) -> usize {
        self.history.len()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ScrollList {
    pub items: VecDeque<FileItem>,
    pub state: ListState,
}

impl ScrollList {
    pub fn new(items: VecDeque<FileItem>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        Self { items, state }
    }

    pub fn up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len().saturating_sub(1) {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn render_with_border<'a>(
        &'a self,
        is_focus: bool,
        index: Option<usize>,
        title: &'a str,
    ) -> Paragraph<'a> {
        let style = if is_focus {
            Style::default().fg(style::Color::Yellow).bold()
        } else {
            style::Style::default().dim()
        };
        self.render(is_focus, index)
            .block(Block::bordered().title(title).border_style(style))
    }

    pub fn render<'a>(&'a self, is_focus: bool, index: Option<usize>) -> Paragraph<'a> {
        let lines = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let mut line = item.as_line();
                if is_focus && index.map(|_i| _i == i).unwrap_or(false) {
                    line = line.reversed();
                }
                line
            })
            .collect::<Vec<Line>>();
        Paragraph::new(lines)
    }
}

/// 这个玩意还有啥用？其实没有，因为ListState已经取代了index的作用了。
#[derive(Debug, Default, Clone, Copy)]
pub struct Cursor {
    pub focus: ListType,
}

impl Cursor {
    pub fn new(focus: ListType) -> Self {
        Self { focus }
    }
}
