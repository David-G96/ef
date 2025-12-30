use std::{path::PathBuf, time::Duration};

use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use tokio::{
    sync::mpsc::{Sender, channel},
    task::JoinHandle,
};

use crate::core::msg::Msg;

#[derive(Debug)]
pub enum WatchCommand {
    ChangeWatchPath(PathBuf),
}

#[derive(Debug)]
pub struct Watcher {
    task: JoinHandle<()>,
    cmd_tx: Sender<WatchCommand>,
}

impl Watcher {
    pub fn new(tx: Sender<Msg>, path: PathBuf) -> Self {
        let (cmd_tx, mut cmd_rx) = channel(1024);

        let task = tokio::task::spawn_blocking(move || {
            let mut debouncer = new_debouncer(
                Duration::from_millis(500),
                move |res: DebounceEventResult| match res {
                    Ok(_events) => {
                        // 修复：在同步回调中使用 blocking_send 确保消息发出
                        let _ = tx.blocking_send(Msg::FileChanged);
                        tracing::info!("[Watcher] file changed debounced!")
                    }
                    Err(e) => {
                        tracing::error!("watch error: {:?}", e)
                    }
                },
            )
            .unwrap();

            let mut current_path = path;

            debouncer
                .watcher()
                .watch(&current_path, notify::RecursiveMode::NonRecursive)
                .expect("[Watcher] init watch failed!");

            tracing::info!("[Watcher] start watching: {:?}", current_path);

            loop {
                match cmd_rx.blocking_recv() {
                    Some(WatchCommand::ChangeWatchPath(new_path)) => {
                        tracing::info!("[Watcher] changing path to: {:?}", new_path);

                        let _ = debouncer.watcher().unwatch(&current_path);

                        if let Err(e) = debouncer
                            .watcher()
                            .watch(&new_path, notify::RecursiveMode::NonRecursive)
                        {
                            tracing::error!(
                                "[Watcher] failed to change path from `{:?}` to `{:?}` due to {:?}",
                                &current_path,
                                &new_path,
                                e
                            )
                        } else {
                            current_path = new_path;
                        }
                    }
                    None => break, // Channel 已关闭，退出循环
                }
            }
        });

        Self { task, cmd_tx }
    }
}
