use crate::packets::IncomingPacket;
use crate::{NetResult, ServerState};
use std::sync::Arc;
use ferrumc_macros::{packet, NetDecode, Event};
use ferrumc_net_codec::{
    decode::{NetDecode, NetDecodeResult, NetDecodeOpts},
    net_types::var_int::VarInt
};
use ferrumc_events::infrastructure::Event;

#[derive(Event)]
pub struct PlayerAsyncChatEvent {
    pub entity: usize,
    pub message: ChatMessagePacket,
}

// Custom type because not all options have a boolean value before it to know if it exists or not
#[derive(Debug)]
pub struct Signature(pub Option<[u8; 256]>);

impl NetDecode for Signature {
    fn decode<R: std::io::Read>(reader: &mut R, opts: &NetDecodeOpts) -> NetDecodeResult<Self> {
        if bool::decode(reader, opts)? {
            Ok(Signature(Some(<[u8; 256]>::decode(reader, opts)?)))
        } else {
            Ok(Signature(None))
        }
    }
}

#[derive(NetDecode, Debug)]
#[packet(packet_id = 0x06, state = "play")]
pub struct ChatMessagePacket {
    pub message: String,
    // don't know what to do with most of this
    pub timestamp: i64,
    pub salt: i64,
    pub signature: Signature,
    pub message_count: VarInt,
    pub acknowledged: [u8; 2], // ceil(20 / 8)
}

impl IncomingPacket for ChatMessagePacket {
    async fn handle(self, conn_id: usize, state: Arc<ServerState>) -> NetResult<()> {
        tokio::spawn(PlayerAsyncChatEvent::trigger(PlayerAsyncChatEvent {
            entity: conn_id,
            message: self,
        }, Arc::clone(&state)));

        Ok(())
    }
}
