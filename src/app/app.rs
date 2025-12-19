use std::collections::VecDeque;

use color_eyre::Result as Res;
use color_eyre::eyre::Ok;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, KeyEvent, KeyEventKind},
    widgets::ListState,
};

use crate::{
    app::message::{FileItem, ListType},
    ui,
};

#[derive(Debug)]
pub struct ScrollList {
    pub items: VecDeque<FileItem>,
    pub state: ListState,
}

impl ScrollList {
    pub fn new() -> Self {
        Self {
            items: VecDeque::default(),
            state: ListState::default(),
        }
    }
}

#[derive(Debug)]
pub struct Cursor {
    pub focus: ListType,
    pub index: usize,
}

impl Cursor {
    pub fn new(focus: ListType) -> Self {
        Self { focus, index: 0 }
    }
}

#[derive(Debug)]
pub struct App {
    pub(crate) mid: ScrollList,
    pub(crate) left: ScrollList,
    pub(crate) right: ScrollList,

    pub cursor: Cursor,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            mid: ScrollList::new(),
            left: ScrollList::new(),
            right: ScrollList::new(),
            cursor: Cursor::new(ListType::Mid),
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| ui::render(frame, self))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> Res<()> {
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {
                todo!()
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            event::KeyCode::Left => {
                self.cursor.focus = self.cursor.focus.left();
            }
            event::KeyCode::Right => {
                self.cursor.focus = self.cursor.focus.right();
            }
            _ => {
                todo!()
            }
        }
    }
}
