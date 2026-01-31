use std::env::home_dir;
use std::{collections::VecDeque, path::PathBuf};

use crate::core::file_ops::{self};
use crate::core::{
    cmd::Cmd,
    model::component::{Cursor, FileItem, History, ListType, ScrollList},
    msg::Msg,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Constraint;
use ratatui::macros::constraints;
use ratatui::style::{self, Style};
use ratatui::{
    layout::Layout,
    style::Stylize,
    text::Line,
    widgets::{Block, List, Paragraph, StatefulWidget, Widget as _},
};

use color_eyre::Result as Res;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SelectOperation {
    // 移动操作：记录从哪来、到哪去，以及它在原列表中的原始位置（用于完美还原）
    Move {
        item_id: u64,
        from_list: ListType,
        from_index: usize, // 撤销时放回原位所需
        to_list: ListType,
    },
}

/// 分三列的工作区！（左右中）
#[derive(Debug, Default, Clone)]
pub struct SelectModel {
    pub(crate) path: PathBuf,
    pub(crate) all_items: VecDeque<FileItem>,
    pub(crate) mid: ScrollList,
    pub(crate) left: ScrollList,
    pub(crate) right: ScrollList,
    pub(crate) cursor: Cursor,
    pub(crate) history: History<SelectOperation>,
    pub(crate) show_hidden: bool,
    pub(crate) respect_gitignore: bool,
}

impl SelectModel {
    pub fn new(current_path: PathBuf, show_hidden: bool, respect_gitignore: bool) -> Res<Self> {
        let res = file_ops::list_items(&current_path, show_hidden, respect_gitignore)?;
        let mut model = Self::new_with(current_path, res);
        model.show_hidden = show_hidden;
        model.respect_gitignore = respect_gitignore;
        model.sync_view();
        Ok(model)
    }

    fn new_with(path: PathBuf, pending: VecDeque<FileItem>) -> Self {
        Self {
            path,
            all_items: pending.clone(),
            mid: ScrollList::new(pending),
            left: ScrollList::default(),
            right: ScrollList::default(),
            history: History::default(),
            cursor: Cursor::new(ListType::Mid),
            show_hidden: false,
            respect_gitignore: true,
        }
    }

    fn sync_view(&mut self) {
        let filtered: VecDeque<_> = self
            .all_items
            .iter()
            .filter(|item| self.show_hidden || !item.display_name.starts_with('.'))
            .cloned()
            .collect();
        self.mid = ScrollList::new(filtered);
    }

    fn get_list_mut(&mut self, list_type: ListType) -> &mut ScrollList {
        match list_type {
            ListType::Left => &mut self.left,
            ListType::Mid => &mut self.mid,
            ListType::Right => &mut self.right,
        }
    }

    fn get_list(&self, list_type: ListType) -> &ScrollList {
        match list_type {
            ListType::Left => &self.left,
            ListType::Mid => &self.mid,
            ListType::Right => &self.right,
        }
    }

    fn move_item(&mut self, from_focus: ListType, to_focus: ListType) -> Option<()> {
        let from_list = self.get_list_mut(from_focus);
        let from_index = from_list.state.selected()?;
        if from_list.items.is_empty() {
            return None;
        }

        let item = from_list.items.remove(from_index)?;
        let item_id = item.id;

        // 移除后修正原列表的选中项（防止越界）
        if from_index >= from_list.items.len() {
            from_list
                .state
                .select(Some(from_list.items.len().saturating_sub(1)));
        } else if from_list.items.is_empty() {
            from_list.state.select(None);
        }

        let cmd = SelectOperation::Move {
            item_id,
            from_list: from_focus,
            from_index,
            to_list: to_focus,
        };

        let updated_item = item;
        let to_list = self.get_list_mut(to_focus);
        to_list.items.push_front(updated_item);
        to_list.state.select(Some(0)); // 移动过去后选中新项

        self.history.log(cmd);

        Some(())
    }

    fn undo(&mut self) -> Option<()> {
        if let Some(cmd) = self.history.last() {
            match cmd.clone() {
                // Clone the command to own it
                SelectOperation::Move {
                    item_id,
                    from_list,
                    from_index,
                    to_list,
                    ..
                } => {
                    // 1. 从“去向列表”中移除该项
                    let target_list = &mut self.get_list_mut(to_list).items;
                    let pos = target_list.iter().position(|i| i.id == item_id)?;
                    let item = target_list.remove(pos)?;

                    // 2. 恢复其原始路径，因为在执行 Move 命令时，item 的 path 已经被更新为 new_path

                    // 3. 放回“来源列表”的原始位置
                    let source_list = &mut self.get_list_mut(from_list).items;
                    if from_index >= source_list.len() {
                        source_list.push_back(item);
                    } else {
                        source_list.insert(from_index, item);
                    }
                }
            }
        }

        self.history.undo();
        Some(())
    }

    fn as_lines(scroll_list: &ScrollList, is_focus: bool) -> Vec<Line<'static>> {
        scroll_list
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let mut line = item.as_line();
                if is_focus && scroll_list.state.selected() == Some(i) {
                    line = line.reversed();
                }
                line
            })
            .collect::<Vec<Line>>()
    }

    fn as_list(scroll_list: &ScrollList, is_focus: bool, title: &'static str) -> List<'static> {
        let lines = Self::as_lines(scroll_list, is_focus);
        let list = List::new(lines);
        let list_style = if is_focus {
            Style::default().fg(style::Color::Yellow).bold()
        } else {
            style::Style::default()
        };
        let block = Block::bordered()
            .title(Line::from(title).centered())
            .border_style(list_style);
        list.block(block)
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> Res<Cmd> {
        match key_event.code {
            KeyCode::Left => {
                self.move_item(self.cursor.focus, self.cursor.focus.left());
            }
            KeyCode::Right => {
                self.move_item(self.cursor.focus, self.cursor.focus.right());
            }
            KeyCode::Up => {
                self.get_list_mut(self.cursor.focus).up();
            }
            KeyCode::Down => {
                self.get_list_mut(self.cursor.focus).down();
            }
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Cmd::Exit),
            KeyCode::Char('z') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                _ = self.undo();
            }
            KeyCode::Enter => return Ok(Cmd::IntoProcess(self.clone())),
            KeyCode::Tab => {}
            KeyCode::Char('.') => {
                self.show_hidden = !self.show_hidden;
                self.sync_view();
                return Ok(Cmd::ToggleShowHidden);
            }
            KeyCode::Char('g') => {
                self.respect_gitignore = !self.respect_gitignore;
                return Ok(Cmd::Seq(vec![
                    Cmd::ToggleRespectGitIgnore,
                    Cmd::LoadDir(self.path.clone()),
                ]));
            }
            KeyCode::Char(':') => {
                
            }
            _ => {}
        }
        Ok(Cmd::None)
    }
}

