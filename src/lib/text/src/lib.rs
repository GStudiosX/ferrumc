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

//#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default, NBTSerialize)]
//pub struct TextComponent(TextInner);

/// A TextComponent that can be a Text, Translate or Keybind.
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct TextComponent {
    #[serde(flatten)]
    //#[nbt(flatten)]
    /// The content field of this TextComponent.
    ///
    /// ```ignore
    /// TextContent::Text { text: "text".to_string() }
    /// TextContent::Translate { .. }
    /// TextContent::Keybind { keybind: "key.jump".to_string() }
    /// ```
    pub content: TextContent,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The color field of this TextComponent.
    pub color: Option<Color>,

    /*#[serde(default, skip_serializing_if = "Option::is_none")]
    /// The font field of this TextComponent.
    pub font: Option<Font>,*/

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The bold field of this TextComponent.
    pub bold: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The italic field of this TextComponent.
    pub italic: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The underlined field of this TextComponent.
    pub underlined: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The strikethrough field of this TextComponent.
    pub strikethrough: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The obfuscated field of this TextComponent.
    pub obfuscated: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    //#[nbt(skip_if = "Vec::is_empty")]
    /// The with field of this TextComponent.
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
        #[nbt(skip_if = "Vec::is_empty")]
        with: Vec<TextComponent>,
    },
    Keybind {
        keybind: String,
    },
}
