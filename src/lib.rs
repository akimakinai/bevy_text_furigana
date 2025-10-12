//! Naive implementation of [Ruby characters](https://en.wikipedia.org/wiki/Ruby_character) for UI and 2D Text in Bevy.
#[cfg(feature = "text2d")]
mod text2d;
mod ui;

use bevy::{ecs::query::QueryData, prelude::*};

#[cfg(feature = "text2d")]
pub use text2d::{LinkedRubyText2d, RubyText2d};
pub use ui::{LinkedRubyText, RubyText};

pub struct FuriganaPlugin;

impl Plugin for FuriganaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FuriganaSettings>();

        app.add_plugins(ui::plugin);

        #[cfg(feature = "text2d")]
        app.add_plugins(text2d::plugin);
    }
}

#[derive(Resource)]
pub struct FuriganaSettings {
    /// Update `GlobalUiTransform` to eliminate one-frame delay.
    pub update_ui_global_transform: bool,
}

impl Default for FuriganaSettings {
    fn default() -> Self {
        Self {
            update_ui_global_transform: true,
        }
    }
}

/// Component to add ruby text to a `Text`, `Text2d`, or `TextSpan`.
#[derive(Component, Clone, Debug)]
pub struct Ruby {
    /// Ruby text.
    pub rt: String,
    pub position: RubyPosition,
    pub align: RubyAlign,
    /// Font size relative to this text's font size. (e.g. 0.5 for half size)
    pub font_size_scale: f32,
    /// Color for ruby text. If `None`, inherits the color of the base text.
    pub color: Option<TextColor>,
}

impl Ruby {
    pub fn new(rt: impl Into<String>) -> Self {
        Self {
            rt: rt.into(),
            position: RubyPosition::default(),
            align: RubyAlign::default(),
            font_size_scale: 0.5,
            color: None,
        }
    }
}

impl Default for Ruby {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RubyPosition {
    /// Example:
    ///
    /// <ruby style="ruby-position: over"><rb>Lorem ipsum</rb><rt>Ruby</rt></ruby>
    #[default]
    Over,
    /// Example:
    ///
    /// <ruby style="ruby-position: under"><rb>Lorem ipsum</rb><rt>Ruby</rt></ruby>
    Under,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RubyAlign {
    /// Example:
    ///
    /// <ruby style="ruby-align: start"><rb>Lorem ipsum</rb><rt>Ruby</rt></ruby>
    Start,
    /// Example:
    ///
    /// <ruby style="ruby-align: center"><rb>Lorem ipsum</rb><rt>Ruby</rt></ruby>
    #[default]
    Center,
    /// Example:
    ///
    /// <ruby style="ruby-align: end"><rb>Lorem ipsum</rb><rt>Ruby</rt></ruby>
    End,
}

#[derive(QueryData)]
struct TextRootEntity {
    this: Entity,
    child_of: Option<&'static ChildOf>,
    is_text_span: Has<TextSpan>,
}

impl<'w, 's> TextRootEntityItem<'w, 's> {
    fn get(&self) -> Option<Entity> {
        if self.is_text_span {
            self.child_of.map(ChildOf::parent)
        } else {
            Some(self.this)
        }
    }
}
