use std::sync::Arc;
use std::sync::LazyLock;
use ferrumc_net::{GlobalState, NetResult};
use futures::stream::FuturesUnordered;
use tracing::{debug, debug_span, info, Instrument};
use async_trait::async_trait;
use crate::systems::keep_alive_system::KeepAliveSystem;
use crate::systems::tcp_listener_system::TcpListenerSystem;
use crate::systems::ticking_system::TickingSystem;
use crate::systems::scheduler_system::SchedulerSystem;
use futures::StreamExt;

#[async_trait]
pub trait System: Send + Sync {
    async fn start(self: Arc<Self>, state: GlobalState);
    async fn stop(self: Arc<Self>, state: GlobalState);

    fn name(&self) -> &'static str;
}

static SYSTEMS: LazyLock<Vec<Arc<dyn System>>> = LazyLock::new(|| vec![
    Arc::new(SchedulerSystem),
    Arc::new(KeepAliveSystem::new()),
    Arc::new(TickingSystem::new()),
    Arc::new(TcpListenerSystem::new()),
]);

pub async fn start_all_systems(state: GlobalState) -> NetResult<()> {
    let handles = FuturesUnordered::new();

    for system in &*SYSTEMS {
        let name = system.name();

        let handle = tokio::spawn(
            system.clone()
                .start(state.clone())
                .instrument(debug_span!("sys", %name)),
        );
        handles.push(handle);
    }

    futures::future::join_all(handles).await;

    Ok(())
}

pub async fn stop_all_systems(state: GlobalState) {
    //tokio::spawn(async move {
        info!("Stopping all systems...");

        futures::stream::iter(&*SYSTEMS).for_each_concurrent(None, |system| {
            let state = state.clone();
            async move {
                debug!("Stopping system: {}", system.name());
                system.clone().stop(state).await;
            }
        }).await;
    //})
}
