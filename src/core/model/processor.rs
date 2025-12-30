use crate::core::{
    cmd::Cmd,
    model::{
        AnyModel,
        component::{ScrollList, input::InputBox},
        selector::SelectModel,
    },
    msg::Msg,
    // traits::Model,
};
use color_eyre::Result as Res;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Layout, Position},
    macros::constraints,
    style::Stylize,
    text::Line,
    widgets::{Block, Widget as _},
};
use std::{fmt::Write, path::PathBuf};

#[derive(Debug, Default)]
pub enum InProcess {
    /// alias 'n'
    #[default]
    None,
    /// alias 'd'
    Delete,
    /// alias 't' 'tr'
    Trash,
    /// alias 'o' 'og'
    Organize(InputBox),
    /// alias 'm' 'mv'
    Move(InputBox),
    /// alias 'c' 'cp'
    Copy(InputBox),
    /// not yet impl
    Zip(),
    Rename,
}

impl InProcess {
    pub fn len(&self) -> usize {
        let mut counter = ByteCounter::new();
        write!(&mut counter, "{}", self).expect("this is will be triggered");
        counter.count
    }

    pub fn try_enter(&mut self, ch: char) {
        use InProcess::*;
        match self {
            Organize(x) => x.enter_char(ch),
            Move(x) => x.enter_char(ch),
            Copy(x) => x.enter_char(ch),
            _ => {}
        }
    }

    pub fn try_delete(&mut self) {
        use InProcess::*;
        match self {
            Organize(x) => x.delete_char(),
            Move(x) => x.delete_char(),
            Copy(x) => x.delete_char(),
            _ => {}
        }
    }
}

impl std::fmt::Display for InProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InProcess::*;
        match self {
            None => write!(f, ""),
            Delete => write!(f, "Delete"),
            Trash => write!(f, "Trash"),
            Organize(input) => write!(f, "Organize: {}", input),
            Move(input) => write!(f, "Move: {}", input),
            Copy(input) => write!(f, "Copy: {}", input),
            Zip() => todo!(),
            x => write!(f, "{:?}", x),
        }
    }
}

/// 一个简单的计数器，用于计算格式化后的字节长度而不分配内存
struct ByteCounter {
    count: usize,
}

impl ByteCounter {
    fn new() -> Self {
        ByteCounter { count: 0 }
    }
}

/// 为 ByteCounter 实现 Write trait
impl std::fmt::Write for ByteCounter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        // 累加传入字符串切片的字节长度
        self.count += s.len();
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Processor {
    mid: ScrollList,
    // mid_process : InProcess,
    left: ScrollList,
    left_proc: InProcess,
    right: ScrollList,
    right_proc: InProcess,

    /// default false, which is left
    focus_right: bool,

    /// 用来指示列表切换是否被锁住了，当被锁住时，说明正在输入，应该使用input box逻辑
    is_editing: bool,
}

