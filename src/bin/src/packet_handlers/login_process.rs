use ferrumc_core::identity::player_identity::PlayerIdentity;
use ferrumc_ecs::components::storage::ComponentRefMut;
use ferrumc_net::connection::{ConnectionState, StreamWriter};
use ferrumc_net::errors::NetError;
use ferrumc::events::{event_handler, Event, PlayerStartLoginEvent, PlayerJoinGameEvent RwEvent, EventsError};
use ferrumc_net::errors::NetError;
use ferrumc_net::connection::{ConnectionState, StreamWriter, GameProfile, Profile};
use ferrumc_net::packets::incoming::ack_finish_configuration::AckFinishConfigurationEvent;
use ferrumc_net::packets::incoming::login_acknowledged::LoginAcknowledgedEvent;
use ferrumc_net::packets::incoming::login_start::LoginStartEvent;
use ferrumc_net::packets::incoming::server_bound_known_packs::ServerBoundKnownPacksEvent;
use ferrumc_net::packets::outgoing::client_bound_known_packs::ClientBoundKnownPacksPacket;
use ferrumc_net::packets::outgoing::game_event::GameEventPacket;
use ferrumc_net::packets::outgoing::keep_alive::{KeepAlive, KeepAlivePacket};
use ferrumc_net::packets::outgoing::login_play::LoginPlayPacket;
use ferrumc_net::packets::outgoing::login_success::LoginSuccessPacket;
use ferrumc_net::packets::outgoing::registry_data::{get_registry_packets};
use ferrumc_net::packets::outgoing::set_default_spawn_position::SetDefaultSpawnPositionPacket;
use ferrumc_net::packets::outgoing::synchronize_player_position::SynchronizePlayerPositionPacket;
use ferrumc_net::GlobalState;
use ferrumc_net_codec::encode::NetEncodeOpts;
use ferrumc_net::packets::outgoing::finish_configuration::FinishConfigurationPacket;
use tracing::{debug, trace, info};
use ferrumc_net::packets::outgoing::client_bound_plugin_message::{ConfigurationPluginMessagePacket, PlayPluginMessagePacket};
use ferrumc_net::packets::outgoing::player_info_update::{PlayerInfoUpdatePacket, PlayerActions, PlayerInfo, PlayerAction};
use ferrumc_net_codec::encode::{NetEncodeOpts};
use std::sync::Arc;

#[event_handler]
async fn handle_login_start(
    login_start_event: LoginStartEvent,
    state: GlobalState,
) -> Result<LoginStartEvent, NetError> {
    debug!("Handling login start event");


    let uuid = login_start_event.login_start_packet.uuid;
    let username = login_start_event.login_start_packet.username.as_str();
    debug!("Received login start from user with username {}", username);
    
    //Send a Login Success Response to further the login sequence
    let mut writer = state
        .universe
        .get_mut::<StreamWriter>(login_start_event.conn_id)?;

    let mut profile = state
        .universe
        .get_mut::<Profile>(login_start_event.conn_id)?;

    //Send a Login Success Response to further the login sequence
    let event = RwEvent::new(PlayerStartLoginEvent {
        profile: GameProfile::new(uuid, username),
    });
    RwEvent::<PlayerStartLoginEvent>::trigger(event.clone(), Arc::clone(&state)).await?;

    let event = if let Some(event) = event.into_inner() {
        event
    } else {
        return Err(NetError::EventsError(EventsError::Other(format!("failed to get game profile"))));
    };

    let game_profile = event.profile;
    let response = LoginSuccessPacket::new(game_profile.clone());
    writer.send_packet(&response, &NetEncodeOpts::WithLength).await?;

    // Add the player identity component to the ECS for the entity.
    state.universe.add_component::<PlayerIdentity>(
        login_start_event.conn_id,
        PlayerIdentity::new(game_profile.username.clone(), game_profile.uuid),
    )?;

    profile.profile = Some(game_profile);

    Ok(login_start_event)
}

