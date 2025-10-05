use bevy::{math::Affine2, prelude::*, text::TextLayoutInfo, ui::UiSystems};

pub struct FuriganaPlugin;

impl Plugin for FuriganaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FuriganaSettings>()
            .add_systems(PostUpdate, update_ruby.after(UiSystems::Layout))
            .add_systems(PostUpdate, update_ruby_text.before(UiSystems::Layout))
            .add_observer(add_ruby)
            .add_observer(add_ruby_text_span);
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

#[derive(Component, Clone, Copy)]
#[require(Node)]
#[relationship(relationship_target = LinkedRubyText)]
pub struct RubyText(pub Entity);

/// Tracks ruby text entity corresponding to [`Ruby`].
#[derive(Component, Clone, Copy)]
#[relationship_target(relationship = RubyText, linked_spawn)]
pub struct LinkedRubyText(Entity);

impl LinkedRubyText {
    pub fn entity(&self) -> Entity {
        self.0
    }
}

fn add_ruby(
    on: On<Add, Ruby>,
    ruby: Query<(&Ruby, &TextFont, &ChildOf), With<Text>>,
    commands: Commands,
) {
    if let Ok((ruby, text_font, &ChildOf(parent))) = ruby.get(on.entity) {
        create_ruby_text(on, commands, parent, ruby, text_font, ruby.font_size_scale);
    }
}

fn add_ruby_text_span(
    on: On<Add, Ruby>,
    ruby: Query<&Ruby, With<TextSpan>>,
    text_font: Query<&TextFont>,
    ancestors: Query<&ChildOf>,
    commands: Commands,
) {
    if let Ok(ruby) = ruby.get(on.entity) {
        let Ok(&ChildOf(parent)) = ancestors.get(on.entity) else {
            return;
        };

        let Ok(text_font) = text_font.get(on.entity).or_else(|_| text_font.get(parent)) else {
            return;
        };

        let Ok(&ChildOf(grandparent)) = ancestors.get(parent) else {
            return;
        };

        create_ruby_text(
            on,
            commands,
            grandparent,
            ruby,
            text_font,
            ruby.font_size_scale,
        );
    }
}

fn create_ruby_text(
    on: On<Add, Ruby>,
    mut commands: Commands,
    parent: Entity,
    ruby: &Ruby,
    text_font: &TextFont,
    font_size_scale: f32,
) {
    let rt_id = commands
        .spawn((
            RubyText(on.entity),
            Text(ruby.rt.clone()),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            ruby_text_font(text_font, font_size_scale),
            // Initially hidden to avoid flicker before positioned
            Visibility::Hidden,
        ))
        .id();
    commands.entity(parent).add_child(rt_id);
}

fn ruby_text_font(text_font: &TextFont, font_size_scale: f32) -> TextFont {
    TextFont {
        font_size: text_font.font_size * font_size_scale,
        ..text_font.clone()
    }
}

fn update_ruby_text(
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

fn update_ruby(
    text_layouts: Query<&TextLayoutInfo>,
    mut node_query: Query<(&ComputedNode, &mut UiGlobalTransform, &mut UiTransform)>,
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
    mut ruby_nodes: Query<(&mut Node, &mut Visibility), (With<RubyText>, Without<Ruby>)>,
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

        let Ok((mut node, mut visibility)) = ruby_nodes.get_mut(rt_id) else {
            error!("No ruby text node for entity {:?}", rt_id);
            continue;
        };

        let Ok(layout_info) = text_layouts.get(node_entity) else {
            error!("No TextLayoutInfo for entity {:?}", node_entity);
            continue;
        };
        let section_rect = layout_info
            .section_rects
            .iter()
            .find(|&&(id, _)| id == text_entity)
            .map(|&(_, rect)| rect)
            .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));

        let Ok((node_computed, node_global_transform, &node_transform)) =
            node_query.get(node_entity)
        else {
            continue;
        };

        let offset = if let Ok(&ChildOf(node_parent)) = ancestors.get(node_entity)
            && let Ok((parent_computed, parent_global, ..)) = node_query.get(node_parent)
        {
            (node_global_transform.translation - node_computed.size() / 2.0)
                - (parent_global.translation - parent_computed.size() / 2.0)
        } else {
            Vec2::ZERO
        };

        let (text_scale, text_angle, _) = node_global_transform.to_scale_angle_translation();

        let ruby_pos_local = Vec2::new(
            (section_rect.min.x + section_rect.max.x) / 2.0,
            match ruby.position {
                RubyPosition::Over => section_rect.min.y,
                RubyPosition::Under => section_rect.max.y,
            },
        );

        let ruby_pos_global = node_global_transform
            .transform_point2(ruby_pos_local - node_computed.content_size() / 2.0);

        let Ok((ruby_computed_node, mut rt_global_transform, mut rt_transform)) =
            node_query.get_mut(rt_id)
        else {
            error!("No UiGlobalTransform for ruby text entity {:?}", rt_id);
            continue;
        };

        rt_transform.scale = node_transform.scale;
        rt_transform.rotation = node_transform.rotation;

        if settings.update_ui_global_transform {
            rt_global_transform.set_if_neq(UiGlobalTransform::from(
                Affine2::from_scale_angle_translation(text_scale, text_angle, ruby_pos_global),
            ));
        }

        let ruby_top_left = ruby_pos_local + offset - ruby_computed_node.size() / 2.0;
        let new_top = Val::Px(ruby_top_left.y);
        let new_left = Val::Px(ruby_top_left.x);
        if node.top != new_top {
            node.top = new_top;
        }
        if node.left != new_left {
            node.left = new_left;
        }

        visibility.set_if_neq(Visibility::Inherited);
    }

    Ok(())
}
