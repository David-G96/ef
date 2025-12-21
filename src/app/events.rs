use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, MouseEvent,
};
use notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};

#[derive(Debug)]
#[non_exhaustive]
pub enum AppEvent {
    // CrosstermEvent(CrosstermEvent),
    // Cross term Events
    FocusGained,
    FocusLost,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
    CrosstermEvent(crossterm::event::Event),

    Init,
    Quit,
    Error,
    Closed,
    Tick,
    // Render,
    Input(KeyCode),
    /// Now, call `read_dir()` !
    FileChanged,
    // Exit,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum FileCommand {
    ChangePath(PathBuf),
    // WatchRate(f64),
    // Stop,
}

#[derive(Debug)]
pub struct EventHandler {
    // 接收端保留在这里
    pub rx: Receiver<AppEvent>,
    pub tx_file_cmd: Sender<FileCommand>,
    // 我们可以保留 watcher 的句柄以防它被 drop，或者在这个 demo 中 move 进线程
    _watcher_handle: thread::JoinHandle<()>,
    _input_handle: thread::JoinHandle<()>,
    _tick_handle: thread::JoinHandle<()>,

    watch_path: PathBuf,
}

impl EventHandler {
    pub fn new(watch_path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        let (tx_file_cmd, rx_file_cmd) = mpsc::channel();

        // 启动 Tick 线程
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                loop {
                    _ = tx.send(AppEvent::Tick).ok(); // 忽略发送错误
                    thread::sleep(Duration::from_millis(10));
                }
            })
        };

        // 启动 输入 监听线程
        let input_handle = {
            let input_tx = tx.clone();
            thread::spawn(move || {
                loop {
                    // Changed Event to CrosstermEvent
                    if let Ok(CrosstermEvent::Key(key)) = event::read() {
                        // 这很重要，详见ratatui的FAQ，总之都是win的错
                        if key.kind == KeyEventKind::Press {
                            // Err时，管道关闭，也没有必要发送了
                            if input_tx.send(AppEvent::Input(key.code)).is_err() {
                                break;
                            }
                        }
                    }
                }
            })
        };

        // 启动 文件 监听线程
        let watcher_handle = {
            // 这里 clone 初始路径
            let initial_path = watch_path.clone();
            let tx = tx.clone();
            // 注意：rx_file_cmd 需要 move 进这个线程
            // let rx_file_cmd = rx_file_cmd;

            thread::spawn(move || {
                // 1. 创建 Debouncer
                let mut debouncer = new_debouncer(
                    Duration::from_millis(500),
                    move |res: DebounceEventResult| match res {
                        Ok(_events) => {
                            // 发送事件给主线程
                            let _ = tx.send(AppEvent::FileChanged);
                            tracing::info!(" [FileMgr] file changes debounced!")
                        }
                        Err(e) => tracing::error!("Watch error: {:?}", e),
                    },
                )
                .unwrap();

                // 2. 初始监听 (在 Loop 之外！)
                // 维护一个 current_path 变量，用来记住我们正在监听谁，方便以后 unwatch
                let mut current_path = initial_path;

                debouncer
                    .watcher()
                    .watch(&current_path, RecursiveMode::NonRecursive)
                    .expect("初始监听失败");

                tracing::info!(" [FileMgr] start monitoring: {:?}", current_path);

                // 3. 进入控制循环
                loop {
                    // 阻塞等待命令
                    match rx_file_cmd.recv() {
                        Ok(cmd) => match cmd {
                            FileCommand::ChangePath(new_path) => {
                                tracing::info!(
                                    " [FileMgr] 切换监听路径: {:?} -> {:?}",
                                    current_path,
                                    new_path
                                );

                                // A. 取消监听旧路径
                                //即使失败也不要 panic，可能是路径已经被删除了
                                let _ = debouncer.watcher().unwatch(&current_path);

                                // B. 监听新路径
                                if let Err(e) = debouncer
                                    .watcher()
                                    .watch(&new_path, RecursiveMode::NonRecursive)
                                {
                                    tracing::error!(" [FileMgr] 监听新路径失败: {:?}", e);
                                    // 如果新路径监听失败，你可以选择退回旧路径，或者保持空闲
                                } else {
                                    // C. 只有成功 watch 后，才更新 current_path
                                    current_path = new_path;
                                }
                            }
                        },
                        Err(_) => {
                            tracing::info!(" [FileMgr] 主线程断开，文件监听线程退出");
                            break;
                        }
                    }
                }
            })
        };

        Self {
            tx_file_cmd,
            rx,
            _watcher_handle: watcher_handle,
            _input_handle: input_handle,
            _tick_handle: tick_handle,
            watch_path,
        }
    }

    pub fn watch_path(
        &mut self,
        new_watch_path: PathBuf,
    ) -> Result<(), mpsc::SendError<FileCommand>> {
        self.tx_file_cmd
            .send(FileCommand::ChangePath(new_watch_path))
    }

    // 封装 recv，对外提供干净的接口
    pub fn next(&self) -> Result<AppEvent, std::sync::mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn next_timeout(&self) -> Result<AppEvent, mpsc::RecvTimeoutError> {
        self.rx.recv_timeout(Duration::from_millis(33))
    }
}
