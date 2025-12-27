use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEventKind};
use tokio::{
    sync::mpsc::{Sender, channel},
    task::JoinHandle,
};

use crate::core::msg::Msg;

#[derive(Debug)]
#[non_exhaustive]
pub enum ListenCommand {}

/// listen to user input
#[derive(Debug)]
pub struct Listener {
    task: JoinHandle<()>,
    cmd_tx: Sender<ListenCommand>,
}

impl Listener {
    pub fn new(tx: Sender<Msg>) -> Self {
        let (cmd_tx, mut cmd_rx) = channel(1024);

        // 使用 spawn_blocking 处理阻塞的 IO 操作
        let task = tokio::task::spawn_blocking(move || {
            loop {
                // 1. 优先检查控制命令 (非阻塞)
                match cmd_rx.try_recv() {
                    Ok(_cmd) => {} // TODO: 处理命令，例如退出循环
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
                }

                // 2. 使用 poll 检查是否有输入事件 (带超时)
                // 这样可以定期醒来检查 cmd_rx，而不是一直阻塞在 read()
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if key.kind == KeyEventKind::Press
                                && tx.blocking_send(Msg::Key(key)).is_err() {
                                    break;
                                }
                        }
                        Ok(CrosstermEvent::Paste(paste_content)) => {
                            if tx.blocking_send(Msg::Paste(paste_content)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(w, h)) => {
                            if tx.blocking_send(Msg::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        Err(_) => break, // IO 错误（如终端关闭）时退出
                        _ => {}          // 忽略 Focus, Mouse 等未处理事件，防止 Panic
                    }
                }
            }
        });

        Self { task, cmd_tx }
    }
}
