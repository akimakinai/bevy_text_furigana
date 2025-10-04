use bevy::{
    math::Affine2,
    prelude::*,
    text::{LineHeight, TextLayoutInfo},
    ui::UiSystems,
};

pub struct FuriganaPlugin;

impl Plugin for FuriganaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_furigana.after(UiSystems::Layout))
            .add_observer(add_ruby)
            .add_observer(add_ruby_text_span)
            .add_observer(remove_ruby);
    }
}

#[derive(Component, Clone)]
#[require(RubyText(Entity::PLACEHOLDER))]
pub struct Ruby {
    pub rt: String,
}

#[derive(Component, Clone, Copy)]
#[require(Node)]
pub struct RubyTextOf(pub Entity);

#[derive(Component, Clone, Copy)]
pub struct RubyText(pub Entity);

fn add_ruby(
    on: On<Add, Ruby>,
    ruby: Query<(&Ruby, &TextFont, &ChildOf), With<Text>>,
    commands: Commands,
) {
    if let Ok((ruby, text_font, &ChildOf(parent))) = ruby.get(on.entity) {
        create_ruby_text(on, commands, parent, ruby, text_font);
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

        create_ruby_text(on, commands, grandparent, ruby, text_font);
    }
}

fn create_ruby_text(
    on: On<Add, Ruby>,
    mut commands: Commands,
    parent: Entity,
    ruby: &Ruby,
    text_font: &TextFont,
) {
    let rt_id = commands
        .spawn((
            RubyTextOf(on.entity),
            Text(ruby.rt.clone()),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            ruby_text_font(text_font),
            // Initially hidden to avoid flicker before positioned
            Visibility::Hidden,
        ))
        .id();
    commands.entity(parent).add_child(rt_id);
    commands.entity(on.entity).insert(RubyText(rt_id));
}

fn remove_ruby(on: On<Remove, Ruby>, ruby_text: Query<&RubyText>, mut commands: Commands) {
    if let Ok(ruby_text) = ruby_text.get(on.entity) {
        let id = ruby_text.0;
        if id != Entity::PLACEHOLDER {
            commands.entity(id).try_despawn();
        }
    }
}

const RUBY_FONT_SIZE_SCALE: f32 = 0.5;

fn ruby_text_font(text_font: &TextFont) -> TextFont {
    TextFont {
        font_size: text_font.font_size * RUBY_FONT_SIZE_SCALE,
        ..text_font.clone()
    }
}

fn update_furigana(
    text_layouts: Query<(&TextLayoutInfo, Ref<TextFont>), Without<RubyTextOf>>,
    mut node_query: Query<(&ComputedNode, &mut UiGlobalTransform, &mut UiTransform)>,
    query: Query<
        (
            Entity,
            Ref<Ruby>,
            &RubyText,
            Option<Ref<TextFont>>,
            Option<&ChildOf>,
            Has<TextSpan>,
        ),
        Without<RubyTextOf>,
    >,
    mut ruby_nodes: Query<
        (
            &RubyTextOf,
            &ComputedNode,
            &mut Text,
            &mut Node,
            &mut TextFont,
            &mut Visibility,
        ),
        Without<Ruby>,
    >,
    ancestors: Query<&ChildOf>,
) -> Result<()> {
    for (entity, ruby, &RubyText(rt_id), text_font, child_of, is_text_span) in &query {
        let node_entity = if is_text_span {
            if let Some(&ChildOf(parent)) = child_of {
                parent
            } else {
                continue;
            }
        } else {
            entity
        };
        let Ok((layout_info, node_text_font)) = text_layouts.get(node_entity) else {
            error!("No TextLayoutInfo for entity {:?}", node_entity);
            continue;
        };

        let Ok((_, ruby_computed_node, mut text, mut node, mut ruby_font, mut visibility)) =
            ruby_nodes.get_mut(rt_id)
        else {
            error!("No Text for entity {:?}", rt_id);
            continue;
        };

        if ruby.is_changed() {
            text.0 = ruby.rt.clone();
        }

        let text_font = text_font.unwrap_or(node_text_font);

        if text_font.is_changed() {
            *ruby_font = ruby_text_font(&text_font);
        }

        let text_node_rect = node_query
            .get(node_entity)
            .map(|(computed_node, ui_transform, _)| global_rect(computed_node, ui_transform))?;

        let parent_rect = if let Ok(&ChildOf(node_parent)) = ancestors.get(node_entity) {
            let (computed_node, ui_transform, _) = node_query.get(node_parent)?;
            global_rect(computed_node, ui_transform)
        } else {
            Rect {
                min: Vec2::ZERO,
                max: Vec2::ZERO,
            }
        };

        let section_rect = layout_info
            .section_rects
            .iter()
            .find(|&&(id, _)| id == entity)
            .map(|&(_, rect)| rect)
            .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));

        let line_height_corr = |rect_height| match text_font.line_height {
            LineHeight::RelativeToFont(factor) => rect_height / factor,
            LineHeight::Px(px) => rect_height - px,
        };

        let Ok((_, &text_ui_transform, _)) = node_query.get(node_entity) else {
            continue;
        };

        let (text_scale, text_angle, _) = text_ui_transform.to_scale_angle_translation();

        let section_pos = Vec2::new(
            (section_rect.min.x + section_rect.max.x) / 2.0,
            section_rect.min.y
                - (line_height_corr(section_rect.max.y - section_rect.min.y)
                    * RUBY_FONT_SIZE_SCALE),
        )
        .rotate(Vec2::from_angle(text_angle));

        let new_ui_x = text_node_rect.min.x - (ruby_computed_node.size().x / 2.0) + section_pos.x;
        let new_ui_y = text_node_rect.min.y + section_pos.y;

        let Ok((_, mut rt_global_transform, mut rt_transform)) = node_query.get_mut(rt_id) else {
            error!("No UiGlobalTransform for entity {:?}", rt_id);
            continue;
        };

        rt_transform.scale = text_scale;
        rt_transform.rotation = Rot2::radians(text_angle);

        // Update GlobalUiTransform to erase one-frame delay
        rt_global_transform.set_if_neq(UiGlobalTransform::from(
            Affine2::from_scale_angle_translation(
                text_scale,
                text_angle,
                Vec2::new(new_ui_x, new_ui_y) + ruby_computed_node.size() / 2.0,
            ),
        ));

        let new_top = Val::Px(new_ui_y - parent_rect.min.y);
        let new_left = Val::Px(new_ui_x - parent_rect.min.x);
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

fn global_rect(node: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    // from bevy_ui_render/src/ui_material_pipeline.rs
    let uinode_rect = Rect {
        min: Vec2::ZERO,
        max: node.size(),
    };
    let rect_size = uinode_rect.size();

    Rect {
        min: transform.transform_point2(Vec2::splat(-0.5) * rect_size),
        max: transform.transform_point2(Vec2::splat(0.5) * rect_size),
    }
}
