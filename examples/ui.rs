use bevy::{
    asset::UnapprovedPathMode,
    ecs::relationship::RelatedSpawnerCommands,
    input::keyboard::Key,
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
        .add_systems(Update, (update_ui_rotator, settings))
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
                        font_size_scale: 0.5,
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
                        RubyPosition::Over,
                    );
                });

            parent.spawn((
                Text::new("Lorem ipsum dolor sit amet\n"),
                text_font.clone(),
                Ruby {
                    rt: "consectetur adipiscing elit".into(),
                    ..default()
                },
            ));

            parent.spawn((
                Text::default(),
                text_font.clone(),
                children![(
                    TextSpan::new("超電磁砲"),
                    text_font.clone(),
                    Ruby {
                        rt: "レールガン".into(),
                        position: RubyPosition::Under,
                        font_size_scale: 0.8,
                    },
                )],
            ));

            parent
                .spawn((
                    Node {
                        border: UiRect::all(Val::Px(10.0)),
                        margin: UiRect::top(px(100.0)),
                        padding: UiRect::all(px(10.0)),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    UiRotator(0.0),
                    BorderColor::all(Color::BLACK),
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Text::default(),
                            TextLayout::new_with_justify(Justify::Center),
                            text_font.clone(),
                        ))
                        .with_children(|parent| {
                            // Sampled from 探検実記
                            ruby_spans(
                                parent,
                                &[
                                    ("幻花翁、\n", Some("げんくわおう")),
                                    ("望蜀生、\n", Some("ぼうしよくせい")),
                                    ("玄川子", Some("げんせんし")),
                                ],
                                RubyPosition::Over,
                            );
                        });

                    // FIXME: these are broken
                    parent
                        .spawn((
                            Text::new("品川\n"),
                            text_font.clone(),
                            Ruby {
                                rt: "しながは".into(),
                                ..default()
                            },
                            Node {
                                margin: UiRect::left(px(20)),
                                ..default()
                            },
                        ))
                        .with_child((
                            TextSpan::new("下末吉村"),
                            text_font.clone(),
                            Ruby {
                                rt: "しもすゑよしむら".into(),
                                ..default()
                            },
                        ));
                });
        });

    commands
        .spawn((
            Text::new("ルート"),
            Ruby {
                rt: "Root".into(),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(150.0),
                right: Val::Px(5.0),
                ..default()
            },
            text_font.clone(),
            UiTransform::from_rotation(Rot2::degrees(90.0)),
        ))
        .with_children(|parent| {
            ruby_spans(
                parent,
                &[("テキスト", Some("Text")), ("ノード", Some("Node"))],
                RubyPosition::Over,
            );
        });

    commands.spawn((Camera2d, Camera::default()));
}

#[derive(Component)]
struct UiRotator(f32);

fn update_ui_rotator(mut query: Query<(&mut UiTransform, &mut UiRotator)>, time: Res<Time>) {
    for (mut ui_transform, mut rotator) in &mut query {
        rotator.0 += time.delta_secs() * 30.0;
        ui_transform.rotation = Rot2::degrees(rotator.0);
    }
}

fn settings(key: Res<ButtonInput<Key>>, mut configs: ResMut<FuriganaSettings>) {
    if key.just_pressed(Key::Character("u".into())) {
        configs.update_ui_global_transform = !configs.update_ui_global_transform;
        info!(
            "update_ui_global_transform: {}",
            configs.update_ui_global_transform
        );
    }
}
