use std::{env, fmt::Debug};

use crate::core::{
    cmd::Cmd,
    context::Context,
    model::{processor::Processor, selector::SelectModel},
    msg::Msg,
    service::servicer::Servicer,
    // traits::{AnyModel, Model},
};

use color_eyre::{Result as Res, eyre::Ok};
use ratatui::{DefaultTerminal, layout::Rect};

use crate::core::model::AnyModel;

#[derive(Debug, Default, Clone, Copy)]
struct EpochEnvelope<T> {
    /// if None, then ignore epoch and send anyway
    pub epoch: Option<u32>,
    pub payload: T,
}

impl<T> EpochEnvelope<T> {
    pub fn new(payload: T) -> Self {
        Self {
            epoch: None,
            payload,
        }
    }

    pub fn new_with_epoch(payload: T, epoch: u32) -> Self {
        Self {
            epoch: Some(epoch),
            payload,
        }
    }
}

#[derive(Default)]
struct EpochModel {
    curr_model: Option<Box<dyn AnyModel<Context = Context, Cmd = Cmd, Msg = Msg>>>,
    curr_epoch: u32,
}

impl EpochModel {
    pub fn new(model: Box<dyn AnyModel<Context = Context, Cmd = Cmd, Msg = Msg>>) -> Self {
        Self {
            curr_model: Some(model),
            curr_epoch: 0,
        }
    }

    pub fn change_model(
        &mut self,
        model: Box<dyn AnyModel<Context = Context, Cmd = Cmd, Msg = Msg>>,
    ) {
        self.curr_model = Some(model);
        self.curr_epoch = self.curr_epoch.overflowing_add(1).0;
    }

    pub fn update(&mut self, msg: EpochEnvelope<Msg>, ctx: &Context) -> EpochEnvelope<Cmd> {
        let payload = match &mut self.curr_model {
            Some(model)
                if msg
                    .epoch
                    .map(|epoch| epoch == self.curr_epoch)
                    .unwrap_or(true) =>
            {
                model.update(&msg.payload, ctx)
            }
            _ => Cmd::None,
        };

        EpochEnvelope {
            payload,
            epoch: Some(self.curr_epoch),
        }
    }

    pub fn render(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        if let Some(model) = &mut self.curr_model {
            _ = model.draw(frame, area)
        }
    }
}

impl Debug for EpochModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug, Default)]
pub struct Runner {
    model_manager: EpochModel,
    servicer: Servicer,
    context: Context,
    should_exit: bool,
    dry_run: bool,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            dry_run: true,
            ..Default::default()
        }
    }
    
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub async fn run(&mut self, term: &mut DefaultTerminal) -> Res<()> {
        self.model_manager
            .change_model(Box::new(SelectModel::new()?));
        self.servicer = Servicer::new()
            .with_listener()
            .with_watcher(env::current_dir()?)
            .with_ticker(4.0)
            .with_task_manager(8);

        // 1. 初始渲染：确保程序启动时用户能看到界面
        self.draw(term)?;

        // 2. 阻塞式事件循环：只有收到消息（信号）时才继续执行
        while let Some(msg) = self.servicer.recv().await {
            let mut should_redraw = self.handle_msg(msg);

            // 3. 性能优化：排空当前队列中所有积压的消息，避免连续多次重绘
            while let Result::Ok(msg) = self.servicer.try_recv() {
                if self.handle_msg(msg) {
                    should_redraw = true;
                }
            }

            if self.should_exit {
                break;
            }

            // 4. 只有在处理完所有当前消息且确实需要重绘时才执行
            if should_redraw {
                self.draw(term)?;
            }
        }

        Ok(())
    }

    /// redraw logic inside
    fn handle_msg(&mut self, msg: Msg) -> bool {
        // 基础逻辑：Tick 默认不触发重绘（除非你有动画需求），其他事件触发重绘
        let should_redraw = match msg {
            Msg::Tick => true,
            _ => true,
        };

        let envelope = self
            .model_manager
            .update(EpochEnvelope::new(msg), &self.context);
        self.handle_cmd(envelope);

        should_redraw
    }

    fn handle_cmd(&mut self, envelope: EpochEnvelope<Cmd>) {
        match envelope.payload {
            Cmd::Exit => {
                self.should_exit = true;
            }
            Cmd::Error(e) => {
                tracing::error!("{:?}", e);
            }
            Cmd::IntoProcess(m) => {
                self.model_manager.change_model(Box::new(Processor::new(m)));
            }
            Cmd::Organize(items, target_path) => {
                tracing::info!("organize:{:?}->{:?}", &items, &target_path);
                if !self.dry_run
                    && let Err(e) = crate::core::file_ops::organize(&items, &target_path) {
                        tracing::error!("Organize failed: {:?}", e);
                    }
            }
            Cmd::Copy(items, target_path) => {
                tracing::info!("copy:{:?}->{:?}", &items, &target_path);
                if !self.dry_run
                    && let Err(e) = crate::core::file_ops::copy(&items, &target_path) {
                        tracing::error!("Copy failed: {:?}", e);
                    }
            }
            Cmd::Move(items, target_path) => {
                tracing::info!("move:{:?}->{:?}", &items, &target_path);
                if !self.dry_run
                    && let Err(e) = crate::core::file_ops::organize(&items, &target_path) {
                        tracing::error!("Move failed: {:?}", e);
                    }
            }
            Cmd::Delete(items) => {
                tracing::info!("delete:{:?}", &items);
                if !self.dry_run
                    && let Err(e) = crate::core::file_ops::delete(&items) {
                        tracing::error!("Delete failed: {:?}", e);
                    }
            }
            Cmd::Trash(items) => {
                tracing::info!("trash:{:?}", &items);
                if !self.dry_run
                    && let Err(e) = crate::core::file_ops::trash(&items) {
                        tracing::error!("Trash failed: {:?}", e);
                    }
            }
            Cmd::Seq(cmds) => {
                for cmd in cmds {
                    self.handle_cmd(EpochEnvelope::new(cmd))
                }
            }
            _ => {}
        }
        // None
    }

    fn draw(&mut self, term: &mut DefaultTerminal) -> Res<()> {
        term.draw(|f| self.model_manager.render(f, f.area()))?;
        Ok(())
    }
}
