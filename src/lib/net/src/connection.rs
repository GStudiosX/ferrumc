use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tracing::{debug, debug_span, trace, warn, Instrument};
use ferrumc_net_codec::{
    encode::{NetEncode, NetEncodeOpts},
    decode::{NetDecode, NetDecodeOpts, NetDecodeResult}, 
    net_types::length_prefixed_vec::LengthPrefixedVec
};
use crate::{handle_packet, NetResult, ServerState};
use crate::packets::incoming::packet_skeleton::PacketSkeleton;
use ferrumc_macros::{Event, NetEncode, NetDecode};
use ferrumc_events::infrastructure::Event;
use ferrumc_ecs::entities::Entity;
use std::io::Write;
use std::io::Read;
use tokio::io::AsyncWriteExt;
use crate::errors::NetError;
use ferrumc_text::*;

#[derive(Clone, PartialEq)]
#[repr(u8)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Play,
    Configuration,
}

impl ConnectionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionState::Handshaking => "handshake",
            ConnectionState::Status => "status",
            ConnectionState::Login => "login",
            ConnectionState::Play => "play",
            ConnectionState::Configuration => "configuration",
        }
    }
}

/// The GameProfile holds information about a player.
///
/// Fields:
/// `uuid`: The uuid of the GameProfile.
/// `username`: The username of the GameProfile.
/// `properties`: The properties of the GameProfile for example textures.
///
/// ```ignore
/// GameProfile {
///     uuid: Uuid::new_v4().as_u128(),
///     username: String::from("Name"),
///     properties: vec![ProfileProperty {
///         name: String::from("textures"),
///         value: String::from("ewogICJ0aW1lc3RhbXAiIDog..."),
///         is_signed: false,
///         signature: None,
///     }],
/// }
/// ```
///
#[derive(Debug, Clone, NetEncode, NetDecode)]
pub struct GameProfile {
    /// The uuid of this GameProfile
    pub uuid: u128,
    /// The username of this GameProfile
    pub username: String,
    /// The properties of this GameProfile
    pub properties: LengthPrefixedVec<ProfileProperty>
}

/// A property of a GameProfile.
///
/// Fields:
/// `name`: The name of the Property.
/// `value`: The value of the Property.
/// `is_signed`: If the Property is signed.
/// `signature`: The signature of the Property
///
#[derive(Debug, Clone, NetEncode)]
pub struct ProfileProperty {
    /// The name of this Property.
    pub name: String,
    /// The value of this Property.
    pub value: String,
    /// If this Property is signed.
    pub is_signed: bool, // note: perhaps this field can be removed with custom NetEncode implementation.
    /// The signature of this Property.
    pub signature: Option<String>,
}

impl NetDecode for ProfileProperty {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> NetDecodeResult<Self> {
        let name = String::decode(reader, opts)?;
        let value = String::decode(reader, opts)?;
        let is_signed = bool::decode(reader, opts)?;
        let signature = if is_signed {
            Some(String::decode(reader, opts)?)
        } else {
            None
        };

        Ok(ProfileProperty {
            name, value, is_signed, signature
        })
    }
}

impl GameProfile {
    /// Create a new GameProfile from uuid and username.
    pub fn new(uuid: u128, username: String) -> Self {
        Self {
            uuid,
            username,
            properties: LengthPrefixedVec::new(vec![
                /*ProfileProperty {
                    name: String::from("textures"),
                    value: String::from("ewogICJ0aW1lc3RhbXAiIDogMTcyOTQ4NTMzNzM4MiwKICAicHJvZmlsZUlkIiA6ICJhNTNjMjllZjRjZjE0OWYxYWU5MjBiN2NjMmQ2ZDJhYSIsCiAgInByb2ZpbGVOYW1lIiA6ICJHU3R1ZGlvc1giLAogICJ0ZXh0dXJlcyIgOiB7CiAgICAiU0tJTiIgOiB7CiAgICAgICJ1cmwiIDogImh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvMTU2YzllNzQzMWE2YzYxZGIyZWJlOWI4YzQ0MWUxMzU5Y2QyMmNlZTQ1ODcwNmM1MDczMmNiM2U1MTM0NWRiNyIKICAgIH0sCiAgICAiQ0FQRSIgOiB7CiAgICAgICJ1cmwiIDogImh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvYWZkNTUzYjM5MzU4YTI0ZWRmZTNiOGE5YTkzOWZhNWZhNGZhYTRkOWE5YzNkNmFmOGVhZmIzNzdmYTA1YzJiYiIKICAgIH0KICB9Cn0="),
                    is_signed: false,
                    signature: None,
                }*/
            ])
        }
    }
}

