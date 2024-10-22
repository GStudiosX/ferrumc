// Security or something like that
#![forbid(unsafe_code)]
#![feature(async_closure)]

#![feature(slice_as_chunks)]

use ferrumc_ecs::Universe;
use ferrumc_net::ServerState;
use std::sync::{Arc};
use tracing::{error, info};
use ferrumc_net::server::create_server_listener;
use systems::definition;
use ferrumc::get_scheduler;

pub(crate) mod errors;
mod packet_handlers;
mod systems;

pub type Result<T> = std::result::Result<T, errors::BinaryError>;

/*
use ferrumc::events::{event_handler, PlayerStartLoginEvent, GlobalState, NetError, RwEvent};

#[event_handler]
async fn handle_login_start(
    event: RwEvent<PlayerStartLoginEvent>,
    state: GlobalState,
) -> std::result::Result<RwEvent<PlayerStartLoginEvent>, NetError> {
    {
        let profile = &mut event.write().unwrap().profile;
        profile.username = profile.username.clone() + "1";
    }
    Ok(event)
}
*/

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
    let state = create_state().await?;
    let global_state = Arc::new(state);
    
    let all_systems = tokio::spawn(definition::start_all_systems(Arc::clone(&global_state)));

    // Start the systems and wait until all of them are done
    all_systems.await??;
    
    // Stop all systems
    definition::stop_all_systems(global_state).await?;

    Ok(())
}


async fn create_state() -> Result<ServerState> {
    let listener = create_server_listener().await?;

    Ok(ServerState {
        universe: Universe::new(),
        tcp_listener: listener,
    })
}
