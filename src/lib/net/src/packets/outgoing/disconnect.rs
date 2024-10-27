use ferrumc_macros::{packet, NetEncode};
use std::io::Write;
use ferrumc_text::*;

#[derive(NetEncode)]
pub enum DisconnectPacket {
    Login(LoginDisconnect),
    Play(PlayDisconnect),
}

#[derive(NetEncode)]
#[packet(packet_id = 0x00)]
pub struct LoginDisconnect {
    pub reason: JsonTextComponent,
}

#[derive(NetEncode)]
#[packet(packet_id = 0x1D)]
pub struct PlayDisconnect {
    pub reason: TextComponent,
}

impl LoginDisconnect {
    pub fn new<C: Into<JsonTextComponent>>(reason: C) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl PlayDisconnect {
    pub fn new<C: Into<TextComponent>>(reason: C) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

