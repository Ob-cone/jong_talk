use crate::{despawn_screen, BasicInfos, Font, WriteMpsc};
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::color::palettes::basic::{BLACK, RED, WHITE};
use bevy::color::palettes::css::{GRAY, WHEAT};
use bevy::color::Srgba;
use bevy::prelude::{default, in_state, AlignItems, AppExtStates, BackgroundColor, BorderRadius, Button, Click, Commands, Component, FlexDirection, ImageNode, IntoScheduleConfigs, JustifyContent, NextState, Node, On, OnEnter, OnExit, Out, Over, Pointer, Query, Res, ResMut, Resource, Single, States, Text, TextColor, TextFont, UiRect, Update, Val, With, ZIndex};
use bevy::text::TextSpan;
use rand::random_range;
use server_lib::{Data, DataType, DataTypeKind, RPSType};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;


#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum RpsState{
    Display,
    #[default]
    None
}

pub fn rps_plugin(app: &mut App)
{
    app.init_state::<RpsState>()
        .add_systems(OnEnter(RpsState::Display),rps_modal)
        .add_systems(Update,accept_timer.run_if(in_state(RpsState::Display)))
        .add_systems(Update,game_timer.run_if(in_state(RpsState::Display)))
        .add_systems(OnExit(RpsState::Display),despawn_screen::<Rps>);
}

#[derive(Component)]
struct Rps;
#[derive(Resource,Clone,PartialEq)]
pub struct RpsList(pub HashMap<Vec<u8>,(String,String)>);
#[derive(Resource,Clone)]
pub struct RpsModalResource(pub RpsModalType);
#[derive(Resource)]
pub enum RpsTimer{
    None,
    Accept(Instant),
    Game(Instant),
}
#[derive(Clone)]
pub enum RpsModalType{
    Send(String),
    Accept(Vec<u8>,String),
    Game(Vec<u8>),
    None
}

#[derive(Component)]
struct GameTimeText;