pub struct StreamReader {
    pub reader: OwnedReadHalf,
}

impl StreamReader {
    pub fn new(reader: OwnedReadHalf) -> Self {
        Self { reader }
    }
}

pub struct StreamWriter {
    pub writer: OwnedWriteHalf,
}

impl StreamWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self { writer }
    }

    pub async fn send_packet(
        &mut self,
        packet: &impl NetEncode,
        net_encode_opts: &NetEncodeOpts,
    ) -> NetResult<()> {
        packet
            .encode_async(&mut self.writer, net_encode_opts)
            .await?;
        Ok(())
    }

    pub async fn kick<S: Into<JsonTextComponent>>(&mut self, conn_state: &ConnectionState, reason: S) -> NetResult<()> {
        let packet = if conn_state == &ConnectionState::Login {
            crate::packets::outgoing::disconnect::LoginDisconnect::new(reason)
        } else {
            return Err(NetError::InvalidState(conn_state.clone() as u8));
        };

        self.send_packet(&packet, &NetEncodeOpts::WithLength).await
    }
}

#[derive(Clone)]
pub struct Profile {
    pub profile: Option<GameProfile>,
}

impl Profile {
    pub fn new() -> Self {
        Self {
            profile: None,
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CompressionStatus {
    pub enabled: bool,
}

impl CompressionStatus {
    pub fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for CompressionStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// This is called when the player gets disconnected either by the server, player leaving or invalid packets and other errors.
///
#[derive(Event)]
pub struct PlayerDisconnectEvent {
    entity: Entity,
}

impl PlayerDisconnectEvent {
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

pub async fn handle_connection(state: Arc<ServerState>, tcp_stream: TcpStream) -> NetResult<()> {
    let (mut reader, writer) = tcp_stream.into_split();

    let entity = state
        .universe
        .builder()
        //.with(StreamReader::new(reader))?
        .with(StreamWriter::new(writer))?
        .with(ConnectionState::Handshaking)?
        .with(CompressionStatus::new())?
        .with(Profile::new())? // initialize with empty profile
        .build();

    'recv: loop {
        let compressed = state.universe.get::<CompressionStatus>(entity)?.enabled;
        let Ok(mut packet_skele) = PacketSkeleton::new(&mut reader, compressed).await else {
            trace!("Failed to read packet. Possibly connection closed.");
            break 'recv;
        };

        // Log the packet if the environment variable is set (this env variable is set at compile time not runtime!)
        if option_env!("FERRUMC_LOG_PACKETS").is_some() {
            trace!("Received packet: {:?}", packet_skele);
        }

        let conn_state = state.universe.get::<ConnectionState>(entity)?.clone();
        if let Err(e) = handle_packet(
            packet_skele.id,
            entity,
            &conn_state,
            &mut packet_skele.data,
            Arc::clone(&state),
        )
            .await
            .instrument(debug_span!("eid", %entity))
            .inner()
        {
            match e {
                NetError::Kick(msg) => {
                    warn!("Failed to handle packet: {}. packet_id: {:02X}; conn_state: {}", msg, packet_skele.id, conn_state.as_str());
                    let _ = state.universe.get_mut::<StreamWriter>(entity)?
                        .kick(&conn_state, msg)
                        .await;
                },
                _ => {
                    warn!("Failed to handle packet: {:?}. packet_id: {:02X}; conn_state: {}", e, packet_skele.id, conn_state.as_str());
                    let _ = state.universe.get_mut::<StreamWriter>(entity)?
                        .kick(&conn_state, TextComponent::from("§cDisconnected".to_string()))
                        .await;
                }
            }
            break 'recv;
        };
    }

    debug!("Connection closed for entity: {:?}", entity);

    match PlayerDisconnectEvent::trigger(PlayerDisconnectEvent { entity }, Arc::clone(&state)).await {
        Ok(_) => {}
        Err(e) => error!("Error calling player disconnect event: {}", e)
    }

    // Remove all components from the entity

    drop(reader);

    // Wait until anything that might be using the entity is done
    if let Err(e) = remove_all_components_blocking(state.clone(), entity).await {
        warn!("Failed to remove all components from entity: {:?}", e);
    }

    trace!("Dropped all components from entity: {:?}", entity);

    Ok(())
}

/// Since parking_lot is single-threaded, we use spawn_blocking to remove all components from the entity asynchronously (on another thread).
async fn remove_all_components_blocking(state: Arc<ServerState>, entity: usize) -> NetResult<()> {
    let res = tokio::task::spawn_blocking(move || {
        state.universe.remove_all_components(entity)
    }).await?;

    Ok(res?)
}
