use crate::*;
use ferrumc_net_codec::encode::{
    NetEncode, NetEncodeOpts, errors::NetEncodeError
};
use std::io::Write;
use std::marker::Unpin;
use tokio::io::AsyncWriteExt;
use std::fmt;

impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        Self {
            content: TextContent::Text {
                text: value,
            },
            ..Default::default()
        }
    }
}

impl From<&str> for TextComponent {
    fn from(value: &str) -> Self {
        Self {
            content: TextContent::Text {
                text: value.into(),
            },
            ..Default::default()
        }
    }
}

impl Into<String> for TextComponent {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl fmt::Display for TextComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(value) = serde_json::to_string(self) {
            write!(f, "{}", value)
        } else {
            write!(f, "Couldn't convert to String")
        }
    }
}

/*fn serialize_none(v: &TextComponent) -> Vec<u8> {
}*/

impl TextComponent {
    fn serialize_nbt(&self) -> Vec<u8> {
        /*use ferrumc_nbt::{NBTSerializable, NBTSerializeOptions};
        let mut vec = Vec::new();
        NBTSerializable::serialize(self, &mut vec, &NBTSerializeOptions::None);
        vec*/
        self.serialize_as_network()
    }
}

impl NetEncode for TextComponent {
    fn encode<W: Write>(&self, writer: &mut W, _: &NetEncodeOpts) -> Result<(), NetEncodeError> {
        writer.write_all(&self.serialize_nbt()[..])?;
        Ok(())
    }

    async fn encode_async<W: AsyncWriteExt + Unpin>(&self, writer: &mut W, _: &NetEncodeOpts) -> Result<(), NetEncodeError>{
        writer.write_all(&self.serialize_nbt()[..]).await?;
        Ok(())
    }
}

impl Default for TextContent {
    fn default() -> Self {
        TextContent::Text {
            text: String::new(),
        }
    }
}
