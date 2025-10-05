use bevy::{
    math::Affine2,
    prelude::*,
    text::{Text2dUpdateSystems, TextLayoutInfo},
    ui::UiSystems,
};

pub struct FuriganaPlugin;

impl Plugin for FuriganaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FuriganaSettings>()
            .add_systems(PostUpdate, update_ruby.after(UiSystems::Layout))
            .add_systems(PostUpdate, update_ruby_text.before(UiSystems::Layout))
            .add_systems(PostUpdate, update_ruby_2d.before(Text2dUpdateSystems))
            .add_systems(PostUpdate, update_ruby_text_2d.after(Text2dUpdateSystems))
            .add_observer(add_ruby)
            .add_observer(add_ruby_text_span)
            .add_observer(add_ruby_2d)
            .add_observer(add_ruby_text_span_2d);
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

/// Component for UI ruby text entity.
#[derive(Component, Clone, Copy)]
#[require(Node)]
#[relationship(relationship_target = LinkedRubyText)]
pub struct RubyText(
    /// Entity of the corresponding `Ruby` component.
    pub Entity,
);

/// Component for 2D ruby text entity.
#[derive(Component, Clone, Copy)]
#[relationship(relationship_target = LinkedRubyText2d)]
pub struct RubyText2d(
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

/// Tracks ruby text entity corresponding to [`Ruby`] for 2D text.
#[derive(Component, Clone, Copy)]
#[relationship_target(relationship = RubyText2d, linked_spawn)]
pub struct LinkedRubyText2d(Entity);

impl LinkedRubyText2d {
    pub fn entity(&self) -> Entity {
        self.0
    }
}

fn add_ruby(
    on: On<Add, Ruby>,
    ruby_ui: Query<(&Ruby, &TextFont, &ChildOf), With<Text>>,
    commands: Commands,
) {
    if let Ok((ruby, text_font, &ChildOf(parent))) = ruby_ui.get(on.entity) {
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

fn add_ruby_2d(
    on: On<Add, Ruby>,
    ruby: Query<(&Ruby, &TextFont), With<Text2d>>,
    commands: Commands,
) {
    if let Ok((ruby, text_font)) = ruby.get(on.entity) {
        create_ruby_text_2d(on, commands, ruby, text_font, ruby.font_size_scale);
    }
}

fn add_ruby_text_span_2d(
    on: On<Add, Ruby>,
    ruby: Query<&Ruby, With<TextSpan>>,
    text_font: Query<&TextFont>,
    ancestors: Query<&ChildOf>,
    text_2d: Query<(), With<Text2d>>,
    commands: Commands,
) {
    if let Ok(ruby) = ruby.get(on.entity) {
        let Ok(&ChildOf(parent)) = ancestors.get(on.entity) else {
            return;
        };

        let Ok(text_font) = text_font.get(on.entity) else {
            return;
        };

        if text_2d.get(parent).is_err() {
            return;
        }

        create_ruby_text_2d(on, commands, ruby, text_font, ruby.font_size_scale);
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
        ))
        .id();
    commands.entity(parent).add_child(rt_id);
}

fn create_ruby_text_2d(
    on: On<Add, Ruby>,
    mut commands: Commands,
    ruby: &Ruby,
    text_font: &TextFont,
    font_size_scale: f32,
) {
    commands.spawn((
        RubyText2d(on.entity),
        Text2d(ruby.rt.clone()),
        ruby_text_font(text_font, font_size_scale),
    ));
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

fn update_ruby_text_2d(
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

fn update_ruby(
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
            error!("No TextLayoutInfo for entity {:?}", node_entity);
            continue;
        };
        let section_rect = layout_info
            .section_rects
            .iter()
            .find(|&&(id, _)| id == text_entity)
            .map(|&(_, rect)| rect)
            .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));

        let Ok((node_computed, node_global_transform, &node_transform, _)) =
            node_query.get(node_entity)
        else {
            continue;
        };

        let (offset, scale_factor);
        if let Ok(&ChildOf(node_parent)) = ancestors.get(node_entity)
            && let Ok((parent_computed, parent_global, .., parent_render_target)) =
                node_query.get(node_parent)
        {
            offset = (node_global_transform.translation - node_computed.size() / 2.0)
                - (parent_global.translation - parent_computed.size() / 2.0)
                // I don't know why but need to subtract border
                - Vec2::new(parent_computed.border().left, parent_computed.border().top);
            scale_factor = parent_render_target.scale_factor();
        } else {
            offset = Vec2::ZERO;
            scale_factor = 1.0;
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

        let Ok((ruby_computed_node, mut rt_global_transform, mut rt_transform, _)) =
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

        let Ok(mut node) = ruby_nodes.get_mut(rt_id) else {
            error!("No ruby text node for entity {:?}", rt_id);
            continue;
        };

        let ruby_top_left = ruby_pos_local + offset - ruby_computed_node.size() / 2.0;
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

fn update_ruby_2d(
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
            error!("No TextLayoutInfo for entity {:?}", text_entity);
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

        let mut ruby_pos = ruby_pos_local.extend(0.0) - layout_info.size.extend(0.0) / 2.0;
        // Y+ down to Y+ up
        ruby_pos.y = -ruby_pos.y;

        let ruby_pos_global = text_global_transform.transform_point(ruby_pos);

        let Ok(mut transform) = ruby_transforms.get_mut(rt_id) else {
            error!("No Transform for ruby text 2d entity {:?}", rt_id);
            continue;
        };

        let ruby_rotation = text_global_transform.to_scale_rotation_translation().1;

        if transform.translation == ruby_pos_global && transform.rotation == ruby_rotation {
            continue;
        }
        transform.translation = ruby_pos_global;
        transform.rotation = ruby_rotation;
    }

    Ok(())
}
