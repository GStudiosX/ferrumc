use ferrumc_macros::{NetEncode, packet};
use ferrumc_net_codec::net_types::{
    var_int::VarInt,
    length_prefixed_vec::LengthPrefixedVec,
};
use crate::connection::{GameProfile, ProfileProperty};
use bitmask_enum::bitmask;
use std::io::Write;
use tokio::io::AsyncWriteExt;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

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

#[derive(NetEncode, Debug)]
#[packet(packet_id = 0x3E)]
pub struct PlayerInfoUpdatePacket {
    player_actions: PlayerActions,
    player_infos: LengthPrefixedVec<PlayerInfo>,
}

impl PacketInfoUpdatePacket {
    pub fn new(player_infos: Vec<PlayerInfo>) -> Result<Self, String>
    {
        Self {
            player_actions: Self::get_player_actions(&player_infos)?,
            player_infos: LengthPrefixedVec::new(player_infos),
        }
    }

    pub fn is_valid(player_infos: &Vec<PlayerInfo>) -> bool {
        let first = &player_infos[0].actions;
        if player_infos.iter().all(|info| info.actions == first) {
            true
        } else {
            false
        }
    }

    pub fn get_player_actions(player_infos: &Vec<PlayerInfo>) -> Result<u8, String> {
        if !Self::is_valid(player_infos) {
            Err("The player_infos provided is invalid.".to_string())
        } else {
            let first = &player_infos[0].actions;
            let flags = 0u8;

            for action in first.iter() {
                // for each action apply flag
                flags = flags | match action {
                    PlayerAction::AddPlayer { .. } => PlayerActions::AddPlayer,
                    PlayerAction::InitializeChat { .. } => PlayerActions::InitializeChat,
                    PlayerAction::UpdateGameMode(..) => PlayerActions::UpdateGameMode,
                    PlayerAction::UpdateListed(..) => PlayerActions::UpdateListed,
                    PlayerAction::UpdateLatency(..) => PlayerActions::UpdateLatency,
                    PlayerAction::UpdateDisplayName { .. } => PlayerActions::UpdateDisplayName,
                };
            }

            Ok(flags)
        }
    }
}

#[derive(NetEncode, Debug)]
pub struct PlayerInfo {
    pub uuid: u128,
    pub actions: HashSet<PlayerAction>,
}

#[derive(NetEncode, Debug)]
pub enum PlayerAction {
    AddPlayer {
        username: String,
        properties: LengthPrefixedVec<ProfileProperty>,
    },
    InitializeChat { // TODO
    },
    UpdateGameMode(VarInt),
    UpdateListed(bool),
    UpdateLatency(VarInt),
    UpdateDisplayName { // TODO
    },
}

impl PlayerAction {
    pub fn add_player(profile: &GameProfile) -> Self {
        Self::AddPlayer {
            username: profile.username.clone(),
            properties: profile.properties.clone()
        }
    }
}

impl PartialEq for PlayerAction {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Hash for PlayerAction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}
