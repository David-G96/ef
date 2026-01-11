use crate::core::{
    msg::Msg,
    service::{listener::Listener, tasks::TaskManager, ticker::Ticker, watcher::Watcher},
};
use tokio::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug)]
pub struct Servicer {
    listener: Option<Listener>,
    watcher: Option<Watcher>,
    ticker: Option<Ticker>,
    task_manager: Option<TaskManager>,
    rx: Receiver<Msg>,
    /// 内部持有 Sender，用于按需克隆给新启动的服务
    msg_tx: Sender<Msg>,
}

impl Servicer {
    pub fn new() -> Self {
        let (tx, rx) = channel(1024);
        Self {
            listener: None,
            watcher: None,
            ticker: None,
            task_manager: None,
            rx,
            msg_tx: tx,
        }
    }

    /// 启用按键监听服务
    pub fn with_listener(mut self) -> Self {
        self.listener = Some(Listener::new(self.msg_tx.clone()));
        self
    }


    /// 启用文件观察服务
    pub fn with_watcher(mut self, path: std::path::PathBuf) -> Self {
        self.watcher = Some(Watcher::new(self.msg_tx.clone(), path));
        self
    }

    /// 启用定时器服务
    pub fn with_ticker(mut self, tick_rate: f64) -> Self {
        self.ticker = Some(Ticker::new(self.msg_tx.clone(), tick_rate));
        self
    }

    pub fn with_task_manager(mut self, permits: usize) -> Self {
        self.task_manager = Some(TaskManager::new(self.msg_tx.clone(), permits));
        self
    }

    pub fn try_recv(&mut self) -> Result<Msg, tokio::sync::mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }

    /// 异步接收消息，直到有消息到达或通道关闭
    pub async fn recv(&mut self) -> Option<Msg> {
        self.rx.recv().await
    }
}

impl Default for Servicer {
    fn default() -> Self {
        Self::new()
    }
}
