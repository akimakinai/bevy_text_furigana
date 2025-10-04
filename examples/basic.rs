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
        .add_systems(Update, update_ui_rotator)
        .run();
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    let font = assets.load(r"C:\Windows\Fonts\meiryo.ttc");
    let text_font = TextFont {
        font: font.clone(),
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
                    },
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
                padding: UiRect::all(px(10.0)),
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
                        rt: "とし".into(),
                        ..default()
                    },
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
                        RubyPosition::Above,
                    );
                });

            parent.spawn((
                Text("Lorem ipsum dolor sit amet".into()),
                text_font.clone(),
                Ruby {
                    rt: "consectetur adipiscing elit".into(),
                    ..default()
                },
            ));

            parent
                .spawn((
                    Text(String::new()),
                    text_font.clone(),
                    UiRotator(0.0),
                    Node {
                        margin: UiRect::top(px(100.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((TextSpan::new("とある"), text_font.clone()));
                    parent.spawn((
                        TextSpan::new("科学"),
                        text_font.clone(),
                        Ruby {
                            rt: "かがく".into(),
                            position: RubyPosition::Above,
                        },
                    ));
                    parent.spawn((TextSpan::new("の\n"), text_font.clone()));
                    parent.spawn((
                        TextSpan::new("超電磁砲"),
                        text_font.clone(),
                        Ruby {
                            rt: "レールガン".into(),
                            position: RubyPosition::Below,
                        },
                    ));
                });
        });

    commands.spawn((Camera2d, Camera::default()));
}

#[derive(Component)]
struct UiRotator(f32);

fn update_ui_rotator(mut query: Query<(&mut UiTransform, &mut UiRotator)>, time: Res<Time>) {
    for (mut ui_transform, mut rotator) in &mut query {
        rotator.0 += time.delta_secs() * 15.0;
        ui_transform.rotation = Rot2::degrees(rotator.0);
    }
}
