use crate::*;
use ferrumc_net_codec::encode::{
    NetEncode, NetEncodeOpts, errors::NetEncodeError
};
use ferrumc_nbt::{NBTSerializable, NBTSerializeOptions};
use std::io::Write;
use std::marker::Unpin;
use tokio::io::AsyncWriteExt;
use std::fmt;
use std::ops::Add;
use std::str::FromStr;
use std::ops::{Deref, DerefMut};

impl Deref for TextComponent {
    type Target = TextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TextComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        Self(TextInner {
            content: TextContent::Text {
                text: value,
            },
            ..Default::default()
        })
    }
}

impl From<&str> for TextComponent {
    fn from(value: &str) -> Self {
        Self(TextInner {
            content: TextContent::Text {
                text: value.into(),
            },
            ..Default::default()
        })
    }
}

impl<T> Add<T> for TextComponent
where
    T: Into<TextComponent>,
{
    type Output = Self;

    fn add(mut self, other: T) -> Self {
        self.extra.push(other.into());
        self
    }
}

impl<T> Add<T> for TextComponentBuilder
where
    T: Into<TextComponent>,
{
    type Output = Self;

    fn add(mut self, other: T) -> Self {
        self.extra.push(other.into());
        self
    }
}

impl FromStr for TextComponent {
    type Err = serde_json::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self::default())
        } else {
            serde_json::from_str(s)
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

impl TextComponent {
    pub fn serialize_nbt(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        NBTSerializable::serialize(self, &mut vec, &NBTSerializeOptions::Network);
        vec
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

impl NBTSerializable for TextComponent {
    fn serialize(&self, buf: &mut Vec<u8>, options: &NBTSerializeOptions<'_>) {
        // writing manually since i cant get derive to work
        if matches!(options, NBTSerializeOptions::Network) {
            NBTSerializable::serialize(&Self::id(), buf, &NBTSerializeOptions::None);
        }

        match &self.0.content {
            TextContent::Text { text } => {
                NBTSerializable::serialize(&"text", buf, &NBTSerializeOptions::WithHeader("type"));
                NBTSerializable::serialize(text, buf, &NBTSerializeOptions::WithHeader("text"));
            },
            TextContent::Keybind { keybind } => {
                NBTSerializable::serialize(&"keybind", buf, &NBTSerializeOptions::WithHeader("type"));
                NBTSerializable::serialize(keybind, buf, &NBTSerializeOptions::WithHeader("keybind"));
            },
            TextContent::Translate { translate, with} => {
                NBTSerializable::serialize(&"translatable", buf, &NBTSerializeOptions::WithHeader("type"));
                NBTSerializable::serialize(translate, buf, &NBTSerializeOptions::WithHeader("translate"));
                if !with.is_empty() {
                    NBTSerializable::serialize(with, buf, &NBTSerializeOptions::WithHeader("with"));
                }
            }
        }

        /*
        pub color: Option<Color>,
        pub bold: Option<bool>,
        pub italic: Option<bool>,
        pub underlined: Option<bool>,
        pub strikethrough: Option<bool>,
        pub obfuscated: Option<bool>,
        */
        if let Some(ref val) = self.0.color {
            NBTSerializable::serialize(val, buf, &NBTSerializeOptions::WithHeader("color"));
        }
        if let Some(ref val) = self.0.bold {
            NBTSerializable::serialize(val, buf, &NBTSerializeOptions::WithHeader("bold"));
        }
        if let Some(ref val) = self.0.italic {
            NBTSerializable::serialize(val, buf, &NBTSerializeOptions::WithHeader("italic"));
        }
        if let Some(ref val) = self.0.underlined {
            NBTSerializable::serialize(val, buf, &NBTSerializeOptions::WithHeader("underlined"));
        }
        if let Some(ref val) = self.0.strikethrough {
            NBTSerializable::serialize(val, buf, &NBTSerializeOptions::WithHeader("strikethrough"));
        }
        if let Some(ref val) = self.0.obfuscated {
            NBTSerializable::serialize(val, buf, &NBTSerializeOptions::WithHeader("obfuscated"));
        }

        if !self.0.extra.is_empty() {
            NBTSerializable::serialize(&9u8, buf, &NBTSerializeOptions::None);
            NBTSerializable::serialize(&"extra", buf, &NBTSerializeOptions::None);
            NBTSerializable::serialize(&Self::id(), buf, &NBTSerializeOptions::None);
            NBTSerializable::serialize(&(self.0.extra.len() as i32), buf, &NBTSerializeOptions::None);
            for elem in &self.0.extra {
                NBTSerializable::serialize(elem, buf, &NBTSerializeOptions::None);
            }
        }

        NBTSerializable::serialize(&0u8, buf, &NBTSerializeOptions::None);  
    }

    fn id() -> u8 { 10 }
}

impl Default for TextContent {
    fn default() -> Self {
        TextContent::Text {
            text: String::new(),
        }
    }
}
