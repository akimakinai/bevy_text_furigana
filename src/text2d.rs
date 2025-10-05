use bevy::{prelude::*, text::TextLayoutInfo};

use crate::{Ruby, RubyPosition};

/// Component for 2D ruby text entity.
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
    pub fn entity(&self) -> Entity {
        self.0
    }
}

pub(crate) fn add_ruby_2d(
    on: On<Add, Ruby>,
    ruby: Query<(&Ruby, &TextFont, &Transform), With<Text2d>>,
    commands: Commands,
) {
    if let Ok((ruby, text_font, transform)) = ruby.get(on.entity) {
        create_ruby_text_2d(
            on,
            commands,
            ruby,
            text_font,
            ruby.font_size_scale,
            transform,
        );
    }
}

pub(crate) fn add_ruby_text_span_2d(
    on: On<Add, Ruby>,
    ruby: Query<&Ruby, With<TextSpan>>,
    text_font: Query<&TextFont>,
    ancestors: Query<&ChildOf>,
    text_2d: Query<&Transform, With<Text2d>>,
    commands: Commands,
) {
    if let Ok(ruby) = ruby.get(on.entity) {
        let Ok(&ChildOf(parent)) = ancestors.get(on.entity) else {
            return;
        };

        let Ok(text_font) = text_font.get(on.entity) else {
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
) {
    commands.spawn((
        RubyText2d(on.entity),
        Text2d(ruby.rt.clone()),
        ruby_text_font(text_font, font_size_scale),
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

pub(crate) fn update_ruby_text_2d(
    mut ruby_text: Query<(&RubyText2d, &mut Text2d, &mut TextFont), Without<Ruby>>,
    ruby: Query<(Ref<Ruby>, Ref<TextFont>)>,
) {
    for (&RubyText2d(rt_id), mut text, mut ruby_font) in &mut ruby_text {
        if let Ok((ruby, text_font)) = ruby.get(rt_id) {
            if ruby.is_changed() && text.0 != ruby.rt {
                text.0 = ruby.rt.clone();
            }

            if text_font.is_changed() {
                *ruby_font = ruby_text_font(&text_font, ruby.font_size_scale);
            }
        }
    }
}

pub(crate) fn update_ruby_2d(
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
) -> Result<()> {
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

        let section_rect = layout_info
            .section_rects
            .iter()
            .find(|&&(id, _)| id == ruby_entity)
            .map(|&(_, rect)| rect)
            .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));

        let Ok(text_global_transform) = text_2d_transforms.get(text_entity) else {
            continue;
        };

        let ruby_pos_local = Vec2::new(
            (section_rect.min.x + section_rect.max.x) / 2.0,
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

        let ruby_pos_global = text_global_transform.transform_point(ruby_pos);

        let ruby_rotation = text_global_transform.to_scale_rotation_translation().1;

        if transform.translation == ruby_pos_global && transform.rotation == ruby_rotation {
            continue;
        }
        transform.translation = ruby_pos_global;
        transform.rotation = ruby_rotation;
    }

    Ok(())
}
