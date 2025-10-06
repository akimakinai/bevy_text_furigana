use bevy::{math::Affine2, prelude::*, text::TextLayoutInfo};

use crate::{FuriganaSettings, Ruby, RubyAlign, RubyPosition};

/// Component for UI ruby text.
/// Automatically spawned when [`Ruby`] component is added along with `Text` or `TextSpan`.
#[derive(Component, Clone, Copy)]
#[require(Node)]
#[relationship(relationship_target = LinkedRubyText)]
pub struct RubyText(
    /// Entity of the corresponding `Ruby` component.
    pub Entity,
);

/// Tracks ruby text entity corresponding to [`Ruby`].
#[derive(Component, Clone, Copy)]
#[relationship_target(relationship = RubyText, linked_spawn)]
pub struct LinkedRubyText(Entity);

impl LinkedRubyText {
    pub fn entity(&self) -> Entity {
        self.0
    }
}

pub(crate) fn add_ruby(
    on: On<Add, Ruby>,
    ruby_ui: Query<(&Ruby, &TextFont, Option<&ChildOf>, &ZIndex), With<Text>>,
    commands: Commands,
) {
    if let Ok((ruby, text_font, child_of, &z_index)) = ruby_ui.get(on.entity) {
        let parent = child_of.map(ChildOf::parent);
        create_ruby_text(
            on,
            commands,
            parent,
            ruby,
            text_font,
            ruby.font_size_scale,
            z_index,
        );
    }
}

pub(crate) fn add_ruby_text_span(
    on: On<Add, Ruby>,
    ruby: Query<&Ruby, With<TextSpan>>,
    text_font: Query<&TextFont>,
    ancestors: Query<&ChildOf>,
    nodes: Query<&ZIndex, (With<Node>, With<Text>)>,
    commands: Commands,
) {
    if let Ok(ruby) = ruby.get(on.entity) {
        let Ok(&ChildOf(parent)) = ancestors.get(on.entity) else {
            return;
        };

        // ZIndex is a required component of `Node`
        let Ok(&z_index) = nodes.get(parent) else {
            // Not a UI text span
            return;
        };

        let Ok(text_font) = text_font.get(on.entity).or_else(|_| text_font.get(parent)) else {
            return;
        };

        let grandparent = ancestors.get(parent).ok().map(ChildOf::parent);

        create_ruby_text(
            on,
            commands,
            grandparent,
            ruby,
            text_font,
            ruby.font_size_scale,
            z_index,
        );
    }
}

fn create_ruby_text(
    on: On<Add, Ruby>,
    mut commands: Commands,
    parent: Option<Entity>,
    ruby: &Ruby,
    text_font: &TextFont,
    font_size_scale: f32,
    z_index: ZIndex,
) {
    let rt_id = commands
        .spawn((
            RubyText(on.entity),
            Text(ruby.rt.clone()),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            // Order higher than original text
            ZIndex(z_index.0 + 1),
            ruby_text_font(text_font, font_size_scale),
        ))
        .id();
    if let Some(parent) = parent {
        commands.entity(parent).add_child(rt_id);
    }
}

fn ruby_text_font(text_font: &TextFont, font_size_scale: f32) -> TextFont {
    TextFont {
        font_size: text_font.font_size * font_size_scale,
        ..text_font.clone()
    }
}

