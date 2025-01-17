use std::sync::Arc;
use std::io::Cursor;
use ferrumc::{
    events::{event_handler, LoginPluginResponseEvent, PlayerStartLoginEvent, GlobalState, NetError, RwEvent, EventsError},
    EntityExt, NetDecodeOpts, NetDecode, NetEncodeOpts,
    StreamWriter, NetResult, GameProfile, net_types::var_int::VarInt,
    text::*, get_global_config
};
use tokio::io::AsyncReadExt;
use sha2::Sha256;
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

struct VelocityMessageId(u32);

#[event_handler]
async fn handle_login_start(
    event: RwEvent<PlayerStartLoginEvent>,
    state: GlobalState,
) -> NetResult<RwEvent<PlayerStartLoginEvent>> {
    if get_global_config().velocity.enabled {
        let ev = event.read().unwrap().clone();

        let id = rand::random::<u32>();
        let mut writer = ev.entity
            .get_mut::<StreamWriter>(Arc::clone(&state))?;
        writer.send_packet(&ferrumc_net::packets::outgoing::client_bound_plugin_message::LoginPluginMessagePacket::<()>::new(id, String::from("velocity:player_info"), ()), &NetEncodeOpts::WithLength).await?;
        state.universe.add_component(ev.entity, VelocityMessageId(id))?;

        // this stops the packet handler from doing login success
        Err(NetError::EventsError(EventsError::Cancelled))
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
                    + ComponentBuilder::text("This velocity modern forwarding version is not supported!")
                        .color(NamedColor::Red)
                    .build()));
            }
        } else {
            return Err(NetError::kick(ComponentBuilder::text("[FerrumC]")
                .color(NamedColor::Blue)
                .space()
                + ComponentBuilder::text("The velocity proxy did not send forwarding information!")
                    .color(NamedColor::Red)
                .build()));
        }

        let mut key = HmacSha256::new_from_slice(get_global_config().velocity.secret.as_bytes())
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