impl crate::core::model::Model for SelectModel {
    type Cmd = crate::core::cmd::Cmd;
    type Msg = crate::core::msg::Msg;
    type Context = crate::core::context::Context;
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
    ) -> color_eyre::Result<()> {
        let buf = frame.buffer_mut();

        let [main_area, status_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        let path_display = if let Some(home) = home_dir() {
            if self.path == home {
                "~".to_string()
            } else if let Ok(stripped) = self.path.strip_prefix(&home) {
                format!("~/{}", stripped.display())
            } else {
                self.path.to_string_lossy().to_string()
            }
        } else {
            self.path.to_string_lossy().to_string()
        };

        let status_left = Line::from(vec![" ".into(), path_display.into()]);

        let status_right = Line::from(vec![
            format!(" {} items ", self.mid.items.len()).into(),
            "|".into(),
            " Hidden: ".into(),
            if self.show_hidden {
                "SHOW".green().bold()
            } else {
                "HIDE".red().bold()
            },
            " Git: ".into(),
            if self.respect_gitignore {
                "ON".green().bold()
            } else {
                "OFF".red().bold()
            },
            " ".into(),
        ]);

        let status_style = Style::default().bg(ratatui::style::Color::DarkGray);
        Block::default()
            .style(status_style)
            .render(status_area, buf);

        let [left_status, right_status] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(status_area);

        Paragraph::new(status_left)
            .style(status_style)
            .render(left_status, buf);
        Paragraph::new(status_right)
            .right_aligned()
            .style(status_style)
            .render(right_status, buf);

        let columns = Layout::horizontal(constraints![==33%, == 34%, ==33%]);
        let [left_area, mid_area, right_area] = columns.areas(main_area);

        // left
        StatefulWidget::render(
            Self::as_list(&self.left, self.cursor.focus == ListType::Left, "Left"),
            left_area,
            buf,
            &mut self.left.state,
        );
        // mid - pending
        StatefulWidget::render(
            Self::as_list(&self.mid, self.cursor.focus == ListType::Mid, "Pending"),
            mid_area,
            buf,
            &mut self.mid.state,
        );
        // right
        StatefulWidget::render(
            Self::as_list(&self.right, self.cursor.focus == ListType::Right, "Right"),
            right_area,
            buf,
            &mut self.right.state,
        );

        Ok(())
    }
    fn update(&mut self, msg: &Self::Msg, _ctx: &Self::Context) -> Self::Cmd {
        match msg {
            Msg::Exit => Cmd::Exit,
            Msg::Key(ket_event) => self
                .handle_key_event(ket_event)
                .unwrap_or_else(|e| Cmd::Error(e.to_string())),
            Msg::DirLoaded(path, items) => {
                if *path == self.path {
                    // 去重逻辑：剔除已经在 Left 或 Right 列表中的文件
                    let mut new_items = items.clone();
                    let pending_names: std::collections::HashSet<_> = self
                        .left
                        .items
                        .iter()
                        .chain(self.right.items.iter())
                        .map(|i| i.display_name.clone())
                        .collect();
                    new_items.retain(|i| !pending_names.contains(&i.display_name));
                    self.all_items = new_items;
                    self.sync_view();
                }
                Cmd::None
            }
            _ => Cmd::None,
        }
    }
}
