use std::sync::{Arc, atomic::AtomicU64};

use dashmap::DashMap;

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Processing(f32), // 进度百分比
    Completed,
    Failed(String),
}

#[derive(Debug)]
pub struct TaskManager {
    pub semaphore: Arc<tokio::sync::Semaphore>,
    pub registry: Arc<DashMap<u64, TaskStatus>>,
    pub next_id: AtomicU64,
    pub status_tx: tokio::sync::mpsc::Sender<(u64, TaskStatus)>,
}

impl TaskManager {
    pub fn new(max_concurrency: usize) -> (Self, tokio::sync::mpsc::Receiver<(u64, TaskStatus)>) {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        let manager = Self {
            registry: Arc::new(DashMap::new()),
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrency)),
            next_id: AtomicU64::new(1),
            status_tx: tx,
        };
        (manager, rx)
    }

    // 提交任务的方法
    pub async fn submit<F>(&self, task_fn: F) -> u64
    where
        F: FnOnce() -> Result<(), String> + Send + 'static,
    {
        // 生成唯一 ID
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // 初始化状态
        self.registry.insert(id, TaskStatus::Pending);

        let sem_clone = Arc::clone(&self.semaphore);
        let registry_clone = Arc::clone(&self.registry);
        let tx_clone = self.status_tx.clone();

        // 开启异步包装器
        tokio::spawn(async move {
            // 等待并发许可
            let _permit = sem_clone.acquire().await.unwrap();

            // 更新状态为处理中
            registry_clone.insert(id, TaskStatus::Processing(0.0));
            let _ = tx_clone.send((id, TaskStatus::Processing(0.0))).await;

            // 在阻塞线程池中执行重型任务
            let result = tokio::task::spawn_blocking(task_fn).await;

            // 处理最终结果
            let final_status = match result {
                Ok(Ok(_)) => TaskStatus::Completed,
                Ok(Err(e)) => TaskStatus::Failed(e),
                Err(e) => TaskStatus::Failed(format!("Runtime Error: {}", e)),
            };

            registry_clone.insert(id, final_status.clone());
            let _ = tx_clone.send((id, final_status)).await;
        });

        id
    }

    /// 清理已完成或失败的任务状态，防止内存泄漏
    pub fn prune_finished(&self) {
        self.registry.retain(|_, status| {
            matches!(status, TaskStatus::Pending | TaskStatus::Processing(_))
        });
    }

    pub fn get_status(&self, id: u64) -> Option<TaskStatus> {
        self.registry.get(&id).map(|r| r.value().clone())
    }
}
