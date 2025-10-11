use bevy::{
    asset::UnapprovedPathMode, ecs::relationship::RelatedSpawnerCommands, prelude::*,
    text::LineHeight, window::WindowResolution,
};

use bevy_text_furigana::*;

fn main() {
    let scale_factor = 1.5;
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::default()
                            .with_scale_factor_override(scale_factor),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(FuriganaPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, rotate_text)
        .add_observer(drag_line)
        .run();
}

#[derive(Component)]
#[require(ShowAabbGizmo)]
struct Draggable;

fn drag_line(
    on: On<Pointer<Drag>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    aabbs: Query<(Entity, &bevy::camera::primitives::Aabb, &GlobalTransform), With<Draggable>>,
    mut transform: Query<&mut Transform>,
) -> Result<()> {
    let (camera, camera_transform) = cameras.single()?;

    let pointer = camera.viewport_to_world_2d(camera_transform, on.pointer_location.position)?;

    let origin = camera.world_to_viewport(camera_transform, Vec3::ZERO)?;
    let world_delta = camera.viewport_to_world_2d(camera_transform, origin + on.delta)?;

    for (id, aabb, g_transform) in &aabbs {
        let aabb = bevy::camera::primitives::Aabb {
            center: aabb.center + g_transform.translation().to_vec3a(),
            half_extents: aabb.half_extents,
        };
        if aabb.min().x <= pointer.x
            && pointer.x <= aabb.max().x
            && aabb.min().y <= pointer.y
            && pointer.y <= aabb.max().y
        {
            let mut transform = transform.get_mut(id)?;
            transform.translation.x += world_delta.x;
            transform.translation.y += world_delta.y;
        }
    }

    Ok(())
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    let font = assets.load(r"C:\Windows\Fonts\meiryo.ttc");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 32.0,
        line_height: LineHeight::RelativeToFont(1.6),
        ..default()
    };

    let ruby_spans = |spawner: &mut RelatedSpawnerCommands<ChildOf>,
                      arr: &[(&str, Option<&str>)],
                      position: RubyPosition| {
        for &(text, rt) in arr {
            if let Some(rt) = rt {
                spawner.spawn((
                    TextSpan::new(text),
                    text_font.clone(),
                    Ruby {
                        rt: rt.into(),
                        position,
                        ..default()
                    },
                ));
            } else {
                spawner.spawn((TextSpan::new(text), text_font.clone()));
            }
        }
    };

    // Simple Text2d with ruby
    commands.spawn((
        Text2d("Lorem ipsum dolor sit amet".into()),
        text_font.clone(),
        Ruby {
            rt: "consectetur adipiscing elit".into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
    ));

    // Text2d with TextSpan children
    commands
        .spawn((
            Text2d::new("年紀"),
            text_font.clone(),
            Ruby {
                rt: "とし".into(),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 200.0, 0.0)),
        ))
        .with_children(|parent| {
            ruby_spans(
                parent,
                &[
                    // Sampled from 高野聖
                    ("は若し、お", None),
                    ("前様", Some("まえさん")),
                    ("、", None),
                    ("私", Some("わし")),
                    ("は", None),
                    ("真赤", Some("まっか")),
                    ("になった、手に汲んだ川の水を飲みかねて", None),
                    ("猶予", Some("ためら")),
                    ("っているとね。\n", None),
                    ("そうすれば上段の", None),
                    ("室", Some("へや")),
                    ("に寝かして一晩", None),
                    ("扇", Some("あお")),
                    ("いでいてそれで", None),
                    ("功徳", Some("くどく")),
                    ("のためにする家があると", None),
                    ("承", Some("うけたまわ")),
                    ("りましても、\n", None),
                    // Sampled from 大岡政談
                    ("下野國", Some("しもつけのくに")),
                    ("日光山", Some("につくわうざん")),
                    ("に", None),
                    ("鎭座", Some("ちんざ")),
                    ("まします", None),
                    ("東照大神", Some("とうせうだいじん")),
                    ("より第八代の", None),
                    ("將軍", Some("しやうぐん")),
                    ("有徳院吉宗公", Some("いうとくゐんよしむねこう")),
                    ("と", None),
                    ("稱", Some("しよう")),
                    ("し", None),
                    ("奉", Some("たてま")),
                    ("つるは", None),
                    ("東照神君", Some("とうせうしんくん")),
                    ("の", None),
                ],
                RubyPosition::Over,
            );
        });

    // Text2d with ruby under
    commands.spawn((
        Text2d("超電磁砲".into()),
        TextBackgroundColor(bevy::color::palettes::css::ORANGE.into()),
        text_font.clone(),
        Ruby {
            rt: "レールガン".into(),
            position: RubyPosition::Under,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        TextRotator(0.0),
    ));

    commands.spawn((
        Draggable,
        Text2d::default(),
        text_font.clone(),
        Transform::from_translation(Vec3::new(0.0, -200.0, 0.0)),
        children![
            // Sampled from 探検実記
            (
                TextSpan::new("幻花翁"),
                text_font.clone(),
                TextColor(bevy::color::palettes::css::GREEN.into()),
                Ruby {
                    rt: "げんくわおう".into(),
                    align: RubyAlign::Start,
                    ..default()
                },
            ),
            (TextSpan::new("、"), text_font.clone()),
            (
                TextSpan::new("望蜀生"),
                text_font.clone(),
                Ruby {
                    rt: "ぼうしよくせい".into(),
                    align: RubyAlign::Center,
                    ..default()
                },
            ),
            (TextSpan::new("、"), text_font.clone()),
            (
                TextSpan::new("玄川子"),
                text_font.clone(),
                Ruby {
                    rt: "げんせんし".into(),
                    align: RubyAlign::End,
                    ..default()
                },
            ),
        ],
    ));

    commands.spawn((
        Camera2d,
        Camera::default(),
        Transform::from_translation(Vec3::new(0.0, 200.0, 0.0))
            .with_rotation(Quat::from_rotation_z(-15.0f32.to_radians()))
            .with_scale(Vec3::new(1.2, 1.2, 1.0)),
    ));
}

#[derive(Component)]
struct TextRotator(f32);

fn rotate_text(mut query: Query<(&mut Transform, &mut TextRotator)>, time: Res<Time>) {
    for (mut transform, mut rotator) in &mut query {
        rotator.0 += time.delta_secs() * 30.0;
        transform.rotation = Quat::from_rotation_z(rotator.0.to_radians());
    }
}
