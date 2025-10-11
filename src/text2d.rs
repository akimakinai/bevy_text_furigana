use bevy::{prelude::*, text::TextLayoutInfo};

use crate::{Ruby, RubyAlign, RubyPosition};

/// Component for 2D ruby text.
/// Automatically spawned when [`Ruby`] component is added along with `Text2d` or `TextSpan`.
#[derive(Component, Clone, Copy)]
#[relationship(relationship_target = LinkedRubyText2d)]
pub struct RubyText2d(
    /// Entity of the corresponding `Ruby` component.
    pub Entity,
);

/// Tracks ruby text entity corresponding to [`Ruby`] for 2D text.
#[derive(Component, Clone, Copy)]
#[relationship_target(relationship = RubyText2d, linked_spawn)]
pub struct LinkedRubyText2d(Entity);

impl LinkedRubyText2d {
    pub const fn entity(&self) -> Entity {
        self.0
    }
}

pub fn add_ruby_2d(
    on: On<Add, Ruby>,
    ruby: Query<(&Ruby, &TextFont, &Transform, &TextColor), With<Text2d>>,
    commands: Commands,
) {
    if let Ok((ruby, text_font, transform, text_color)) = ruby.get(on.entity) {
        create_ruby_text_2d(
            on,
            commands,
            ruby,
            text_font,
            ruby.font_size_scale,
            transform,
            *text_color,
        );
    }
}

pub fn add_ruby_text_span_2d(
    on: On<Add, Ruby>,
    ruby: Query<&Ruby, With<TextSpan>>,
    text_config: Query<(&TextFont, &TextColor)>,
    ancestors: Query<&ChildOf>,
    text_2d: Query<&Transform, With<Text2d>>,
    commands: Commands,
) {
    if let Ok(ruby) = ruby.get(on.entity) {
        let Ok(&ChildOf(parent)) = ancestors.get(on.entity) else {
            return;
        };

        let Ok((text_font, color)) = text_config.get(on.entity) else {
            return;
        };

        let Ok(transform) = text_2d.get(parent) else {
            return;
        };

        create_ruby_text_2d(
            on,
            commands,
            ruby,
            text_font,
            ruby.font_size_scale,
            transform,
            *color,
        );
    }
}

fn create_ruby_text_2d(
    on: On<Add, Ruby>,
    mut commands: Commands,
    ruby: &Ruby,
    text_font: &TextFont,
    font_size_scale: f32,
    transform: &Transform,
    text_color: TextColor,
) {
    commands.spawn((
        RubyText2d(on.entity),
        Text2d(ruby.rt.clone()),
        ruby_text_font(text_font, font_size_scale),
        ruby.color.unwrap_or(text_color),
        // Order higher than original text
        Transform::from_translation(Vec3::new(0.0, 0.0, transform.translation.z + 0.01)),
    ));
}

fn ruby_text_font(text_font: &TextFont, font_size_scale: f32) -> TextFont {
    TextFont {
        font_size: text_font.font_size * font_size_scale,
        ..text_font.clone()
    }
}

pub fn update_ruby_text_2d(
    mut ruby_text: Query<(&RubyText2d, &mut Text2d, &mut TextFont, &mut TextColor), Without<Ruby>>,
    ruby: Query<(Ref<Ruby>, Ref<TextFont>, &TextColor)>,
) {
    for (&RubyText2d(rt_id), mut text, mut ruby_font, mut ruby_text_color) in &mut ruby_text {
        if let Ok((ruby, text_font, text_color)) = ruby.get(rt_id) {
            if ruby.is_changed() && text.0 != ruby.rt {
                text.0.clone_from(&ruby.rt);
            }

            if text_font.is_changed() {
                *ruby_font = ruby_text_font(&text_font, ruby.font_size_scale);
            }

            *ruby_text_color = ruby.color.unwrap_or(*text_color);
        }
    }
}

pub fn update_ruby_2d(
    text_layouts: Query<&TextLayoutInfo>,
    ruby_query: Query<
        (
            Entity,
            Ref<Ruby>,
            &LinkedRubyText2d,
            Option<&ChildOf>,
            Has<TextSpan>,
        ),
        Without<RubyText2d>,
    >,
    _ancestors: Query<&ChildOf>,
    mut ruby_transforms: Query<&mut Transform, (With<RubyText2d>, Without<Ruby>)>,
    text_2d_transforms: Query<&GlobalTransform, With<Text2d>>,
) {
    for (ruby_entity, ruby, &LinkedRubyText2d(rt_id), child_of, is_text_span) in &ruby_query {
        let text_entity = if is_text_span {
            let Some(&ChildOf(parent)) = child_of else {
                continue;
            };
            parent
        } else {
            ruby_entity
        };

        let Ok(layout_info) = text_layouts.get(text_entity) else {
            continue;
        };

        let Some(section_rect) = layout_info
            .section_rects
            .iter()
            .find(|&&(id, _)| id == ruby_entity)
            .map(|&(_, rect)| rect)
        else {
            continue;
        };
        let section_rect = Rect::from_corners(
            section_rect.min / layout_info.scale_factor,
            section_rect.max / layout_info.scale_factor,
        );

        let Ok(ruby_layout_info) = text_layouts.get(rt_id) else {
            continue;
        };

        let ruby_pos_local = Vec2::new(
            match ruby.align {
                RubyAlign::Start => section_rect.min.x + ruby_layout_info.size.x / 2.0,
                RubyAlign::Center => f32::midpoint(section_rect.min.x, section_rect.max.x),
                RubyAlign::End => section_rect.max.x - ruby_layout_info.size.x / 2.0,
            },
            match ruby.position {
                RubyPosition::Over => section_rect.min.y,
                RubyPosition::Under => section_rect.max.y,
            },
        );

        let Ok(mut transform) = ruby_transforms.get_mut(rt_id) else {
            continue;
        };

        let mut ruby_pos =
            ruby_pos_local.extend(transform.translation.z) - layout_info.size.extend(0.0) / 2.0;
        // Y+ down to Y+ up
        ruby_pos.y = -ruby_pos.y;

        let Ok(text_global_transform) = text_2d_transforms.get(text_entity) else {
            continue;
        };

        let ruby_pos_global = text_global_transform.transform_point(ruby_pos);

        let ruby_rotation = text_global_transform.to_scale_rotation_translation().1;

        if transform.translation == ruby_pos_global && transform.rotation == ruby_rotation {
            continue;
        }
        transform.translation = ruby_pos_global;
        transform.rotation = ruby_rotation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_ruby_creates_ruby_text_2d() {
        let mut app = App::new();
        app.add_plugins(crate::FuriganaPlugin);

        let text_entity = app
            .world_mut()
            .spawn((Ruby::new("ruby"), Text2d::new("text")))
            .id();

        let linked = app.world().get::<LinkedRubyText2d>(text_entity).unwrap();
        let ruby_text = app.world().get::<Text2d>(linked.entity()).unwrap();
        assert_eq!(ruby_text.0, "ruby");

        // UI counterpart must not be created
        assert!(
            app.world()
                .get::<crate::ui::LinkedRubyText>(text_entity)
                .is_none()
        );
    }
}
