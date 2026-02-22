use crate::{despawn_screen, Font, MainState, ServerOffBroadcast, ServerResource, TalkState};
use bevy::app::{App, AppExit};
use bevy::asset::AssetServer;
use bevy::color::palettes::basic::WHITE;
use bevy::color::palettes::css::{BLACK, WHEAT};
use bevy::color::{Color, Srgba};
use bevy::prelude::{default, AlignItems, AppExtStates, BackgroundColor, BorderRadius, Button, Click, Commands, Component, EventWriter, FlexDirection, JustifyContent, NextState, Node, On, OnEnter, OnExit, Out, Over, Pointer, Press, Query, Release, Res, ResMut, States, Text, Trigger, Val};
use bevy::text::{TextColor, TextFont};
use bevy::ui::{Pressed, UiRect, ZIndex};


#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ModalState{
    Display,
    #[default]
    None
}

pub fn modal_plugin(app: &mut App)
{
    app.init_state::<ModalState>()
        .add_systems(OnEnter(ModalState::Display),modal)
        .add_systems(OnExit(ModalState::Display),despawn_screen::<Modal>);
}

#[derive(Component)]
struct Modal;

fn modal(mut commands: Commands, asset_server: Res<AssetServer>, server: Res<ServerResource>)
{

    let ui_layer = ZIndex(999);

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Srgba::new(1.0,1.0,1.0,0.55).into()),
        ui_layer.clone(),
        Modal
    )).with_children(|p| {

        p.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                ..default()
            },
            ui_layer.clone(),
            BorderRadius::all(Val::Px(12.0)),
            BackgroundColor(WHEAT.into()),
        )).with_children(|p| {

            let base_node = (
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ui_layer.clone(),
                BorderRadius::all(Val::Px(12.0)),
                BackgroundColor(WHITE.into()),
            );

            let base_font = (
                TextFont {
                    font: asset_server.load(Font::Bold.get()),
                    font_size: 42.0,
                    ..default()
                },
                TextColor(BLACK.into())
            );

            p.spawn(Node::default()).with_children(|p| {
                p.spawn((
                    Text::new("Addr: "),
                    TextFont {
                        font: asset_server.load(Font::Bold.get()),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(BLACK.into())
                ));
                p.spawn((
                    Node::default(),
                    Text::new(server.addr.clone()),
                    TextFont {
                        font: asset_server.load(Font::Bold.get()),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::NONE),
                    BackgroundColor(WHITE.into()),
                    ZIndex(1000)
                )).observe(|trigger: On<Pointer<Over>>,mut color: Query<&mut BackgroundColor>| {
                    if let Ok(mut color) = color.get_mut(trigger.entity){
                        color.0 = BLACK.into()
                    }
                }).observe(|trigger: On<Pointer<Out>>,mut color: Query<&mut BackgroundColor>| {
                    if let Ok(mut color) = color.get_mut(trigger.entity){
                        color.0 = WHITE.into()
                    }
                }).observe(|trigger: On<Pointer<Press>>, mut color: Query<&mut TextColor>| {
                    if let Ok(mut color) = color.get_mut(trigger.entity){
                        color.0 = WHITE.into();
                    }
                }).observe(|trigger: On<Pointer<Release>>, mut color: Query<&mut TextColor>| {
                    if let Ok(mut color) = color.get_mut(trigger.entity){
                        color.0 = Color::NONE;
                    }
                });
            });

            p.spawn(base_node.clone()).with_child((
                Text::new("Main"),
                base_font.clone()
            )).observe(|
                _: On<Pointer<Click>>,
                mut next_state: ResMut<NextState<MainState>>,
                mut next_modal_state: ResMut<NextState<ModalState>>,
                mut next_talk_state: ResMut<NextState<TalkState>>,
                tx: ResMut<ServerOffBroadcast>
            | {
                next_state.set(MainState::MainHome);
                let _ = tx.0.send(true);
                next_talk_state.set(TalkState::None);
                next_modal_state.set(ModalState::None);
            });

            p.spawn(base_node.clone()).with_child((
                Text::new("Setting"),
                base_font.clone()
            )).observe(|
                _: Trigger<Pointer<Click>>,
                mut next_state: ResMut<NextState<MainState>>,
                mut next_modal_state: ResMut<NextState<ModalState>>,
            | {
                next_state.set(MainState::Setting);
                next_modal_state.set(ModalState::None);
            });

            p.spawn(base_node.clone()).with_child((
                Text::new("Quit"),
                base_font.clone()
            )).observe(|_: Trigger<Pointer<Click>>,mut app_exit: EventWriter<AppExit>| {
                app_exit.write(AppExit::Success);
            });


        });
    });
}