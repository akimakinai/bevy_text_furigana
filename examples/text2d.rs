use bevy::{
    asset::UnapprovedPathMode, ecs::relationship::RelatedSpawnerCommands, prelude::*,
    text::LineHeight,
};

use bevy_text_furigana::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            unapproved_path_mode: UnapprovedPathMode::Allow,
            ..default()
        }))
        .add_plugins(FuriganaPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, rotate_text)
        .run();
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
                        font_size_scale: 0.5,
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
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
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
        Transform::from_translation(Vec3::new(0.0, -200.0, 0.0)),
        TextRotator(0.0),
    ));

    commands.spawn((Camera2d, Camera::default()));
}

#[derive(Component)]
struct TextRotator(f32);

fn rotate_text(mut query: Query<(&mut Transform, &mut TextRotator)>, time: Res<Time>) {
    for (mut transform, mut rotator) in &mut query {
        rotator.0 += time.delta_secs() * 30.0;
        transform.rotation = Quat::from_rotation_z(rotator.0.to_radians());
    }
}