fn rps_modal(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_rps_state: ResMut<NextState<RpsState>>,
    mut rps_timer: ResMut<RpsTimer>,
    rps_modal_type: Res<RpsModalResource>
){
    if let RpsModalType::None = rps_modal_type.0{
        next_rps_state.set(RpsState::None);
        return;
    }

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
        Rps
    )).with_children(|p| {
        match rps_modal_type.0.clone() {
            RpsModalType::Send(_) => {
                p.spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Auto,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    ui_layer.clone(),
                    BackgroundColor(WHEAT.into()),
                )).with_children(|p| {
                    p.spawn((
                        Text("※WARNING※".to_string()),
                        TextFont{
                            font: asset_server.load(Font::Bold.get()),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(RED.into())
                    ));
                    p.spawn((
                        Text(String::new()),
                    )).with_children(|t| {
                        t.spawn((
                            TextSpan("If you lose, your".to_string()),
                            TextFont{
                                font: asset_server.load(Font::Medium.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));

                        t.spawn((
                            TextSpan(" Computer may shut down.\n".to_string()),
                            TextFont{
                                font: asset_server.load(Font::Bold.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));

                        t.spawn((
                            TextSpan(" Do you still wish to".to_string()),
                            TextFont{
                                font: asset_server.load(Font::Medium.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));

                        t.spawn((
                            TextSpan(" Continue?".to_string()),
                            TextFont{
                                font: asset_server.load(Font::Bold.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));

                    });

                    p.spawn(
                        Node{
                            width: Val::Percent(100.0),
                            column_gap: Val::Px(10.0),
                            ..default()
                        }
                    ).with_children(|n| {

                        let base_node = (
                            Button,
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Px(60.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(12.0)),
                                ..default()
                            },
                            ui_layer.clone(),
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

                        n.spawn(base_node.clone()).with_child((
                            Text("Yes".to_string()),
                            base_font.clone()
                        )).observe(|
                            trigger: On<Pointer<Click>>,
                            write_mpsc: Res<WriteMpsc>,
                            base: Res<BasicInfos>,
                            mut next_rps_state: ResMut<NextState<RpsState>>,
                            mut rpstype: ResMut<RpsModalResource>
                        | {
                            let mut _token = String::new();
                            if let RpsModalType::Send(token) = rpstype.0.clone(){
                                _token = token;
                            }

                            let tx = write_mpsc.0.clone();
                            let id: Vec<u8> = (0..3).map(|_| random_range(1..=255) ).collect();
                            let _ = tx.send(Data {
                                token: Some(Arc::new(base.token.clone())),
                                type_kind: DataTypeKind::RPS,
                                inform: DataType::RPS(RPSType::Send(id,Arc::new(_token)))
                            });

                            *rpstype = RpsModalResource(RpsModalType::None);
                            next_rps_state.set(RpsState::None);
                        });

                        n.spawn(base_node.clone()).with_child((
                            Text("No".to_string()),
                            base_font.clone()
                        )).observe(|
                            trigger: On<Pointer<Click>>,
                            mut next_rps_state: ResMut<NextState<RpsState>>,
                            mut rpstype: ResMut<RpsModalResource>
                        | {
                            *rpstype = RpsModalResource(RpsModalType::None);
                            next_rps_state.set(RpsState::None);
                        });

                    });

                });

            }
            RpsModalType::Accept(_,send_name) => {
                *rps_timer = RpsTimer::Accept(Instant::now());
                p.spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Auto,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    ui_layer.clone(),
                    BackgroundColor(WHEAT.into()),
                )).with_children(|p| {
                    p.spawn((
                        Text("※WARNING※".to_string()),
                        TextFont{
                            font: asset_server.load(Font::Bold.get()),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(RED.into())
                    ));
                    p.spawn((
                        Text(String::new()),
                    )).with_children(|t| {
                        t.spawn((
                            TextSpan(send_name),
                            TextFont{
                                font: asset_server.load(Font::Bold.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));
                        t.spawn((
                            TextSpan(" asked to Game.\n".to_string()),
                            TextFont{
                                font: asset_server.load(Font::Medium.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));
                        t.spawn((
                            TextSpan("If you lose, your".to_string()),
                            TextFont{
                                font: asset_server.load(Font::Medium.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));

                        t.spawn((
                            TextSpan(" Computer may shut down.\n".to_string()),
                            TextFont{
                                font: asset_server.load(Font::Bold.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));

                        t.spawn((
                            TextSpan(" Do you still wish to".to_string()),
                            TextFont{
                                font: asset_server.load(Font::Medium.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));

                        t.spawn((
                            TextSpan(" Continue?".to_string()),
                            TextFont{
                                font: asset_server.load(Font::Bold.get()),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(BLACK.into())
                        ));

                    });

                    p.spawn(
                        Node{
                            width: Val::Percent(100.0),
                            column_gap: Val::Px(10.0),
                            ..default()
                        }
                    ).with_children(|n| {

                        let base_node = (
                            Button,
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Px(60.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(12.0)),
                                ..default()
                            },
                            ui_layer.clone(),
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

                        n.spawn(base_node.clone()).with_child((
                            Text("Yes".to_string()),
                            base_font.clone()
                        )).observe(|
                            trigger: On<Pointer<Click>>,
                            write_mpsc: Res<WriteMpsc>,
                            base: Res<BasicInfos>,
                            mut rps_timer: ResMut<RpsTimer>,
                            mut next_rps_state: ResMut<NextState<RpsState>>,
                            mut rpstype: ResMut<RpsModalResource>
                        | {
                            *rps_timer = RpsTimer::None;
                            let mut _id = Vec::new();
                            if let RpsModalType::Accept(id,_) = rpstype.0.clone(){
                                _id = id;
                            }
                            let tx = write_mpsc.0.clone();
                            let _ = tx.send(Data {
                                token: Some(Arc::new(base.token.clone())),
                                type_kind: DataTypeKind::RPS,
                                inform: DataType::RPS(RPSType::Accept(_id,true))
                            });

                            *rpstype = RpsModalResource(RpsModalType::None);
                            next_rps_state.set(RpsState::None);
                        });

                        n.spawn(base_node.clone()).with_child((
                            Text("No".to_string()),
                            base_font.clone()
                        )).observe(|
                            trigger: On<Pointer<Click>>,
                            write_mpsc: Res<WriteMpsc>,
                            base: Res<BasicInfos>,
                            mut rps_timer: ResMut<RpsTimer>,
                            mut next_rps_state: ResMut<NextState<RpsState>>,
                            mut rpstype: ResMut<RpsModalResource>
                        | {
                            *rps_timer = RpsTimer::None;
                            let mut _id = Vec::new();
                            if let RpsModalType::Accept(id,_) = rpstype.0.clone(){
                                _id = id;
                            }
                            let tx = write_mpsc.0.clone();
                            let _ = tx.send(Data {
                                token: Some(Arc::new(base.token.clone())),
                                type_kind: DataTypeKind::RPS,
                                inform: DataType::RPS(RPSType::Accept(_id,false))
                            });

                            *rpstype = RpsModalResource(RpsModalType::None);
                            next_rps_state.set(RpsState::None);
                        });

                    });

                });
            }
            RpsModalType::Game(_) => {
                println!("Game Make!!");
                *rps_timer = RpsTimer::Game(Instant::now());

                p.spawn((
                    Node {
                        width: Val::Px(424.0),
                        height: Val::Auto,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    ui_layer.clone(),
                    BackgroundColor(WHEAT.into()),
                )).with_children(|p| {

                    p.spawn((
                        Text("RPS: ".to_string()),
                        TextFont{
                            font: asset_server.load(Font::Bold.get()),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(BLACK.into())
                    )).with_child((
                        TextSpan("60.0s".to_string()),
                        TextFont{
                            font: asset_server.load(Font::Medium.get()),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(BLACK.into()),
                        GameTimeText
                    ));

                    p.spawn(
                        Node{
                            width: Val::Percent(100.0),
                            column_gap: Val::Px(10.0),
                            ..default()
                        }
                    ).with_children(|n| {

                        let base_node = (
                            Node {
                                width: Val::Auto,
                                height: Val::Auto,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(12.0)),
                                ..default()
                            },
                            ui_layer.clone(),
                            BackgroundColor(WHITE.into()),
                        );

                        n.spawn(base_node.clone()).with_child((
                            ImageNode::new(asset_server.load("rock.png")),

                        )).observe(|
                            trigger: On<Pointer<Click>>,
                            write_mpsc: Res<WriteMpsc>,
                            base: Res<BasicInfos>,
                            mut rps_timer: ResMut<RpsTimer>,
                            mut next_rps_state: ResMut<NextState<RpsState>>,
                            mut rpstype: ResMut<RpsModalResource>
                        | {
                            *rps_timer = RpsTimer::None;
                            let mut _id = Vec::new();
                            if let RpsModalType::Game(id) = rpstype.0.clone(){
                                _id = id;
                            }
                            let tx = write_mpsc.0.clone();
                            let _ = tx.send(Data {
                                token: Some(Arc::new(base.token.clone())),
                                type_kind: DataTypeKind::RPS,
                                inform: DataType::RPS(RPSType::Rock(_id))
                            });

                            *rpstype = RpsModalResource(RpsModalType::None);
                            next_rps_state.set(RpsState::None);
                        }).observe(|
                            trigger: On<Pointer<Over>>,
                            mut q_bg: Query<&mut BackgroundColor>
                        | {
                            if let Ok(mut bg) = q_bg.get_mut(trigger.entity){
                                bg.0 = GRAY.into();
                            }
                        }).observe(|
                            trigger: On<Pointer<Out>>,
                            mut q_bg: Query<&mut BackgroundColor>
                        | {
                            if let Ok(mut bg) = q_bg.get_mut(trigger.entity){
                                bg.0 = WHITE.into();
                            }
                        });

                        n.spawn(base_node.clone()).with_child((
                            ImageNode::new(asset_server.load("paper.png")),

                        )).observe(|
                            trigger: On<Pointer<Click>>,
                            write_mpsc: Res<WriteMpsc>,
                            base: Res<BasicInfos>,
                            mut rps_timer: ResMut<RpsTimer>,
                            mut next_rps_state: ResMut<NextState<RpsState>>,
                            mut rpstype: ResMut<RpsModalResource>
                        | {
                            *rps_timer = RpsTimer::None;
                            let mut _id = Vec::new();
                            if let RpsModalType::Game(id) = rpstype.0.clone(){
                                _id = id;
                            }
                            let tx = write_mpsc.0.clone();
                            let _ = tx.send(Data {
                                token: Some(Arc::new(base.token.clone())),
                                type_kind: DataTypeKind::RPS,
                                inform: DataType::RPS(RPSType::Paper(_id))
                            });

                            *rpstype = RpsModalResource(RpsModalType::None);
                            next_rps_state.set(RpsState::None);
                        }).observe(|
                            trigger: On<Pointer<Over>>,
                            mut q_bg: Query<&mut BackgroundColor>
                        | {
                            if let Ok(mut bg) = q_bg.get_mut(trigger.entity){
                                bg.0 = GRAY.into();
                            }
                        }).observe(|
                            trigger: On<Pointer<Out>>,
                            mut q_bg: Query<&mut BackgroundColor>
                        | {
                            if let Ok(mut bg) = q_bg.get_mut(trigger.entity){
                                bg.0 = WHITE.into();
                            }
                        });

                        n.spawn(base_node.clone()).with_child((
                            ImageNode::new(asset_server.load("scissors.png")),

                        )).observe(|
                            trigger: On<Pointer<Click>>,
                            write_mpsc: Res<WriteMpsc>,
                            base: Res<BasicInfos>,
                            mut rps_timer: ResMut<RpsTimer>,
                            mut next_rps_state: ResMut<NextState<RpsState>>,
                            mut rpstype: ResMut<RpsModalResource>
                        | {
                            *rps_timer = RpsTimer::None;
                            let mut _id = Vec::new();
                            if let RpsModalType::Game(id) = rpstype.0.clone(){
                                _id = id;
                            }
                            let tx = write_mpsc.0.clone();
                            let _ = tx.send(Data {
                                token: Some(Arc::new(base.token.clone())),
                                type_kind: DataTypeKind::RPS,
                                inform: DataType::RPS(RPSType::Scissor(_id))
                            });

                            *rpstype = RpsModalResource(RpsModalType::None);
                            next_rps_state.set(RpsState::None);
                        }).observe(|
                            trigger: On<Pointer<Over>>,
                            mut q_bg: Query<&mut BackgroundColor>
                        | {
                            if let Ok(mut bg) = q_bg.get_mut(trigger.entity){
                                bg.0 = GRAY.into();
                            }
                        }).observe(|
                            trigger: On<Pointer<Out>>,
                            mut q_bg: Query<&mut BackgroundColor>
                        | {
                            if let Ok(mut bg) = q_bg.get_mut(trigger.entity){
                                bg.0 = WHITE.into();
                            }
                        });

                    });

                });

            }
            RpsModalType::None => {}
        };
    });
}

fn accept_timer(
    write_mpsc: Res<WriteMpsc>,
    base: Res<BasicInfos>,
    mut rps_timer: ResMut<RpsTimer>,
    mut next_rps_state: ResMut<NextState<RpsState>>,
    mut rpstype: ResMut<RpsModalResource>
){
    if let RpsTimer::Accept(time) = *rps_timer{
        let duration = time.elapsed();
        //println!("Accept Duration: {:?}, Sec: {:?}s",duration,duration.as_secs());
        if duration.as_secs() >= 180{
            *rps_timer = RpsTimer::None;
            let mut _id = Vec::new();
            if let RpsModalType::Accept(id,_) = rpstype.0.clone(){
                _id = id;
            }
            let tx = write_mpsc.0.clone();
            let _ = tx.send(Data {
                token: Some(Arc::new(base.token.clone())),
                type_kind: DataTypeKind::RPS,
                inform: DataType::RPS(RPSType::Accept(_id,false))
            });

            *rpstype = RpsModalResource(RpsModalType::None);
            next_rps_state.set(RpsState::None);
        }
    }
}

fn game_timer(
    write_mpsc: Res<WriteMpsc>,
    base: Res<BasicInfos>,
    mut rps_timer: ResMut<RpsTimer>,
    mut next_rps_state: ResMut<NextState<RpsState>>,
    mut rpstype: ResMut<RpsModalResource>,
    mut s_time_text: Single<&mut TextSpan,With<GameTimeText>>
) {
    if let RpsTimer::Game(time) = *rps_timer{
        let duration = time.elapsed();
        //println!("Game Duration: {:?}, Sec: {:?}s",duration,duration.as_secs());
        if duration.as_secs() >= 60{
            *rps_timer = RpsTimer::None;
            let mut _id = Vec::new();
            if let RpsModalType::Game(id) = rpstype.0.clone(){
                _id = id;
            }
            let tx = write_mpsc.0.clone();
            let _ = tx.send(Data {
                token: Some(Arc::new(base.token.clone())),
                type_kind: DataTypeKind::RPS,
                inform: DataType::RPS(RPSType::Fail(_id))
            });

            *rpstype = RpsModalResource(RpsModalType::None);
            next_rps_state.set(RpsState::None);
        }else {
            let text  = format!("{:05.2}s",(60000.0 - duration.as_millis() as f64)/1000.0);
            s_time_text.0 = text;
        }
    }
}

pub fn choice_to_string(choice: u8) -> String{
    //0:Rock 1:Paper 2:Scissor 3:Fail
    match choice {
        0 => "Rock".to_string(),
        1 => "Paper".to_string(),
        2 => "Scissor".to_string(),
        3 => "Forfeit".to_string(),
        _ => "Err".to_string()
    }
}