pub(crate) fn update_ruby_text(
    mut ruby_text: Query<(&RubyText, &mut Text, &mut TextFont), Without<Ruby>>,
    ruby: Query<(Ref<Ruby>, Ref<TextFont>)>,
) {
    for (&RubyText(rt_id), mut text, mut ruby_font) in &mut ruby_text {
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

pub(crate) fn update_ruby(
    text_layouts: Query<&TextLayoutInfo>,
    mut node_query: Query<(
        &ComputedNode,
        &mut UiGlobalTransform,
        &mut UiTransform,
        &ComputedUiRenderTargetInfo,
    )>,
    ruby_query: Query<
        (
            Entity,
            Ref<Ruby>,
            &LinkedRubyText,
            Option<&ChildOf>,
            Has<TextSpan>,
        ),
        Without<RubyText>,
    >,
    ancestors: Query<&ChildOf>,
    mut ruby_nodes: Query<&mut Node, (With<RubyText>, Without<Ruby>)>,
    settings: Res<FuriganaSettings>,
) -> Result<()> {
    for (text_entity, ruby, &LinkedRubyText(rt_id), child_of, is_text_span) in &ruby_query {
        let node_entity = if is_text_span {
            let Some(&ChildOf(parent)) = child_of else {
                continue;
            };
            parent
        } else {
            text_entity
        };

        let Ok(layout_info) = text_layouts.get(node_entity) else {
            continue;
        };
        let Some(section_rect) = layout_info
            .section_rects
            .iter()
            .find(|&&(id, _)| id == text_entity)
            .map(|&(_, rect)| rect)
        else {
            continue;
        };

        let (scale_factor, parent_global, parent_computed) = if let Ok(&ChildOf(node_parent)) =
            ancestors.get(node_entity)
            && let Ok((parent_computed, parent_global, .., parent_render_target)) =
                node_query.get(node_parent)
        {
            (
                parent_render_target.scale_factor(),
                *parent_global,
                *parent_computed,
            )
        } else {
            (1.0, UiGlobalTransform::default(), ComputedNode::default())
        };

        let Ok((&node_computed, &node_global_transform, &node_transform, _)) =
            node_query.get(node_entity)
        else {
            continue;
        };

        let Ok((ruby_computed_node, mut rt_global_transform, mut rt_transform, _)) =
            node_query.get_mut(rt_id)
        else {
            continue;
        };

        let ruby_pos_local_topleft = Vec2::new(
            match ruby.align {
                RubyAlign::Start => section_rect.min.x + ruby_computed_node.size().x / 2.0,
                RubyAlign::Center => (section_rect.min.x + section_rect.max.x) / 2.0,
                RubyAlign::End => section_rect.max.x - ruby_computed_node.size().x / 2.0,
            },
            match ruby.position {
                RubyPosition::Over => section_rect.min.y,
                RubyPosition::Under => section_rect.max.y,
            },
        );

        let ruby_pos_local = ruby_pos_local_topleft - node_computed.size() / 2.0;

        let ruby_pos_global = node_global_transform.transform_point2(ruby_pos_local);

        rt_transform.scale = node_transform.scale;
        rt_transform.rotation = node_transform.rotation;

        if settings.update_ui_global_transform {
            let (text_scale, text_angle, _) = node_global_transform.to_scale_angle_translation();

            rt_global_transform.set_if_neq(UiGlobalTransform::from(
                Affine2::from_scale_angle_translation(text_scale, text_angle, ruby_pos_global),
            ));
        }

        let Ok(mut node) = ruby_nodes.get_mut(rt_id) else {
            error!("No ruby text node for entity {:?}", rt_id);
            continue;
        };

        let ruby_top_left = parent_global.inverse().transform_point2(ruby_pos_global)
            + parent_computed.size() / 2.0
            - Vec2::new(parent_computed.border().left, parent_computed.border().top)
            - ruby_computed_node.size() / 2.0;
        let new_top = Val::Px(ruby_top_left.y / scale_factor);
        let new_left = Val::Px(ruby_top_left.x / scale_factor);
        if node.top != new_top {
            node.top = new_top;
        }
        if node.left != new_left {
            node.left = new_left;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_ruby_creates_ruby_text() {
        let mut app = App::new();
        app.add_plugins(crate::FuriganaPlugin);

        let text_entity = app
            .world_mut()
            .spawn((Ruby::new("ruby"), Text::new("text")))
            .id();

        let linked = app.world().get::<LinkedRubyText>(text_entity).unwrap();
        let ruby_text = app.world().get::<Text>(linked.entity()).unwrap();
        assert_eq!(ruby_text.0, "ruby");

        // 2D counterpart must not be created
        #[cfg(feature = "text2d")]
        assert!(
            app.world()
                .get::<crate::text2d::LinkedRubyText2d>(text_entity)
                .is_none()
        );
    }
}
