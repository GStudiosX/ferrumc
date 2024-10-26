use ferrumc_macros::NBTSerialize;
use serde::{Serialize, Deserialize};
use ferrumc_nbt::NBTSerializable;
use ferrumc_nbt::NBTSerializeOptions;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Color {
    Named(NamedColor),
    // TODO: come up with a way for custom colors
    /*#[serde(untagged)]
    HexString(String),
    #[serde(untagged)]
    Hex(&'static str),*/
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(rename_all(serialize = "snake_case"))]
pub enum NamedColor {
    #[default]
    Reset,
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
    White,
}

impl fmt::Display for NamedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Reset => "reset",
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
            NBTSerializeOptions::Network => {}
        }

        match self {
            Color::Named(color) => 
                NBTSerializable::serialize(color, buf, &NBTSerializeOptions::None),
        }
    }

    fn id() -> u8 { 0 }
}

impl From<NamedColor> for Color {
    fn from(value: NamedColor) -> Color {
        Color::Named(value)
    }
}
