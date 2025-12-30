use std::time::Duration;

use tokio::sync::mpsc::{Sender, channel};
use tokio::{task::JoinHandle, time};

use crate::core::msg::Msg;

#[derive(Debug, Clone, Copy)]
pub enum TickerCommand {
    ChangeTickRate(f64),
}

#[derive(Debug)]
pub struct Ticker {
    task: JoinHandle<()>,
    cmd_tx: Sender<TickerCommand>,
}

impl Ticker {
    pub fn new(tx: Sender<Msg>, tick_rate: f64) -> Self {
        let (cmd_tx, mut cmd_rx) = channel(1204);

        let task = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs_f64(1.0 / tick_rate));
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if tx.send(Msg::Tick).await.is_err() {
                            break;
                        }
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            TickerCommand::ChangeTickRate(new_rate) => {
                                interval = time::interval(Duration::from_secs_f64(1.0 / new_rate));
                                interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
                            }
                        }
                    }
                }
            }
        });
        Self { task, cmd_tx }
    }

    pub fn stop(&self) {
        self.task.abort();
    }

    pub fn change_tick_rate(&self, new_rate: f64) {
        let _ = self.cmd_tx.send(TickerCommand::ChangeTickRate(new_rate));
    }
}