impl Processor {
    pub fn new(inner: SelectModel) -> Self {
        Self {
            left: inner.left.clone(),
            right: inner.right.clone(),
            focus_right: false,
            ..Default::default()
        }
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> Res<Cmd> {
        match key_event.code {
            KeyCode::Left | KeyCode::Right => {
                self.focus_right = !self.focus_right;
                self.is_editing = false;
            }
            KeyCode::Char(ch) => {
                if self.is_editing {
                    self.curr_proc_mut().try_enter(ch);
                } else {
                    match ch {
                        'o' => {
                            self.is_editing = true;
                            *self.curr_proc_mut() = InProcess::Organize(Default::default());
                        }
                        'd' => {
                            self.is_editing = false;
                            *self.curr_proc_mut() = InProcess::Delete;
                        }
                        'c' => {
                            self.is_editing = true;
                            *self.curr_proc_mut() = InProcess::Copy(Default::default());
                        }
                        'm' => {
                            self.is_editing = true;
                            *self.curr_proc_mut() = InProcess::Move(Default::default());
                        }
                        _ => {
                            self.is_editing = false;
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                if self.is_editing {
                    self.curr_proc_mut().try_delete();
                }
            }
            // This is actually Fn+Del on macOS!
            KeyCode::Delete => {
                todo!()
            }
            KeyCode::Enter => {
                return Ok(Cmd::Batch(vec![
                    self.proc_into_cmd(&self.left_proc, &self.left),
                    self.proc_into_cmd(&self.right_proc, &self.right),
                ]));
            }
            KeyCode::Esc => return Ok(Cmd::Exit),
            _ => {}
        }
        Ok(Cmd::None)
    }

    fn proc_into_cmd(&self, proc: &InProcess, list: &ScrollList) -> Cmd {
        match proc {
            InProcess::None => Cmd::None,
            InProcess::Delete => Cmd::Delete(list.items.iter().map(|f| f.path.clone()).collect()),
            InProcess::Organize(to) => Cmd::Organize(
                list.items.iter().map(|i| i.path.clone()).collect(),
                PathBuf::from(to.input()),
            ),
            _ => Cmd::None,
        }
    }

    fn curr_proc_mut(&mut self) -> &mut InProcess {
        if self.focus_right {
            &mut self.right_proc
        } else {
            &mut self.left_proc
        }
    }

    fn curr_proc(&self) -> &InProcess {
        if self.focus_right {
            &self.right_proc
        } else {
            &self.left_proc
        }
    }
    fn render_list_panel(
        &self,
        list: &ScrollList,
        proc: &InProcess,
        is_focused: bool,
        side_name: &str,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let title = if is_focused {
            if self.is_editing {
                if self.curr_proc().len() == 0 {
                    format!("{} - Processing", side_name)
                } else {
                    format!("{} - {} ", side_name, proc)
                }
            } else {
                format!("{} - *{}", side_name, proc)
            }
        } else {
            format!("{} - {}", side_name, proc)
        };
        list.render_with_border(is_focused, None, &title)
            .render(area, buf);
    }
}

// impl Model for Processor {
//     fn update(
//         &mut self,
//         msg: crate::core::msg::Msg,
//         _: &crate::core::context::Context,
//     ) -> crate::core::cmd::Cmd {
//         match msg {
//             Msg::Exit => Cmd::Exit,
//             Msg::Key(key_event) => self
//                 .handle_key_event(&key_event)
//                 .unwrap_or_else(|e| Cmd::Error(e.to_string())),

//             _ => Cmd::None,
//         }
//     }

//     fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
//         let instructions = Line::from(vec![
//             " Delete ".into(),
//             "<D>".blue().bold(),
//             " Organize ".into(),
//             "<O>".blue().bold(),
//             " Quit ".into(),
//             "<Q> ".blue().bold(),
//         ]);

//         let block = Block::bordered()
//             // .title(Line::from("Processing...").centered())
//             .title_bottom(instructions.centered())
//             // .border_set(border::THICK)
//             ;

//         let inner_area = block.inner(area);
//         block.render(area, buf);

//         let columns = Layout::horizontal(constraints![==50%, ==50%]);
//         let [left_area, right_area] = columns.areas(inner_area);

//         self.render_list_panel(
//             &self.left,
//             &self.left_proc,
//             !self.focus_right,
//             "Left",
//             left_area,
//             buf,
//         );
//         self.render_list_panel(
//             &self.right,
//             &self.right_proc,
//             self.focus_right,
//             "Right",
//             right_area,
//             buf,
//         );
//     }
// }

impl AnyModel for Processor {
    type Cmd = crate::core::cmd::Cmd;
    type Msg = crate::core::msg::Msg;
    type Context = crate::core::context::Context;

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
    ) -> color_eyre::Result<()> {
        let buf = frame.buffer_mut();
        let instructions = Line::from(vec![
            " Delete ".into(),
            "<D>".blue().bold(),
            " Organize ".into(),
            "<O>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            // .title(Line::from("Processing...").centered())
            .title_bottom(instructions.centered())
            // .border_set(border::THICK)
            ;

        let inner_area = block.inner(area);
        block.render(area, buf);

        let columns = Layout::horizontal(constraints![==50%, ==50%]);
        let [left_area, right_area] = columns.areas(inner_area);

        self.render_list_panel(
            &self.left,
            &self.left_proc,
            !self.focus_right,
            "Left",
            left_area,
            buf,
        );
        self.render_list_panel(
            &self.right,
            &self.right_proc,
            self.focus_right,
            "Right",
            right_area,
            buf,
        );

        if self.is_editing {
            // right list
            if self.focus_right {
                frame.set_cursor_position(Position::new(
                    right_area.x + "right - ".len() as u16 + self.right_proc.len() as u16 + 1,
                    1,
                ));
            }
            // left list
            else {
                frame.set_cursor_position(Position::new(
                    left_area.x + "left - ".len() as u16 + self.left_proc.len() as u16 + 1,
                    1,
                ));
            }
        }

        Ok(())
    }

    fn update(&mut self, msg: &Self::Msg, _: &Self::Context) -> Self::Cmd {
        match msg {
            Msg::Exit => Cmd::Exit,
            Msg::Key(key_event) => self
                .handle_key_event(key_event)
                .unwrap_or_else(|e| Cmd::Error(e.to_string())),

            _ => Cmd::None,
        }
    }
}
