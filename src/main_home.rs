use crate::talks::rps_game::{RpsList, RpsModalResource, RpsModalType, RpsTimer};
use crate::{despawn_screen, Font, LastState, MainState, ResUserList, ServerResource};
use bevy::app::{App, AppExit};
use bevy::color::palettes::css::{BLACK, WHITE};
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::picking::Pickable;
use bevy::prelude::{AssetServer, Button, ChildOf, Click, Commands, Component, FlexDirection, Interaction, MessageWriter, NextState, On, OnEnter, OnExit, Out, Over, Pointer, PositionType, Query, Res, ResMut, Text, UiRect};
use bevy::text::{LineHeight, TextColor, TextFont, Underline};
use bevy::ui::{AlignItems, BackgroundColor, BorderRadius, JustifyContent, Node, Val, ZIndex};
use bevy::utils::default;
use crate::info::InfoState;

pub fn main_plugin(app: &mut App){
    app.add_systems(OnEnter(MainState::MainHome), main_setup)
        .add_systems(OnEnter(MainState::MainHome), reset_resource)
        .add_systems(OnExit(MainState::MainHome), despawn_screen::<OnMainState>);
}

#[derive(Component)]
struct OnMainState;

fn main_setup(mut commands: Commands,asset_server: Res<AssetServer>){

    let button_node = (
        Button,
        Node {
            width: Val::Px(300.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(WHITE.into()),
    );

    commands.spawn((
        Node{
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
        OnMainState
    )).with_children(|parent| {
        parent.spawn((
            Text::new("JongTalk"),
            TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 70.0,
                ..default()
            },
            TextColor(BLACK.into()),
        ));

        parent.spawn(button_node.clone()).with_child((
            Text::new("Host"),
            TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 42.0,
                ..default()
            },
            TextColor(BLACK.into())
        )).observe(|_: On<Pointer<Click>>,mut state: ResMut<NextState<MainState>>| {
            state.set(MainState::Host);
        });

        parent.spawn(button_node.clone()).with_child((
            Text::new("Join"),
            TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 42.0,
                ..default()
            },
            TextColor(BLACK.into())
        )).observe(|_: On<Pointer<Click>>,mut state: ResMut<NextState<MainState>>| {
            state.set(MainState::Join);
        });

        parent.spawn(button_node.clone()).with_child((
            Text::new("Setting"),
            TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 42.0,
                ..default()
            },
            TextColor(BLACK.into())
        )).observe(|_: On<Pointer<Click>>,mut state: ResMut<NextState<MainState>>| {
            state.set(MainState::Setting);
        });

        parent.spawn(button_node.clone()).with_child((
            Text::new("Quit"),
            TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 42.0,
                ..default()
            },
            TextColor(BLACK.into()),
        )).observe(|_: On<Pointer<Click>>,mut app_exit: MessageWriter<AppExit>| {
            app_exit.write(AppExit::Success);
        });

    });

    commands.spawn((
        Node{
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        OnMainState
    )).with_children(|p| {
        p.spawn((
            Text::new("Made by Ob_Cone"),
            TextFont{
                font: asset_server.load(Font::Medium.get()),
                font_size: 20.0,
                ..default()
            },
            TextColor(BLACK.into()),
            Underline,
        )).observe(|_:On<Pointer<Click>>,mut state: ResMut<NextState<InfoState>>|{
            println!("Click!");
            state.set(InfoState::Display);
        }).observe(|trigger:On<Pointer<Over>>,mut text_color: Query<&mut TextColor>|{
            if let Ok(mut color) = text_color.get_mut(trigger.entity){
                color.0 = WHITE.into();
            }
        }).observe(|trigger:On<Pointer<Out>>,mut text_color: Query<&mut TextColor>|{
            if let Ok(mut color) = text_color.get_mut(trigger.entity){
                color.0 = BLACK.into();
            }
        });
    });

}

fn reset_resource(
    mut user_list: ResMut<ResUserList>,
    mut rps_timer: ResMut<RpsTimer>,
    mut rps_list: ResMut<RpsList>,
    mut rps_modal_resource: ResMut<RpsModalResource>,
    mut server_resource: ResMut<ServerResource>,
){
    user_list.0.clear();
    *rps_timer = RpsTimer::None;
    rps_list.0.clear();
    rps_modal_resource.0 = RpsModalType::None;
    *server_resource = ServerResource::default();
}

pub fn get_main_home_back_button(parent: &mut RelatedSpawnerCommands<ChildOf>,asset_server: AssetServer){
    
    parent.spawn((
        Node {
            margin: UiRect::all(Val::Px(10.0)),
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    )).with_children(|parent| {

        parent.spawn((
            Text::new("<"),
            TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 70.0,
                ..default()
            },
            LineHeight::Px(55.0),
            TextColor(BLACK.into())
        )).observe(|trigger:On<Pointer<Over>>, mut text_color: Query<&mut TextColor>| {
            if let Ok(mut color) = text_color.get_mut(trigger.entity){
                color.0 = WHITE.into();
            }
        }).observe(|trigger:On<Pointer<Out>>, mut text_color: Query<&mut TextColor>| {
            if let Ok(mut color) = text_color.get_mut(trigger.entity){
                color.0 = BLACK.into();
            }
        }).observe(|_:On<Pointer<Click>>, mut state: ResMut<NextState<MainState>>, last_state: Res<LastState>| {
            if let Some(ls) = last_state.0 {
                state.set(ls);
            }
            else {
                state.set(MainState::MainHome);
            }
        });

    });
    
}
