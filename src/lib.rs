#[cfg(feature = "text2d")]
mod text2d;
mod ui;

#[cfg(feature = "text2d")]
use bevy::text::Text2dUpdateSystems;
use bevy::{prelude::*, ui::UiSystems};

#[cfg(feature = "text2d")]
pub use text2d::{LinkedRubyText2d, RubyText2d};
pub use ui::{LinkedRubyText, RubyText};

pub struct FuriganaPlugin;

impl Plugin for FuriganaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FuriganaSettings>()
            .add_systems(PostUpdate, ui::update_ruby.after(UiSystems::Layout))
            .add_systems(PostUpdate, ui::update_ruby_text.before(UiSystems::Layout))
            .add_observer(ui::add_ruby)
            .add_observer(ui::add_ruby_text_span);

        #[cfg(feature = "text2d")]
        app.add_systems(
            PostUpdate,
            text2d::update_ruby_2d.before(Text2dUpdateSystems),
        )
        .add_systems(
            PostUpdate,
            text2d::update_ruby_text_2d.after(Text2dUpdateSystems),
        )
        .add_observer(text2d::add_ruby_2d)
        .add_observer(text2d::add_ruby_text_span_2d);
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

#[derive(Component, Clone, Debug)]
pub struct Ruby {
    pub rt: String,
    pub position: RubyPosition,
    pub font_size_scale: f32,
}

impl Ruby {
    pub fn new(rt: impl Into<String>) -> Self {
        Self {
            rt: rt.into(),
            position: RubyPosition::default(),
            font_size_scale: 0.5,
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
    #[default]
    Over,
    Under,
}
