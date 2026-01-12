use std::sync::{Arc, atomic::AtomicU64};

use dashmap::DashMap;
use tokio::sync::{Semaphore, mpsc::Sender};

use crate::core::msg::{Msg, TaskState, TaskStatus};

#[derive(Debug)]
pub struct TaskManager {
    pub semaphore: Arc<tokio::sync::Semaphore>,
    pub registry: Arc<DashMap<u64, TaskStatus>>,
    pub next_id: AtomicU64,
    pub status_tx: tokio::sync::mpsc::Sender<Msg>,
}

impl TaskManager {
    pub fn new(tx: Sender<Msg>, permits: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(permits)),
            registry: Arc::new(DashMap::new()),
            next_id: AtomicU64::new(0),
            status_tx: tx,
        }
    }

    // 提交任务的方法
    pub fn submit<F>(&self, id: u64, epoch: u32, task_fn: F)
    where
        F: FnOnce(Arc<dyn Fn(f32) + Send + Sync>) -> Result<(), String> + Send + 'static,
    {
        // 初始化状态
        self.registry.insert(id, TaskStatus::Pending);

        let sem_clone = Arc::clone(&self.semaphore);
        let registry_clone = Arc::clone(&self.registry);
        let tx_clone = self.status_tx.clone();

        // 创建进度汇报闭包
        let registry_progress = Arc::clone(&self.registry);
        let tx_progress = self.status_tx.clone();
        let reporter = Arc::new(move |progress: f32| {
            let status = TaskStatus::Processing(progress);
            registry_progress.insert(id, status.clone());
            // 在 blocking 线程中需使用 blocking_send
            let _ = tx_progress.blocking_send(Msg::TaskState(TaskState::new(id, epoch, status)));
        });

        // 开启异步包装器
        tokio::spawn(async move {
            // 等待并发许可
            let _permit = sem_clone.acquire().await.unwrap();

            // 更新状态为处理中
            registry_clone.insert(id, TaskStatus::Processing(0.0));
            let _ = tx_clone
                .send(Msg::TaskState(TaskState::new(
                    id,
                    epoch,
                    TaskStatus::Processing(0.0),
                )))
                .await;

            // 在阻塞线程池中执行重型任务
            let result = tokio::task::spawn_blocking(move || task_fn(reporter)).await;

            // 处理最终结果
            let final_status = match result {
                Ok(Ok(_)) => TaskStatus::Completed,
                Ok(Err(e)) => TaskStatus::Failed(e),
                Err(e) => TaskStatus::Failed(format!("Runtime Error: {}", e)),
            };

            registry_clone.insert(id, final_status.clone());
            let _ = tx_clone
                .send(Msg::TaskState(TaskState::new(id, epoch, final_status)))
                .await;
        });
    }

    /// 清理已完成或失败的任务状态，防止内存泄漏
    pub fn prune_finished(&self) {
        self.registry
            .retain(|_, status| matches!(status, TaskStatus::Pending | TaskStatus::Processing(_)));
    }

    pub fn get_status(&self, id: u64) -> Option<TaskStatus> {
        self.registry.get(&id).map(|r| r.value().clone())
    }
}
