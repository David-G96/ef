use std::{env::current_dir, fmt::Debug, sync::Arc};

use crate::core::{
    cmd::Cmd,
    context::Context,
    model::{processor::Processor, selector::SelectModel},
    msg::Msg,
    service::servicer::Servicer,
};

use crate::core::file_ops;
use color_eyre::Result as Res;
use ratatui::{DefaultTerminal, layout::Rect};

use crate::core::model::Model;

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
}

#[derive(Default)]
struct EpochGuard {
    curr_model: Option<Box<dyn Model<Context = Context, Cmd = Cmd, Msg = Msg>>>,
    curr_epoch: u32,
}

impl EpochGuard {
    pub fn change_model(&mut self, model: Box<dyn Model<Context = Context, Cmd = Cmd, Msg = Msg>>) {
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

impl Debug for EpochGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.curr_epoch)
    }
}

#[derive(Debug)]
pub struct Runner {
    guard_model: EpochGuard,
    servicer: Servicer,
    context: Context,
    should_exit: bool,
    dry_run: bool,
}

impl Runner {
    pub fn new(config: crate::core::config::Config) -> Self {
        let tick_rate = config.tick_rate;
        Self {
            dry_run: true,
            context: Context { config },
            servicer: Servicer::new(tick_rate, 8),
            guard_model: Default::default(),
            should_exit: false,
        }
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    fn submit_task<F>(&mut self, task_fn: F, id: u64)
    where
        F: FnOnce(Arc<dyn Fn(f32) + Send + Sync>) -> Result<(), String> + Send + 'static,
    {
        let epoch = self.guard_model.curr_epoch;
        self.servicer.task_manager.submit(id, epoch, task_fn);
    }

    pub async fn run(&mut self, term: &mut DefaultTerminal) -> Res<()> {
        let init_path = self
            .context
            .config
            .default_path
            .clone()
            .map_or_else(current_dir, Ok)?;

        // init model
        self.guard_model.change_model(Box::new(SelectModel::new(
            init_path.clone(),
            self.context.config.show_hidden,
            self.context.config.respect_gitignore,
        )?));

        // init service
        self.servicer.set_watcher(init_path);

        // 1. 初始渲染：确保程序启动时用户能看到界面
        self.draw(term)?;

        // 2. 阻塞式事件循环：只有收到消息（信号）时才继续执行
        while let Some(msg) = self.servicer.recv().await {
            let mut should_redraw = true;
            self.handle_msg(msg);

            // 3. 排空当前队列中所有积压的消息，避免连续多次重绘
            while let Result::Ok(msg) = self.servicer.try_recv() {
                _ = self.handle_msg(msg);
                should_redraw = true;
            }

            if self.should_exit {
                break;
            }

            // 4. 只有在处理完所有当前消息且确实需要重绘时才执行
            if should_redraw {
                self.draw(term)?;
            }
        }

        ratatui::restore();

        Ok(())
    }

    /// 理论上来说应该按需重绘，但是无所谓了
    fn handle_msg(&mut self, msg: Msg) -> bool {
        let envelope = self
            .guard_model
            .update(EpochEnvelope::new(msg), &self.context);
        self.handle_cmd(envelope);
        true
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
                self.guard_model.change_model(Box::new(Processor::new(m)));
            }
            Cmd::Organize(items, target_path) => {
                tracing::info!("organize:{:?}->{:?}", &items, &target_path);
                if !self.dry_run {
                    if let Err(e) = file_ops::organize(&items, &target_path) {
                        tracing::error!("Organize failed: {:?}", e);
                    }
                }
            }
            Cmd::Copy(items, target_path) => {
                tracing::info!("copy:{:?}->{:?}", &items, &target_path);
                if !self.dry_run {
                    if let Err(e) = file_ops::copy(&items, target_path, None) {
                        tracing::error!("Copy failed: {:?}", e);
                    }
                }
            }
            Cmd::Move(items, target_path) => {
                tracing::info!("move:{:?}->{:?}", &items, &target_path);
                if !self.dry_run {
                    if let Err(e) = file_ops::organize(&items, &target_path) {
                        tracing::error!("Move failed: {:?}", e);
                    }
                }
            }
            Cmd::Delete(items) => {
                tracing::info!("delete:{:?}", &items);
                if !self.dry_run {
                    if let Err(e) = crate::core::file_ops::delete(&items) {
                        tracing::error!("Delete failed: {:?}", e);
                    }
                }
            }
            Cmd::Trash(items) => {
                tracing::info!("trash:{:?}", &items);
                if !self.dry_run {
                    if let Err(e) = crate::core::file_ops::trash(&items) {
                        tracing::error!("Trash failed: {:?}", e);
                    }
                }
            }
            Cmd::AsyncDelete(id, items) => {
                tracing::info!("async delete:{:?}", &items);
                if !self.dry_run {
                    self.submit_task(
                        move |_| crate::core::file_ops::delete(&items).map_err(|e| e.to_string()),
                        id,
                    );
                }
            }
            Cmd::AsyncOrganize(id, items, target_path) => {
                tracing::info!("async organize:{:?}->{:?}", &items, &target_path);
                if !self.dry_run {
                    self.submit_task(
                        move |_| file_ops::organize(&items, &target_path).map_err(|e| e.to_string()),
                        id,
                    );
                }
            }
            Cmd::AsyncCopy(id, items, target_path) => {
                tracing::info!("async copy:{:?}->{:?}", &items, &target_path);
                if !self.dry_run {
                    self.submit_task(
                        move |reporter| file_ops::copy(&items, target_path, Some(reporter)).map_err(|e| e.to_string()),
                        id,
                    );
                }
            }
            Cmd::AsyncMove(id, items, target_path) => {
                tracing::info!("async move:{:?}->{:?}", &items, &target_path);
                if !self.dry_run {
                    self.submit_task(
                        move |_| file_ops::organize(&items, &target_path).map_err(|e| e.to_string()),
                        id,
                    );
                }
            }
            Cmd::AsyncTrash(id, items) => {
                tracing::info!("async trash:{:?}", &items);
                if !self.dry_run {
                    self.submit_task(
                        move |_| crate::core::file_ops::trash(&items).map_err(|e| e.to_string()),
                        id,
                    );
                }
            }
            Cmd::Seq(cmds) => {
                for cmd in cmds {
                    self.handle_cmd(EpochEnvelope::new(cmd))
                }
            }
            Cmd::Batch(cmds) => {
                cmds.into_iter().for_each(|cmd: Cmd| {
                    self.handle_cmd(EpochEnvelope::new(cmd));
                });
            }
            Cmd::ToggleShowHidden => {
                self.context.config.show_hidden = !self.context.config.show_hidden;
                tracing::info!("Toggle show_hidden: {}", self.context.config.show_hidden);
            }
            Cmd::ToggleRespectGitIgnore => {
                self.context.config.respect_gitignore = !self.context.config.respect_gitignore;
                tracing::info!(
                    "Toggle respect_gitignore: {}",
                    self.context.config.respect_gitignore
                );
            }
            Cmd::LoadDir(path) => {
                match file_ops::list_items(
                    &path,
                    self.context.config.show_hidden,
                    self.context.config.respect_gitignore,
                ) {
                    Ok(items) => {
                        let msg = Msg::DirLoaded(path, items);
                        let envelope = self
                            .guard_model
                            .update(EpochEnvelope::new(msg), &self.context);
                        self.handle_cmd(envelope);
                    }
                    Err(e) => tracing::error!("Failed to load dir: {:?}", e),
                }
            }

            _ => {}
        }
    }

    fn draw(&mut self, term: &mut DefaultTerminal) -> Res<()> {
        term.draw(|f| self.guard_model.render(f, f.area()))?;
        Ok(())
    }
}
