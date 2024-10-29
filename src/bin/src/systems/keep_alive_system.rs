use std::sync::Arc;
//use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use async_trait::async_trait;
use tracing::info;
use ferrumc_core::identity::player_identity::PlayerIdentity;
use ferrumc_net::GlobalState;
use crate::systems::definition::System;
use tokio::sync::Notify;

pub struct KeepAliveSystem {
    shutdown: Notify
}

impl KeepAliveSystem {
    pub fn new() -> Self {
        Self {
            shutdown: Notify::new()
        }
    }
}

#[async_trait]
impl System for KeepAliveSystem {
    async fn start(self: Arc<Self>, state: GlobalState) {
        tokio::select! {
            _ = async move {
                loop {
                    let online_players = state.universe.query::<&PlayerIdentity>();
                    info!("Online players: {}", online_players.count());

                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            } => {},
            _ = self.shutdown.notified() => {}
        }
    }

    async fn stop(self: Arc<Self>, _state: GlobalState) {
        tracing::debug!("Stopping keep alive system...");
        self.shutdown.notify_one();
    }

    fn name(&self) -> &'static str {
        "keep_alive"
    }
}

