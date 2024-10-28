use crate::*;
use paste::paste;

/// Build a component (text, translate, keybind).
///
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
    pub fn translate<S: Into<String>>(translate: S, with: Vec<TextComponent>) -> TextComponent {
        TextComponent {
            content: TextContent::Translate {
                translate: translate.into(),
                with,
            },
            ..Default::default()
        }
    }

    #[inline]
    pub fn space() -> TextComponent {
        " ".into()
    }
}

/// A builder to build a TextComponent of type text.
///
/// ```rust
/// # use ferrumc_text::color::*;
/// # use ferrumc_text::ComponentBuilder;
/// let _ = ComponentBuilder::text("Hello,")
///     .color(NamedColor::Red)
///     .space()
///     .extra(ComponentBuilder::text("World!"))
///     .build();
/// ```
#[derive(Default)]
pub struct TextComponentBuilder {
    pub(crate) text: String,
    pub(crate) color: Option<Color>,
    pub(crate) font: Option<Font>,
    pub(crate) bold: Option<bool>,
    pub(crate) italic: Option<bool>,
    pub(crate) underlined: Option<bool>,
    pub(crate) strikethrough: Option<bool>,
    pub(crate) obfuscated: Option<bool>,
    pub(crate) insertion: Option<String>,
    pub(crate) extra: Vec<TextComponent>,
}

macro_rules! make_bool_setters {
    ($($field:ident),*) => {
        paste! {
            $(
                pub fn $field(mut self) -> Self {
                    self.$field = Some(true);
                    self
                }

                pub fn [<not_ $field>](mut self) -> Self {
                    self.$field = Some(true);
                    self
                }

                pub fn [<clear_ $field>](mut self) -> Self {
                    self.$field = None;
                    self
                }
            )*
        }
    }
}

macro_rules! make_setters {
    ($(($ty:ident, $field:ident)),*) => {
        paste! {
            $(
                pub fn $field(mut self, $field: impl Into<$ty>) -> Self {
                    self.$field = Some($field.into());
                    self
                }

                pub fn [<clear_ $field>](mut self) -> Self {
                    self.$field = None;
                    self
                }
            )*
        }
    }
}

impl TextComponentBuilder {
    pub fn new<S: Into<String>>(value: S) -> TextComponentBuilder {
        TextComponentBuilder {
            text: value.into(),
            ..Default::default()
        }
    }

    make_setters!((Color, color), (Font, font), (String, insertion));
    make_bool_setters!(italic, underlined, strikethrough, obfuscated);

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
            font: self.font,
            bold: self.bold,
            italic: self.italic,
            underlined: self.underlined,
            strikethrough: self.strikethrough,
            obfuscated: self.obfuscated,
            insertion: self.insertion,
            extra: self.extra
        }
    }
}

impl From<TextComponentBuilder> for TextComponent {
    fn from(value: TextComponentBuilder) -> Self {
        value.build()
    }
}
