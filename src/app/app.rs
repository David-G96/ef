use std::{
    collections::VecDeque,
    env::current_dir,
    fs::{self},
    path::Path,
};

use color_eyre::{Result as Res, eyre::Ok};
use core::result::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, KeyEvent, KeyEventKind},
};
use uuid::{ContextV7, Timestamp, Uuid};

use crate::{
    app::{
        components::{Cursor, FileItem, WorkSpace},
        config::Config,
        events::{AppEvent, EventHandler},
    },
    ui,
};

#[derive(Debug)]
pub struct App {
    pub workspace: WorkSpace,
    pub config: Config,
    pub handler: EventHandler,

    pub cursor: Cursor,

    // pub tick_rate : f64 = 12.0,
    // pub frame_rate : f64 = 60.0,
    // pub watch_rate : 64 = 2.0,
    exit: bool,
}

impl App {
    pub fn new() -> Res<Self> {
        let current_dir = current_dir()?;
        Ok(Self {
            workspace: WorkSpace::new(Self::initial_pending(&current_dir)?),
            config: Config::default(),
            handler: EventHandler::new(current_dir),
            cursor: Cursor::default(),
            exit: false,
        })
    }

    pub fn initial_pending(path: &Path) -> Res<VecDeque<FileItem>> {
        let context = ContextV7::new();
        let dir = fs::read_dir(path)?;
        let mut res: VecDeque<FileItem> = VecDeque::with_capacity(8);
        for entry in dir {
            let entry = entry?;
            let item = FileItem {
                id: Uuid::new_v7(Timestamp::from_unix(&context, 1497624119, 1234)),
                path: entry.path(),
                display_name: entry.file_name().to_string_lossy().to_string(),
                is_dir: entry.file_type()?.is_dir(),
            };
            res.push_back(item);
        }
        Ok(res)
    }

    pub fn run_with_term(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            match self.handler.next_timeout() {
                Result::Ok(AppEvent::Tick) => {
                    terminal.draw(|frame| ui::render(frame, self))?;
                }
                Result::Ok(AppEvent::Key(key_event)) => {
                    self.handle_key_event(key_event);
                    terminal.draw(|f| ui::render(f, self))?;
                }
                Result::Ok(AppEvent::Input(key_code)) => {
                    match key_code {
                        event::KeyCode::Left => {
                            // self.cursor.focus = self.cursor.focus.left();
                            self.workspace
                                .move_item(self.cursor, self.cursor.move_left());
                        }
                        event::KeyCode::Right => {
                            // self.cursor.focus = self.cursor.focus.right();
                            self.workspace
                                .move_item(self.cursor, self.cursor.move_right());
                        }
                        event::KeyCode::Up => {
                            if self.cursor.index
                                < self.workspace.get_list_mut(self.cursor.focus).len()
                            {
                                todo!()
                            }
                            self.cursor.index = self.cursor.index.saturating_add(1);
                        }
                        event::KeyCode::Char('q') => return Ok(()),
                        _ => {
                            todo!()
                        }
                    }
                }

                Err(_) => {}
                _ => {
                    todo!()
                }
            }
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> Res<()> {
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {
                tracing::warn!("event::read something else than key")
                // Do nothing
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            event::KeyCode::Left => {
                // self.cursor.focus = self.cursor.focus.left();
                self.workspace
                    .move_item(self.cursor, self.cursor.move_left());
            }
            event::KeyCode::Right => {
                // self.cursor.focus = self.cursor.focus.right();
                self.workspace
                    .move_item(self.cursor, self.cursor.move_right());
            }
            event::KeyCode::Up => {
                if self.cursor.index < self.workspace.get_list_mut(self.cursor.focus).len() {
                    todo!()
                }
                self.cursor.index = self.cursor.index.saturating_add(1);
            }
            _ => {
                todo!()
            }
        }
    }
}
