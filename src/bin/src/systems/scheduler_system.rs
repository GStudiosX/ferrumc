use std::sync::Arc;
use async_trait::async_trait;
use tracing::debug;
use ferrumc::get_scheduler;
use ferrumc_net::GlobalState;
use crate::systems::definition::System;

pub struct SchedulerSystem;

#[async_trait]
impl System for SchedulerSystem {
    async fn start(&self, state: GlobalState) {
         //get_scheduler().run(Arc::clone(&state)).await;
    }

    async fn stop(&self, _state: GlobalState) {
        debug!("Stopping Scheduler system...");
    }

    fn name(&self) -> &'static str {
        "scheduler"
    }
}
