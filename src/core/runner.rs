use std::{env, fmt::Debug};

use crate::core::{
    cmd::Cmd,
    context::Context,
    model::{processor::Processor, selector::SelectModel},
    msg::Msg,
    services::servicer::Servicer,
    traits::{AnyModel, Model},
};

use color_eyre::{Result as Res, eyre::Ok};
use ratatui::{DefaultTerminal, layout::Rect};

#[derive(Debug, Default)]
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
    curr_model: Option<Box<dyn Model>>,
    curr_epoch: u32,
}

impl EpochModel {
    pub fn new(model: Box<dyn Model>) -> Self {
        Self {
            curr_model: Some(model),
            curr_epoch: 0,
        }
    }

    pub fn change_model(&mut self, model: Box<dyn Model>) {
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
                model.update(msg.payload, ctx)
            }
            _ => Cmd::None,
        };

        EpochEnvelope {
            payload,
            epoch: Some(self.curr_epoch),
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if let Some(model) = &mut self.curr_model {
            model.render(area, buf);
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
}

impl Runner {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
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
            _ => {}
        }
        // None
    }

    fn draw(&mut self, term: &mut DefaultTerminal) -> Res<()> {
        term.draw(|f| self.model_manager.render(f.area(), f.buffer_mut()))?;
        Ok(())
    }
}
