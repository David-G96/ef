use std::path::PathBuf;

use crossterm::event::KeyEvent;

use crate::core::{
    cmd::Cmd,
    context::Context,
    model::{
        Model,
        component::{History, ListType, ScrollList, input::InputBox},
    },
    msg::Msg,
};

#[derive(Debug, Clone, Copy, Default)]
enum HomeMode {
    #[default]
    Sort,
    Proc,
    Preview,
    Confirm,
}

#[derive(Debug, Clone)]
enum HomeOperation {
    IntoProc,
}

/// Note: Waiting is no longer needed.
#[derive(Debug, Default, Clone)]
pub enum InProcess {
    #[default]
    /// alias 'n'
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
    //s not yet impl
    Zip,
    /// not yet impl
    Rename,
}

/// fusion of select and proc model
#[derive(Debug, Clone)]
pub struct HomeModel {
    path: PathBuf,

    left: ScrollList,
    left_proc: Option<InProcess>,
    mid: ScrollList,
    right: ScrollList,
    right_proc: Option<InProcess>,

    mode: HomeMode,

    focus: ListType,
    show_hidden: bool,
    respect_gitignore: bool,

    history: History<HomeOperation>,
}

impl HomeModel {
    pub fn new(path: impl Into<PathBuf>, show_hidden: bool, respect_gitignore: bool) -> Self {
        Self {
            path: path.into(),
            show_hidden,
            respect_gitignore,
            left: Default::default(),
            left_proc: None,
            mid: Default::default(),
            right: Default::default(),
            right_proc: None,
            mode: HomeMode::default(),
            focus: Default::default(),
            history: Default::default(),
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Cmd {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => match self.mode {
                HomeMode::Proc => self.mode = HomeMode::Sort,
                _ => {}
            },
            

            _ => {}
        }

        Cmd::None
    }


}

impl Model for HomeModel {
    type Cmd = Cmd;
    type Msg = Msg;
    type Context = Context;

    fn draw(
        &mut self,
        _frame: &mut ratatui::Frame,
        _area: ratatui::layout::Rect,
    ) -> color_eyre::Result<()> {
        todo!()
    }
    fn update(&mut self, msg: &Self::Msg, _ctx: &Self::Context) -> Self::Cmd {
        match msg {
            Msg::Exit => Cmd::Exit,
            Msg::Key(key_event) => self.handle_key_event(*key_event),
            _ => Cmd::None,
        }
    }
}
