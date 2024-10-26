// Security or something like that
#![forbid(unsafe_code)]
#![feature(async_closure)]

#![feature(slice_as_chunks)]

use ferrumc_ecs::Universe;
use ferrumc_net::ServerState;
use tracing::{error, info};
use ferrumc_net::server::create_server_listener;
use systems::definition;
use std::sync::Arc;
use tracing::{error, info, trace};
use ferrumc::ServerState;

pub(crate) mod errors;
mod packet_handlers;
mod systems;
mod velocity;

pub type Result<T> = std::result::Result<T, errors::BinaryError>;

// test
/*#[derive(NetEncode)]
#[packet(packet_id = 0x6C)]
struct SystemChatMessage {
    message: TextComponent,
    overlay: bool,
}

#[event_handler]
async fn test_join(
    event: PlayerJoinGameEvent,
    _state: GlobalState,
) -> NetResult<PlayerJoinGameEvent> {
    let entity = event.entity.clone();
    get_scheduler().schedule_task(move |state| async move {
        let mut writer = entity
            .get_mut::<StreamWriter>(Arc::clone(&state))?;
        let profile = entity
            .get::<Profile>(Arc::clone(&state))?
            .clone();

        writer.send_packet(&SystemChatMessage {
            message: (ComponentBuilder::text("[") 
                + ComponentBuilder::text(profile.profile.unwrap().username) 
                + ComponentBuilder::text("] ") 
                + ComponentBuilder::text("Hello, World!")
                    .color(NamedColor::Blue))
                    .build(),
            overlay: false,
        }, &NetEncodeOpts::WithLength).await?;

        Ok(())
    }, std::time::Duration::from_secs(5), None).await;

    Ok(event)
}*/

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
    if ferrumc_config::get_global_config().velocity.enabled {
        trace!("Velocity Support Enabled");
    }

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
