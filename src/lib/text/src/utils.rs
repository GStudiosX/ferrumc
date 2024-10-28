//use ferrumc_macros::NBTSerialize;
use serde::{Serialize, Deserialize};
use ferrumc_nbt::NBTSerializable;
use ferrumc_nbt::NBTSerializeOptions;
use std::fmt;

// TODO: better api for custom colors
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Color {
    Named(NamedColor),
    Hex(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(rename_all(serialize = "snake_case"))]
pub enum NamedColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    #[default]
    White,
}

impl fmt::Display for NamedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Black => "black",
            Self::DarkBlue => "dark_blue",
            Self::DarkGreen => "dark_green",
            Self::DarkAqua => "dark_aqua",
            Self::DarkRed => "dark_red",
            Self::DarkPurple => "dark_purple",
            Self::Gold => "gold",
            Self::Gray => "gray",
            Self::DarkGray => "dark_gray",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Aqua => "aqua",
            Self::Red => "red",
            Self::LightPurple => "light_purple",
            Self::Yellow => "yellow",
            Self::White => "white",
        })
    }
}

impl NBTSerializable for NamedColor {
    fn serialize(&self, buf: &mut Vec<u8>, _: &NBTSerializeOptions<'_>) {
        NBTSerializable::serialize(&self.to_string(), buf, &NBTSerializeOptions::None);
    }

    fn id() -> u8 { 8 }
}

impl NBTSerializable for Color {
    fn serialize(&self, buf: &mut Vec<u8>, opts: &NBTSerializeOptions<'_>) {
        match opts {
            NBTSerializeOptions::None => {}
            NBTSerializeOptions::WithHeader(tag_name) => {
                NBTSerializable::serialize(tag_name, buf, &NBTSerializeOptions::Network);
            }
            NBTSerializeOptions::Network | NBTSerializeOptions::Flatten => {}
        }

        match self {
            Color::Named(color) => 
                NBTSerializable::serialize(color, buf, &NBTSerializeOptions::None),
            Color::Hex(str) =>
                NBTSerializable::serialize(str, buf, &NBTSerializeOptions::None),
        }
    }

    fn id() -> u8 { 0 }
}

impl From<NamedColor> for Color {
    fn from(value: NamedColor) -> Self {
        Self::Named(value)
    }
}

/// The font of the text component.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Font {
    /// The default font.
    #[serde(rename = "minecraft:default")]
    Default,
    /// Unicode font.
    #[serde(rename = "minecraft:uniform")]
    Uniform,
    /// Enchanting table font.
    #[serde(rename = "minecraft:alt")]
    Alt,
    #[serde(untagged)]
    Custom(String),
}

impl fmt::Display for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Default => "minecraft:default",
            Self::Uniform => "minecraft:uniform",
            Self::Alt => "minecraft:alt",
            Self::Custom(key) => key.as_str(),
        })
    }
}

impl NBTSerializable for Font {
    fn serialize(&self, buf: &mut Vec<u8>, opts: &NBTSerializeOptions<'_>) {
        NBTSerializable::serialize(&self.to_string(), buf, opts);
    }

    fn id() -> u8 { 8 }
}

impl From<String> for Font {
    fn from(value: String) -> Self {
        Self::Custom(value)
    }
}

impl From<&str> for Font {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_string())
    }
}
