use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, error, info, info_span, Instrument};
use ferrumc_net::connection::handle_connection;
use ferrumc_net::GlobalState;
use crate::systems::definition::System;
use crate::Result;

pub struct SchedulerSystem;

#[async_trait]
impl System for SchedulerSystem {
    async fn start(&self, state: GlobalState) {
        if let Err(e) = SchedulerSystem::initiate_loop(state).await {
            error!("Scheduler system failed with error: {:?}", e);
        }
    }

    async fn stop(&self, _state: GlobalState) {
        debug!("Stopping TCP listener system...");
    }

    fn name(&self) -> &'static str {
        "scheduler"
    }
}

impl SchedulerSystem {
    async fn initiate_loop(state: GlobalState) -> Result<()> {
         get_scheduler().run(Arc::clone(&state)).await
    }
}
