use std::path::PathBuf;

use crate::core::{
    msg::Msg,
    service::{listener::Listener, tasks::TaskManager, ticker::Ticker, watcher::Watcher},
};
use tokio::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug)]
pub struct Servicer {
    listener: Listener,
    watcher: Option<Watcher>,
    ticker: Ticker,
    pub task_manager: TaskManager,
    rx: Receiver<Msg>,
    /// 内部持有 Sender，用于按需克隆给新启动的服务
    msg_tx: Sender<Msg>,
}

impl Servicer {
    pub fn new(tick_rate: f64, task_permits: usize) -> Self {
        let (tx, rx) = channel(1024);
        Self {
            listener: Listener::new(tx.clone()),
            watcher: None,
            ticker: Ticker::new(tx.clone(), tick_rate),
            task_manager: TaskManager::new(tx.clone(), task_permits),
            rx,
            msg_tx: tx,
        }
    }

    pub fn set_watcher(&mut self, watch_path: PathBuf) {
        self.watcher = Watcher::new(self.msg_tx.clone(), watch_path).into();
    }

    pub fn try_recv(&mut self) -> Result<Msg, tokio::sync::mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }

    /// 异步接收消息，直到有消息到达或通道关闭
    pub async fn recv(&mut self) -> Option<Msg> {
        self.rx.recv().await
    }
}
