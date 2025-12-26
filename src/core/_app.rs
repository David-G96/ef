use std::{
    collections::VecDeque,
    env::current_dir,
    fs::{self},
    path::Path,
};

use color_eyre::{Result as Res, eyre::Ok};
use core::result::Result;
use crossterm::event::KeyCode;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, KeyEvent, KeyEventKind},
};
use uuid::{ContextV7, Timestamp, Uuid};

use crate::{
    core::{
        component::{Cursor, FileItem, WorkSpace},
        config::Config,
        events::{AppEvent, BackendHandler},
        _model::AppMode,
    },
    ui::AppWrapper,
};

#[derive(Debug)]
pub struct App {
    // todo: 修改workspace使其拥有path
    pub workspace: WorkSpace,
    pub config: Config,
    pub handler: BackendHandler,

    pub app_mode: AppMode,

    pub cursor: Cursor,
    should_redraw: bool,
    exit: bool,
}

impl App {
    pub fn new() -> Res<Self> {
        let current_dir = current_dir()?;
        Ok(Self {
            workspace: WorkSpace::new(current_dir.clone(), Self::read_dir(&current_dir)?),
            config: Config::default(),
            handler: BackendHandler::new(current_dir),
            app_mode: AppMode::default(),
            cursor: Cursor::default(),
            exit: false,
            should_redraw: true, // 初始状态需要绘制一次
        })
    }

    pub fn read_dir(path: &Path) -> Res<VecDeque<FileItem>> {
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
            // 1. 渲染阶段：只有在标记为脏（需要重绘）时才调用 draw
            if self.should_redraw {
                self.draw(terminal)?;
                self.should_redraw = false;
            }

            // 2. 事件处理阶段
            match self.handler.next() {
                Result::Ok(app_event) => self.handle_events(app_event)?,
                Err(e) => {
                    tracing::error!("{:#?}", e)
                }
            }
        }
        Ok(())
    }

    fn draw(&self, terminal: &mut DefaultTerminal) -> Res<()> {
        terminal.draw(|f| f.render_widget(AppWrapper(&self), f.area()))?;
        return Ok(());
    }

    /// 处理任何AppEvent
    pub fn handle_events(&mut self, app_event: AppEvent) -> Res<()> {
        match app_event {
            AppEvent::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event.code)?;
            }
            AppEvent::Tick => {
                self.should_redraw = true;
            }
            AppEvent::FileChanged => {
                self.should_redraw = true;
            }
            _ => {
                todo!()
                // ignore
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_code: KeyCode) -> Res<()> {
        match key_code {
            event::KeyCode::Left => {
                // self.cursor.focus = self.cursor.focus.left();
                self.workspace
                    .move_item(self.cursor, self.cursor.move_left());
                self.should_redraw = true;
            }
            event::KeyCode::Right => {
                self.workspace
                    .move_item(self.cursor, self.cursor.move_right());
                self.should_redraw = true;
            }
            event::KeyCode::Up => {
                self.cursor = self.cursor.shift_up();
                self.should_redraw = true;
            }
            event::KeyCode::Down => {
                if self.cursor.index < self.workspace.get_list_mut(self.cursor.focus).len() - 1 {
                    self.cursor = self.cursor.shift_down();
                    self.should_redraw = true;
                }
            }
            event::KeyCode::Char('q') | event::KeyCode::Esc => self.exit = true,
            _ => {
                todo!()
            }
        }
        return Ok(());
    }
}
