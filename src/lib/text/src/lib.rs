use ferrumc_macros::NBTSerialize;
use serde::{Serialize, Deserialize};

mod r#impl;
#[cfg(test)]
mod tests;

pub mod color;

mod builders;
pub use builders::*;

use color::Color;

pub type JsonTextComponent = String;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, NBTSerialize, Default)]
pub struct TextComponent {
    #[serde(flatten)]
    #[nbt(flatten)]
    pub content: TextContent,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    /*#[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,*/

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[nbt(skip_if = "Vec::is_empty")]
    pub extra: Vec<TextComponent>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, NBTSerialize)]
#[serde(untagged)]
pub enum TextContent {
    Text {
        text: String,
    },
    Translate {
        translate: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        with: Vec<TextComponent>,
    },
    Keybind {
        keybind: String,
    },
}