#[event_handler]
async fn handle_login_acknowledged(
    login_acknowledged_event: LoginAcknowledgedEvent,
    state: GlobalState,
) -> Result<LoginAcknowledgedEvent, NetError> {
    trace!("Handling Login Acknowledged event");

    //Set the connection State to Configuration
    let mut connection_state = state
        .universe
        .get_mut::<ConnectionState>(login_acknowledged_event.conn_id)?;

    *connection_state = ConnectionState::Configuration;

    let mut writer = state
        .universe
        .get_mut::<StreamWriter>(login_acknowledged_event.conn_id)?;

    let server_brand = ConfigurationPluginMessagePacket::new(String::from("minecraft:brand"), String::from("FerrumC"));
    writer.send_packet(&server_brand, &NetEncodeOpts::WithLength).await?;

    // Send packets packet
    let client_bound_known_packs = ClientBoundKnownPacksPacket::new();
    writer.send_packet(&client_bound_known_packs, &NetEncodeOpts::WithLength).await?;

    Ok(login_acknowledged_event)
}

#[event_handler]
async fn handle_server_bound_known_packs(
    server_bound_known_packs_event: ServerBoundKnownPacksEvent,
    state: GlobalState,
) -> Result<ServerBoundKnownPacksEvent, NetError> {
    trace!("Handling Server Bound Known Packs event");

    let mut writer = state
        .universe
        .get_mut::<StreamWriter>(server_bound_known_packs_event.conn_id)?;

    let registry_packets = get_registry_packets();
    writer.send_packet(&registry_packets, &NetEncodeOpts::None).await?;
    
    writer.send_packet(&FinishConfigurationPacket::new(), &NetEncodeOpts::WithLength).await?;
    
    Ok(server_bound_known_packs_event)
}

#[event_handler]
async fn handle_ack_finish_configuration(
    ack_finish_configuration_event: AckFinishConfigurationEvent,
    state: GlobalState,
) -> Result<AckFinishConfigurationEvent, NetError> {
    trace!("Handling Ack Finish Configuration event");

    let conn_id = ack_finish_configuration_event.conn_id;

    let mut conn_state = state
        .universe
        .get_mut::<ConnectionState>(conn_id)?;

    *conn_state = ConnectionState::Play;

    let mut writer = state
        .universe
        .get_mut::<StreamWriter>(conn_id)?;

    writer.send_packet(&LoginPlayPacket::new(conn_id), &NetEncodeOpts::WithLength).await?;
    writer.send_packet(&SetDefaultSpawnPositionPacket::default(), &NetEncodeOpts::WithLength).await?;
    writer.send_packet(&SynchronizePlayerPositionPacket::default(), &NetEncodeOpts::WithLength).await?;
    writer.send_packet(&GameEventPacket::start_waiting_for_level_chunks(), &NetEncodeOpts::WithLength).await?;

    if let Some(profile) = &state
        .universe
        .get::<Profile>(ack_finish_configuration_event.conn_id)?
        .profile {
        let info_update = PlayerInfoUpdatePacket::new(PlayerActions::AddPlayer | PlayerActions::UpdateListed, vec![
            PlayerInfo {
                uuid: profile.uuid,
                actions: vec![
                    PlayerAction::add_player(&profile),
                    PlayerAction::UpdateListed(true)
                ]
            }
        ]);
        writer.send_packet(&info_update, &NetEncodeOpts::WithLength).await?;
    }

    send_keep_alive(conn_id, state, &mut writer).await?;

    PlayerJoinGameEvent::trigger(PlayerJoinGameEvent {
        entity: ack_finish_configuration_event.conn_id
    }, Arc::clone(&state)).await?;

    Ok(ack_finish_configuration_event)
}

async fn send_keep_alive(conn_id: usize, state: GlobalState, writer: &mut ComponentRefMut<'_, StreamWriter>) -> Result<(), NetError> {
    let keep_alive_packet = KeepAlivePacket::default();
    writer.send_packet(&keep_alive_packet, &NetEncodeOpts::WithLength).await?;

    let id = keep_alive_packet.id;
    state.universe.add_component::<KeepAlive>(conn_id, id)?;
    
    Ok(())
}
