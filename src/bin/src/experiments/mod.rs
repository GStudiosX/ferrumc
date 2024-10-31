use ferrumc::{
    macros::{NetEncode, packet},
    text::*,
};
use std::io::Write;

#[derive(NetEncode)]
#[packet(packet_id = 0x6C)]
pub struct SystemChatMessage {
    message: TextComponent,
    overlay: bool,
}

impl SystemChatMessage {
    pub fn message<T: Into<TextComponent>>(message: T) -> Self {
        Self {
            message: message.into(),
            overlay: false,
        }
    }

    pub fn actionbar<T: Into<TextComponent>>(message: T) -> Self {
        Self {
            message: message.into(),
            overlay: true,
        }
    }
}

mod chat;
mod scheduler;
