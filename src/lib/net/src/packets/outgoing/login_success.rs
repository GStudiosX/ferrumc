use ferrumc_macros::{packet, NetEncode};
use std::io::Write;
use crate::connection::GameProfile;

#[derive(NetEncode)]
#[packet(packet_id = 0x02)]
pub struct LoginSuccessPacket {
    pub profile: GameProfile,
    pub strict_error_handling: bool,
}

impl LoginSuccessPacket {
    pub fn new(profile: GameProfile) -> Self
    {
        Self {
            profile,
            strict_error_handling: false,
        }
    }
}
