use crate::{*, color::Color};

pub struct ComponentBuilder {
    _private: ()
}

impl ComponentBuilder {
    #[inline]
    pub fn text<S: Into<String>>(value: S) -> TextComponentBuilder {
        TextComponentBuilder::new(value)
    }

    #[inline]
    pub fn keybind<S: Into<String>>(keybind: S) -> TextComponent {
        TextComponent {
            content: TextContent::Keybind {
                keybind: keybind.into()
            },
            ..Default::default()
        }
    }

    #[inline]
    pub fn space() -> TextComponent {
        " ".into()
    }
}

#[derive(Default)]
pub struct TextComponentBuilder {
    text: String,
    color: Option<Color>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    extra: Vec<TextComponent>,
}

impl TextComponentBuilder {
    pub fn new<S: Into<String>>(value: S) -> TextComponentBuilder {
        TextComponentBuilder {
            text: value.into(),
            ..Default::default()
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn clear_color(mut self) -> Self {
        self.color = None;
        self
    }

    pub fn space(self) -> Self {
        self.extra(ComponentBuilder::space())
    }

    pub fn extra(mut self, component: impl Into<TextComponent>) -> Self {
        self.extra.push(component.into());
        self
    }

    pub fn build(self) -> TextComponent {
        TextComponent {
            content: TextContent::Text {
                text: self.text,
            },
            color: self.color,
            bold: self.bold,
            italic: self.italic,
            underlined: self.underlined,
            strikethrough: self.strikethrough,
            obfuscated: self.obfuscated,
            extra: self.extra
        }
    }
}

impl From<TextComponentBuilder> for TextComponent {
    fn from(value: TextComponentBuilder) -> Self {
        value.build()
    }
}
