use ferrumc_macros::{packet, NetEncode};
use std::io::Write;
use ferrumc_text::JsonTextComponent;

#[derive(NetEncode)]
#[packet(packet_id = 0x00)]
pub struct LoginDisconnect {
    pub reason: JsonTextComponent,
}

impl LoginDisconnect {
    pub fn new<C: Into<JsonTextComponent>>(reason: C) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}
