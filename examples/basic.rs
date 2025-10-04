use bevy::{
    asset::UnapprovedPathMode,
    ecs::relationship::RelatedSpawnerCommands,
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
    text::LineHeight,
};

use bevy_text_furigana::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                })
                .set(LogPlugin {
                    filter: format!("{},{}=debug", DEFAULT_FILTER, env!("CARGO_PKG_NAME")),
                    ..default()
                }),
        )
        .add_plugins(FuriganaPlugin)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    let font = assets.load(r"C:\Windows\Fonts\meiryo.ttc");
    let text_font = TextFont {
        font: font.clone(),
        line_height: LineHeight::RelativeToFont(1.6),
        ..default()
    };

    let ruby_spans = |spawner: &mut RelatedSpawnerCommands<ChildOf>, arr: &[(&str, Option<&str>)]| {
        for &(text, rt) in arr {
            if let Some(rt) = rt {
                spawner.spawn((
                    TextSpan::new(text),
                    text_font.clone(),
                    Ruby { rt: rt.into() },
                ));
            } else {
                spawner.spawn(TextSpan::new(text));
            }
        }
    };

    commands
        .spawn((
            Node {
                width: percent(50.0),
                height: percent(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Name::new("Root"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Text("年紀".into()),
                    text_font.clone(),
                    Ruby {
                        rt: "とし".into()
                    },
                ))
                .with_children(|parent| {
                    ruby_spans(
                        parent,
                        &[
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
                            ("りましても、", None),
                        ],
                    );
                });

            parent
                .spawn((
                    Text(String::new()),
                    text_font.clone(),
                    UiTransform::from_rotation(Rot2::degrees(45.0)),
                    Node {
                        margin: UiRect::top(px(100.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    ruby_spans(
                        parent,
                        &[
                            ("とある", None),
                            ("科学", Some("かがく")),
                            ("の\n", None),
                            ("超電磁砲", Some("レールガン")),
                        ],
                    );
                });
        });

    commands.spawn((Camera2d, Camera::default()));
}
