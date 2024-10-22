use ferrumc_macros::{packet, NetEncode};
use ferrumc_net_codec::encode::NetEncode;
use ferrumc_net_codec::net_types::var_int::VarInt;
use std::io::Write;
use tokio::io::AsyncWriteExt;

#[derive(NetEncode)]
#[packet(packet_id = 0x01)]
pub struct ConfigurationPluginMessagePacket<T>
where
    T: NetEncode {
    pub channel: String,
    pub data: T,
}

#[derive(NetEncode)]
#[packet(packet_id = 0x19)]
pub struct PlayPluginMessagePacket<T>
where
    T: NetEncode {
    pub channel: String,
    pub data: T,
}


impl<T> ConfigurationPluginMessagePacket<T>
where
    T: NetEncode {
    pub fn new(channel: String, data: T) -> Self
    {
        Self {
            channel,
            data
        }
    }
}

impl<T> PlayPluginMessagePacket<T>
where
    T: NetEncode {
    pub fn new_play(channel: String, data: T) -> Self
    {
        Self {
            channel,
            data
        }
    }
}
