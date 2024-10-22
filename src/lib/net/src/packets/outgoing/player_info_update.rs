use ferrumc_macros::{NetEncode, packet};
use ferrumc_net_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use ferrumc_net_codec::encode::NetEncode;
use crate::connection::{GameProfile, ProfileProperty};
use bitmask_enum::bitmask;
use std::io::Write;
use tokio::io::AsyncWriteExt;

#[bitmask(u8)]
#[derive(NetEncode)]
pub enum PlayerActions {
    AddPlayer = 1,
    IntializeChat = 2,
    UpdateGameMode = 4,
    UpdateListed = 8,
    UpdateLatency = 10,
    UpdateDisplayName = 20,
}

#[derive(NetEncode)]
#[packet(packet_id = 0x3E)]
pub struct PlayerInfoUpdatePacket {
    pub player_actions: PlayerActions,
    pub player_infos: LengthPrefixedVec<PlayerInfo>,
}

#[derive(NetEncode)]
pub struct PlayerInfo {
    pub uuid: u128,
    pub actions: Vec<PlayerAction>
}

#[derive(NetEncode)]
pub enum PlayerAction {
    AddPlayer {
        username: String,
        properties: LengthPrefixedVec<ProfileProperty>,
    },
    UpdateListed(bool),
}

impl PlayerAction {
    pub fn add_player(profile: &GameProfile) -> Self {
        Self::AddPlayer {
            username: profile.username.clone(),
            properties: profile.properties.clone()
        }
    }
}

impl PlayerInfoUpdatePacket {
    pub fn new(actions: PlayerActions, player_infos: Vec<PlayerInfo>) -> Self
    {
        Self {
            player_actions: actions,
            player_infos: LengthPrefixedVec::new(player_infos),
        }
    }
}
