use std::fmt::Debug;

use crate::core::{
    cmd::Cmd,
    context::Context,
    model::models::Models,
    msg::Msg,
    services::{listener::Listener, tasks::TaskManager, watcher::Watcher},
    traits::Model,
};

#[derive(Debug, Default)]
struct EpochEnvelope<T> {
    pub epoch: u32,
    pub payload: T,
}

#[derive(Default)]
struct EpochModelManager {
    curr_model: Option<Box<dyn Model>>,
    curr_epoch: u32,
}

impl EpochModelManager {
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
            Some(model) if msg.epoch == self.curr_epoch => model.update(msg.payload, ctx),
            _ => Cmd::None,
        };

        EpochEnvelope {
            payload,
            epoch: self.curr_epoch,
        }
    }
}

impl Debug for EpochModelManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug)]
pub struct Runner {
    model_manager: EpochModelManager,
    listener: Listener,
    watcher: Watcher,
    task_manager: TaskManager,

    context: Context,
}

impl Runner {}
