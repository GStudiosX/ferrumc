// Security or something like that
#![forbid(unsafe_code)]
#![feature(async_closure)]

#![feature(slice_as_chunks)]

use ferrumc::{ServerState, get_global_config};
use ferrumc_ecs::Universe;
use ferrumc_net::server::create_server_listener;
use systems::definition;
use std::sync::Arc;
use tracing::{error, info, trace};

pub(crate) mod errors;
mod packet_handlers;
mod systems;
mod velocity;

#[cfg(feature = "experiments")]
mod experiments;

pub type Result<T> = std::result::Result<T, errors::BinaryError>;

#[tokio::main]
async fn main() {
    ferrumc_logging::init_logging();

    println!("good day to ya. enjoy your time with ferrumc!");

    if let Err(e) = entry().await {
        error!("Server exited with the following error;");
        error!("{:?}", e);
    } else {
        info!("Server exited successfully.");
    }
}

async fn entry() -> Result<()> {
    if get_global_config().velocity.enabled {
        trace!("Velocity Support Enabled");
    }

    let state = create_state().await?;
    let global_state = Arc::new(state);

    tokio::spawn({
        let global_state = Arc::clone(&global_state);
        async move {
            tokio::signal::ctrl_c().await.unwrap();
            // Stop all systems
            definition::stop_all_systems(global_state).await;
        }
    });

    let systems = tokio::spawn(definition::start_all_systems(Arc::clone(&global_state)));
    systems.await??;
    
    Ok(())
}


async fn create_state() -> Result<ServerState> {
    let listener = create_server_listener().await?;

    Ok(ServerState {
        universe: Universe::new(),
        tcp_listener: listener,
    })
}
