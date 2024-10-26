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
use ferrumc::get_scheduler;
use std::io::Cursor;
use tokio::io::AsyncReadExt;

pub(crate) mod errors;
mod packet_handlers;
mod systems;

pub type Result<T> = std::result::Result<T, errors::BinaryError>;

use ferrumc::{
    events::{event_handler, LoginPluginResponseEvent, PlayerStartLoginEvent, PlayerJoinGameEvent, GlobalState, NetError, RwEvent, EventsError},
    EntityExt, NetDecodeOpts, NetDecode, NetEncodeOpts, 
    macros::{NetEncode, packet}, StreamWriter, ServerState, NetResult,
    Profile, GameProfile, net_types::var_int::VarInt,
    text::*
};
use tokio::io::AsyncWriteExt;
use std::io::Write;
use sha2::Sha256;
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

struct VelocityMessageId(u32);

#[event_handler]
async fn handle_login_start(
    event: RwEvent<PlayerStartLoginEvent>,
    state: GlobalState,
) -> NetResult<RwEvent<PlayerStartLoginEvent>> {
    if ferrumc_config::get_global_config().velocity.enabled {
        let ev = event.read().unwrap().clone();

        let id = rand::random::<u32>();
        let mut writer = ev.entity
            .get_mut::<StreamWriter>(Arc::clone(&state))?;
        writer.send_packet(&ferrumc_net::packets::outgoing::client_bound_plugin_message::LoginPluginMessagePacket::<()>::new(id, String::from("velocity:player_info"), ()), &NetEncodeOpts::WithLength).await?;
        state.universe.add_component(ev.entity, VelocityMessageId(id))?;

        // this stops the packet hqndler from doing login success
        Err(NetError::EventsError(EventsError::Other("cancel login success".to_string())))
    } else {
        Ok(event)
    }
}

#[event_handler]
async fn handle_velocity_response(
    event: LoginPluginResponseEvent,
    state: GlobalState,
) -> NetResult<LoginPluginResponseEvent> {
    let message = &event.packet;
    if message.message_id.val as u32 == event.entity.get::<VelocityMessageId>(Arc::clone(&state))?.0 {
        state.universe.remove_component::<VelocityMessageId>(event.entity)?;

        let len = message.data.len();

        let mut signature = vec![0u8; 32];
        let mut data = Vec::with_capacity(256);
        let mut buf = Cursor::new(&message.data);

        if len > 0 && message.success {
            buf.read_exact(&mut signature).await?;

            let index = buf.position();
            buf.read_to_end(&mut data).await?;
            buf.set_position(index);

            let version = VarInt::decode(&mut buf, &NetDecodeOpts::None)?;
            let _addr = String::decode(&mut buf, &NetDecodeOpts::None)?;

            //info!("{}", addr);

            if version != 1 {
                return Err(NetError::kick(TextComponentBuilder::new("[FerrumC]")
                    .color(NamedColor::Blue)
                    .space()
                    .extra(ComponentBuilder::text("This velocity modern forwarding version is not supported!")
                        .color(NamedColor::Red))
                    .build()));
            }
        } else {
            return Err(NetError::kick(ComponentBuilder::text("[FerrumC]")
                .color(NamedColor::Blue)
                .space()
                .extra(ComponentBuilder::text("The velocity proxy did not send forwarding information!")
                    .color(NamedColor::Red))
                .build()));
            //return Err(NetError::kick("§cForwarding Information was not sent".to_string()))
        }

        let mut key = HmacSha256::new_from_slice(ferrumc_config::get_global_config().velocity.secret.as_bytes())
            .expect("Failed to create HmacSha256 for velocity secret");
        key.update(&data);

        if key.verify_slice(&signature[..]).is_ok() {
            ferrumc::internal::send_login_success(
                event.entity,
                GameProfile::decode(&mut buf, &NetDecodeOpts::None)?,
                Arc::clone(&state)
            ).await?;

            Ok(event)
        } else {
            Err(NetError::kick("Invalid proxy response!".to_string()))
        }
    } else {
        Ok(event)
    }
}

// test
#[derive(NetEncode)]
#[packet(packet_id = 0x6C)]
struct SystemChatMessage {
    message: TextComponent,
    overlay: bool,
}

#[event_handler]
async fn test_join(
    event: PlayerJoinGameEvent,
    state: GlobalState,
) -> NetResult<PlayerJoinGameEvent> {
    let entity = event.entity.clone();
    get_scheduler().schedule_task(move |state| async move {
        let mut writer = entity
            .get_mut::<StreamWriter>(Arc::clone(&state))?;
        let profile = entity
            .get::<Profile>(Arc::clone(&state))?;

        writer.send_packet(&SystemChatMessage {
            message: ComponentBuilder::text("Hello, World!")
                .color(NamedColor::Blue)
                .build(),
            overlay: false,
        }, &NetEncodeOpts::WithLength).await?;

        Ok(())
    }, std::time::Duration::from_secs(5), None).await;

    Ok(event)
}

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
