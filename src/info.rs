use std::f32::consts::TAU;
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::color::palettes::css::{BLACK, RED, WHITE};
use bevy::prelude::{AlignItems, Click, Commands, Component, FlexDirection, JustifyContent, LinearGradient, NextState, On, OnEnter, OnExit, Pointer, PositionType, Res, ResMut, States, Text, Val};
use bevy::text::TextFont;
use bevy::ui::{BackgroundColor, BackgroundGradient, Gradient, Node};
use bevy::utils::default;
use cargo_toml::{Dependency, Manifest};
use crate::{despawn_screen, Font, MainState};

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
    println!("이름: {}", package.name);
    println!("버전: {:?}", package.version);

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
        println!("{}: {}",name,ver);
    }
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
            row_gap: Val::Px(15.0),
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
                font_size: 30.0,
                ..default()
            }
        )).observe(|_:On<Pointer<Click>>| {
            let _ = open::that("https://github.com/Ob-cone/jong_talk");
        });

        p.spawn((
            Node {
                width: Val::Percent(80.0),
                height: Val::Percent(75.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(WHITE.into())
        ));
    });
}