use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, error, info, info_span, Instrument};
use ferrumc_net::connection::handle_connection;
use ferrumc_net::GlobalState;
use crate::systems::definition::System;
use crate::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use ferrumc::{ConnectionState, StreamWriter};
use futures::StreamExt;

pub struct TcpListenerSystem {
    shutdown: AtomicBool,
}

#[async_trait]
impl System for TcpListenerSystem {
    async fn start(self: Arc<Self>, state: GlobalState) {
        if let Err(e) = self.initiate_loop(state).await {
            error!("TCP listener system failed with error: {:?}", e);
        }
    }

    async fn stop(self: Arc<Self>, state: GlobalState) {
        debug!("Stopping TCP listener system...");
        self.shutdown.store(true, Ordering::Relaxed);

        futures::stream::iter(state.universe.query::<(&mut StreamWriter, &ConnectionState)>())
            .for_each_concurrent(None, |(mut writer, conn_state)| async move {
                let _ = writer.kick(&conn_state, "Server Closed").await;
            })
            .await;
    }

    fn name(&self) -> &'static str {
        "tcp"
    }
}

impl TcpListenerSystem {
    pub fn new() -> Self {
        Self {
            shutdown: AtomicBool::new(false),
        }
    }

    async fn initiate_loop(&self, state: GlobalState) -> Result<()> {
        let tcp_listener = &state.tcp_listener;
        info!("Server is listening on [{}]", tcp_listener.local_addr()?);

        while !self.shutdown.load(Ordering::Relaxed) {
            let (stream, _) = tcp_listener.accept().await?;
            let addy = stream.peer_addr()?;
            debug!("Accepted connection from: {}", addy);
            tokio::task::spawn(
                handle_connection(Arc::clone(&state), stream)
                    .instrument(info_span!("conn", %addy).or_current())
            );
        }
        
        Ok(())
    }
}
