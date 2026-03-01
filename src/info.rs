use crate::scroll::ScrollComponent;
use crate::{despawn_screen, Font};
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::color::palettes::css::{BLACK, WHITE};
use bevy::color::Color;
use bevy::prelude::{AlignItems, AlignSelf, Click, Commands, Component, FlexDirection, FontWeight, JustifyContent, LinearGradient, NextState, On, OnEnter, OnExit, Overflow, Pointer, PositionType, Res, ResMut, States, Text, UiRect, Val};
use bevy::text::{TextColor, TextFont};
use bevy::ui::{BackgroundColor, BackgroundGradient, Node};
use bevy::utils::default;
use cargo_toml::{Dependency, Manifest};
use std::f32::consts::TAU;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum InfoState{
    #[default]
    None,
    Display
}
#[derive(Component)]
struct OnInfoState;
pub fn info_plugin(app: &mut App){
    app.add_systems(OnEnter(InfoState::Display), setup)
        .add_systems(OnExit(InfoState::Display), despawn_screen::<OnInfoState>);
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
){
    let bytes = include_bytes!("../Cargo.toml");
    let manifest = Manifest::from_slice(bytes).unwrap();
    let package = manifest.package.unwrap();

    let out = |_:On<Pointer<Click>>,mut state: ResMut<NextState<InfoState>>| {
        state.set(InfoState::None);

    };
    let fill = 40.0;
    let not_fill = (100.0 - fill)/2.0;
    commands.spawn((
        Node {
            width: Val::Percent(not_fill),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundGradient::from(LinearGradient {
            angle: TAU / 4.0,
            stops: vec![
                Color::NONE.into(),
                BLACK.into(),
            ],
            ..default()
        }),
        OnInfoState
    )).observe(out);

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::ZERO,
            width: Val::Percent(not_fill),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundGradient::from(LinearGradient {
            angle: TAU / 4.0,
            stops: vec![
                BLACK.into(),
                Color::NONE.into(),
            ],
            ..default()
        }),
        OnInfoState
    )).observe(out);

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(not_fill),
            width: Val::Percent(fill),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(BLACK.into()),
        OnInfoState
    )).with_children(|p| {
        let ver = if let Ok(ver) = package.version.get(){
            ver
        }else { "?.?.?" };
        let title = format!("Jong Talk v{}",ver);
        p.spawn((
            Text(title),
            TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 35.0,
                weight: FontWeight::BOLD,
                ..default()
            }
        )).observe(|_:On<Pointer<Click>>| {
            let _ = open::that("https://github.com/Ob-cone/jong_talk");
        });

        p.spawn(Node {
            width: Val::Percent(80.0),
            height: Val::Percent(60.0),
            margin: UiRect::new(Val::ZERO,Val::ZERO,Val::Px(60.0),Val::Px(60.0)),
            overflow: Overflow::hidden(),
            ..default()
        }).with_children(|p| {
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    top: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                ScrollComponent::Top
            )).with_children(|p| {
                let title_node = Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                };
                let title_font = (
                    TextFont {
                        font: asset_server.load(Font::Bold.get()),
                        font_size: 30.0,
                        weight: FontWeight::BOLD,
                        ..default()
                    },
                    TextColor(WHITE.into())
                );
                let info_node = Node {
                    width: Val::Percent(80.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                };

                let info_font = (
                    TextFont {
                        font: asset_server.load(Font::Medium.get()),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(WHITE.into())
                );

                p.spawn(title_node.clone()).with_child((
                    Text("Team".to_string()),
                    title_font.clone()
                ));

                p.spawn(info_node.clone()).with_children(|p| {
                    p.spawn((Text("Planning".to_string()), info_font.clone()));
                    p.spawn((Text("Ob-Cone".to_string()), info_font.clone()));
                });
                p.spawn(info_node.clone()).with_children(|p| {
                    p.spawn((Text("Programming".to_string()), info_font.clone()));
                    p.spawn((Text("Ob-Cone".to_string()), info_font.clone()));
                });
                p.spawn(info_node.clone()).with_children(|p| {
                    p.spawn((Text("Design".to_string()), info_font.clone()));
                    p.spawn((Text("Ob-Cone".to_string()), info_font.clone()));
                });

                p.spawn(title_node.clone()).with_child((
                    Text(" ".to_string()),
                    title_font.clone()
                ));

                p.spawn(title_node.clone()).with_child((
                    Text("Font".to_string()),
                    title_font.clone()
                ));

                p.spawn(info_node.clone()).with_children(|p| {
                    p.spawn((Text("PAPERLOGY".to_string()), info_font.clone()));
                    p.spawn((Text("LEE_JUIM".to_string()), info_font.clone()));
                }).observe(|_:On<Pointer<Click>>| {
                    let _ = open::that("https://freesentation.blog/paperlogyfont");
                });

                p.spawn(title_node.clone()).with_child((
                    Text(" ".to_string()),
                    title_font.clone()
                ));


                p.spawn(title_node.clone()).with_child((
                    Text("Special Thanks".to_string()),
                    title_font.clone()
                ));

                for (name, dep) in &manifest.dependencies {
                    let ver = match dep {
                        Dependency::Simple(info) => {info.clone()}
                        Dependency::Inherited(_) => {"None".to_string()}
                        Dependency::Detailed(info) => {
                            match &info.version {
                                None => {"None".to_string()}
                                Some(ver) => {ver.clone()}
                            }
                        }
                    };
                    println!("{:?}",name);
                    let mut entity = p.spawn(info_node.clone());
                    entity.with_children(|p| {
                        p.spawn((Text(name.to_string()), info_font.clone()));
                        p.spawn((Text(ver.clone()), info_font.clone()));
                    });

                    if ver != "None".to_string(){
                        let url_name = name.clone();
                        entity.observe(move |_:On<Pointer<Click>>| {
                            let url = format!("https://crates.io/crates/{}",url_name);
                            let _ = open::that(url);
                        });
                    }

                }
            });
        });
    });
}