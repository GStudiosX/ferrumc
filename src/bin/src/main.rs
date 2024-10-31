// Security or something like that
#![forbid(unsafe_code)]
#![feature(async_closure)]

#![feature(slice_as_chunks)]

use ferrumc::{ServerState, get_global_config};
use ferrumc_ecs::Universe;
use ferrumc_net::server::create_server_listener;
use std::sync::Arc;
use systems::definition;
use tracing::{error, info, trace};
use std::time::Duration;
use rand::seq::IndexedRandom;

pub(crate) mod errors;
mod packet_handlers;
mod systems;
mod velocity;

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

    // Start the systems and wait until all of them are done
    let systems = tokio::spawn(definition::start_all_systems(Arc::clone(&global_state)));

    if get_global_config().lan.enabled {
        info!("Opening to LAN!");

        let interval = get_global_config().lan.ping_interval;

        if let Ok(socket) = tokio::net::UdpSocket::bind("0.0.0.0:0").await {
            let socket = Arc::new(socket);
            ferrumc::get_scheduler().schedule_task(move |_| {
                let socket = Arc::clone(&socket);
                async move {
                    let data = format!("[MOTD]{}[/MOTD][AD]{}[/AD]",
                        get_global_config().motd.choose(&mut rand::thread_rng()).ok_or(anyhow::anyhow!("failed to get motd"))?,
                        get_global_config().port);
                    socket.send_to(&data.as_bytes(), "224.0.2.60:4445").await?;
                    Ok(())
                }
            }, Duration::from_secs_f32(interval),
                Some(Duration::from_secs_f32(interval)))
                .await.unwrap();
        } else {
            error!("Failed to open to LAN.");
        }
    }

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
