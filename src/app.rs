use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use uuid::{ContextV7, Timestamp, Uuid};

use crate::commands::{CommandManager, FileItem, ListType};

use color_eyre::Result as Res;

#[derive(Debug, Default, Clone, Copy)]
pub struct Cursor {
    cursor_pos: ListType,
    cursor_col: Option<usize>,
}

#[derive(Debug)]
pub struct App {
    pub manager: CommandManager,
    cursor: Cursor,
    curr_path: PathBuf,
    exit: bool,
}

impl App {
    pub fn new() -> Res<Self> {
        Ok(Self {
            manager: CommandManager::new(Self::initial_pending(&env::current_dir()?)?),
            cursor: Cursor {
                cursor_pos: ListType::Pending,
                cursor_col: Some(0),
            },
            curr_path: env::current_dir()?,
            exit: false,
        })
    }

    pub fn initial_pending(path: &Path) -> Res<Vec<FileItem>> {
        let context = ContextV7::new();
        let dir = fs::read_dir(path)?;
        let mut res: Vec<FileItem> = Vec::with_capacity(8);
        for entry in dir {
            let item = FileItem::new(
                Uuid::new_v7(Timestamp::from_unix(&context, 1497624119, 1234)),
                entry?,
            )?;
            res.push(item);
        }
        Ok(res)
    }

    // runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
    /// updates the application's state based on user input
    fn handle_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => {
                self.cursor.cursor_col = self.cursor.cursor_col.map(|idx| idx + 1);
            }
            KeyCode::Down => {
                self.cursor.cursor_col = self.cursor.cursor_col.map(|idx| idx + 1);
            }
            // ctrl+z undo
            KeyCode::Char('z') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.manager.undo();
                }
            }
            KeyCode::Left => {
                self.manager.move_item(ListType::Pending, ListType::Left);
            }
            KeyCode::Right => {
                self.manager.move_item(ListType::Pending, ListType::Right);
            }
            _ => {
                panic!("")
            }
        }
    }
    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(self.curr_path.to_string_lossy().to_string().blue().bold());
        let instructions = Line::from(vec![
            " Move Left ".into(),
            "<Left>".blue().bold(),
            " Move Right ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let columns = Layout::horizontal([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ]);
        let [left_area, mid_area, right_area] = columns.areas(inner_area);

        let left_items: Vec<Line> = self
            .manager
            .left
            .iter()
            .map(|item| item.colorize())
            .collect();
        Paragraph::new(left_items)
            .block(Block::bordered().title("Left"))
            .render(left_area, buf);

        let pending_items: Vec<Line> = self
            .manager
            .pending
            .iter()
            .map(|item| item.colorize())
            .collect();
        Paragraph::new(pending_items)
            .block(Block::bordered().title("Unselected"))
            .render(mid_area, buf);

        let right_items: Vec<Line> = self
            .manager
            .right
            .iter()
            .map(|item| item.colorize())
            .collect();
        Paragraph::new(right_items)
            .block(Block::bordered().title("Right"))
            .render(right_area, buf);
    }
}
