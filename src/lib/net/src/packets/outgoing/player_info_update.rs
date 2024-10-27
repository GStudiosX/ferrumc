use ferrumc_macros::{NetEncode, packet};
use ferrumc_net_codec::net_types::{
    var_int::VarInt,
    length_prefixed_vec::LengthPrefixedVec,
};
use crate::connection::{GameProfile, ProfileProperty};
use bitmask_enum::bitmask;
use std::io::Write;
//use std::collections::HashSet;
//use std::hash::{Hash, Hasher};

#[bitmask(u8)]
#[derive(NetEncode)]
pub enum PlayerActions {
    AddPlayer = 0x01,
    InitializeChat = 0x02,
    UpdateGameMode = 0x04,
    UpdateListed = 0x08,
    UpdateLatency = 0x10,
    UpdateDisplayName = 0x20,
}

#[derive(NetEncode, Debug)]
#[packet(packet_id = 0x3E)]
pub struct PlayerInfoUpdatePacket {
    player_actions: PlayerActions,
    player_infos: LengthPrefixedVec<PlayerInfo>,
}

impl PlayerInfoUpdatePacket {
    pub fn new(player_infos: Vec<PlayerInfo>) -> Result<Self, String>
    {
        Ok(Self {
            player_actions: Self::get_player_actions(&player_infos)?,
            player_infos: LengthPrefixedVec::new(player_infos),
        })
    }

    pub fn is_valid(player_infos: &[PlayerInfo]) -> bool {
        let first = &player_infos[0].actions;
        player_infos.iter().all(|info| &info.actions == first)
    }

    pub fn get_player_actions(player_infos: &[PlayerInfo]) -> Result<PlayerActions, String> { 
        if !Self::is_valid(player_infos) {
            Err("The player infos provided is invalid.".to_string())
        } else {
            let first = &player_infos[0].actions;
            let mut flags = PlayerActions::none();

            for action in first.iter() {
                // for each action apply flag
                flags |= match action {
                    PlayerAction::AddPlayer { .. } => PlayerActions::AddPlayer,
                    PlayerAction::InitializeChat { .. } => PlayerActions::InitializeChat,
                    PlayerAction::UpdateGameMode { .. } => PlayerActions::UpdateGameMode,
                    PlayerAction::UpdateListed { .. } => PlayerActions::UpdateListed,
                    PlayerAction::UpdateLatency { .. } => PlayerActions::UpdateLatency,
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
    // note: Not sure if this should be HashSet
    pub actions: Vec<PlayerAction>,
}

impl PlayerInfo {
    pub fn from(profile: &GameProfile) -> Self {
        Self {
            uuid: profile.uuid,
            actions: vec![
                PlayerAction::add_player(profile),
                PlayerAction::UpdateListed { listed: true }
            ],
        }
    }
}

#[derive(NetEncode, Debug, Eq, Clone)]
pub enum PlayerAction {
    AddPlayer {
        username: String,
        properties: LengthPrefixedVec<ProfileProperty>,
    },
    InitializeChat { // TODO
    },
    UpdateGameMode {
        gamemode: VarInt,
    },
    UpdateListed {
        listed: bool,
    },
    UpdateLatency {
        latency: VarInt,
    },
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

/*impl Hash for PlayerAction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}*/
