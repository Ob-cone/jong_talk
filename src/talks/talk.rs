use crate::talks::modal::ModalState;
use crate::talks::talk_struct::{Chat, ChatField, ChatParent, EventButtonState, EventState, EventStateChangeButton, MainNode, OnTalkState, RightNode, TextNode, UserList};
use crate::talks::talk_update_data::{message_event, name_event, remove_event, rps_event, update_data};
use crate::{despawn_screen, BasicInfos, Font, IsSendText, JoinResultReceiver, MainState, RuntimeResource, ServerOffBroadcast, ServerResource, TalkMpsc, TalkState, WriteMpsc};
use bevy::app::{App, Update};
use bevy::asset::AssetServer;
use bevy::color::palettes::css::{BLACK, WHITE};
use bevy::color::palettes::tailwind::BLUE_200;
use bevy::input::ButtonInput;
use bevy::prelude::{in_state, BorderRadius, Button, Changed, Click, Commands, DetectChanges, Display, FlexDirection, IntoScheduleConfigs, JustifyText, KeyCode, LineBreak, Name, NextState, OnEnter, OnExit, Pointer, PositionType, Query, Res, ResMut, Single, State, Text, Transform, Trigger, With};
use bevy::text::{TextColor, TextFont, TextLayout};
use bevy::ui::{AlignItems, BackgroundColor, ComputedNode, JustifyContent, Node, UiRect, Val};
use bevy::utils::default;
use bevy_bc_ime_text_field::event::EnterEvent;
use bevy_bc_ime_text_field::text_field::{TextField, TextFieldInfo};
use bevy_bc_ime_text_field::text_field_style::TextFieldStyle;
use bevy_simple_scroll_view::{ScrollView, ScrollableContent};
use server_lib::{tokio_spawn, Data, DataType, DataTypeKind};
use std::sync::Arc;

pub fn talk_plugin(app: &mut App){
    app.add_systems(OnEnter(TalkState::Display), (setup, get_data))
        .add_systems(Update,update_data.run_if(in_state(TalkState::Display)))
        .add_systems(Update,change_chat.run_if(in_state(TalkState::Display)))
        .add_systems(Update,change_display.run_if(in_state(TalkState::Display)))
        .add_systems(Update,scroll_bottom.run_if(in_state(TalkState::Display)))
        .add_systems(Update,on_modal.run_if(in_state(TalkState::Display)))
        .add_systems(Update,name_event.run_if(in_state(TalkState::Display)))
        .add_systems(Update,remove_event .run_if(in_state(TalkState::Display)))
        .add_systems(Update,message_event.run_if(in_state(TalkState::Display)))
        .add_systems(Update,rps_event.run_if(in_state(TalkState::Display)))
        .add_systems(OnExit(TalkState::Display), despawn_screen::<OnTalkState>);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>,basic_infos: Res<BasicInfos>){

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            column_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        OnTalkState,
        MainNode,
        Transform::from_xyz(0.0,0.0,-1.0),
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(20.0),
                height: Val::Percent(100.0),
                row_gap: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BorderRadius::all(Val::Px(15.0))
        )).with_children(|parent| {

            parent.spawn((
                Name::new("user_list"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0 - 7.5),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(WHITE.into()),
                BorderRadius::all(Val::Px(15.0)),
                ScrollView {scroll_speed: 3600.0},
            )).with_children(|parent| {
                parent.spawn((
                    ScrollableContent::default(),
                    UserList
                ));
            });

            parent.spawn((
                Name::new("user_profile"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(7.5),
                    padding: UiRect::all(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(WHITE.into()),
                BorderRadius::all(Val::Px(15.0))
            )).with_children(|p| {

                p.spawn((
                    Text::new(basic_infos.name.clone()),
                    TextFont {
                        font: asset_server.load(Font::Bold.get()),
                        ..default()
                    },
                    TextColor(BLACK.into())
                ));

                p.spawn((
                    Button,
                    Node {
                        width: Val::Percent(30.0),
                        height: Val::Percent(80.0),
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        right: Val::Px(10.0),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(WHITE.into()),
                )).with_child((
                    EventStateChangeButton,
                    Text::new("RPS"),
                    TextFont {
                        font: asset_server.load(Font::Bold.get()),
                        ..default()
                    },
                    TextColor(BLACK.into())
                )).observe(|
                    _: Trigger<Pointer<Click>>,
                    mut q_text: Query<&mut Text,With<EventStateChangeButton>>,
                    mut info: ResMut<EventButtonState>
                | {
                    if let Ok(mut text) = q_text.single_mut(){
                        match info.0 {
                            EventState::RPS => {info.0 = EventState::OFF}
                            EventState::OFF => {info.0 = EventState::RPS}
                        }
                        text.0 = format!("{:?}",info.0);
                    }
                });

            });

        });
        parent.spawn((
            Node {
                width: Val::Percent(80.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                ..default()
            },
            BackgroundColor(WHITE.into()),
            BorderRadius::all(Val::Px(15.0)),
            RightNode
        )).with_children(|p| {

            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::End,
                    ..default()
                },
                //BackgroundColor(RED.into()),
                ScrollView {scroll_speed: 3600.0},
                ChatParent
            )).with_child((
                ScrollableContent::default(),
                Chat
            ));

            p.spawn((
                Node {
                    height: Val::Auto,
                    min_height: Val::Px(50.0),
                    max_height: Val::Percent(20.0),
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(BLUE_200.into()),
                BorderRadius::all(Val::Px(15.0)),
                TextNode
            )).with_children(|p| {
                p.spawn((
                    TextField::default(),
                    TextLayout::new(JustifyText::Left,LineBreak::WordOrCharacter),
                    TextFieldInfo {
                        max_length: Some(350),
                        placeholder: Some("Hello World!".to_string()),
                        changeable_focus_with_click: false,
                        ..default()
                    },
                    TextFieldStyle {
                        font: TextFont {
                            font: asset_server.load(Font::Medium.get()),
                            ..default()
                        },
                        color: BLACK.into(),
                        ..default()
                    },
                    ChatField
                )).observe(|
                    trigger: Trigger<EnterEvent>,
                    mut q_field: Query<&mut TextField>,
                    write_mpsc: Res<WriteMpsc>,
                    basic_infos: Res<BasicInfos>
                | {
                    if let Ok(mut field) = q_field.get_mut(trigger.entity) {
                        if field.text.trim().is_empty()
                        {
                            field.text = "".to_string();
                            return;
                        }
                        let tx = write_mpsc.0.clone();
                        let _ = tx.send(Data {
                            token: Some(Arc::new(basic_infos.token.clone())),
                            type_kind: DataTypeKind::Message,
                            inform: DataType::Message(Arc::new(field.text.clone().trim().to_string()))
                        });
                        field.text = "".to_string();
                    }
                });
            });

        });
    });
}

fn change_chat(
    q_text: Query<&ComputedNode,With<TextNode>>,
    q_right: Query<&ComputedNode,With<RightNode>>,
    mut q_chat: Query<&mut Node,With<ChatParent>>
){
    let text = q_text.single().unwrap();
    let right = q_right.single().unwrap();

    let height = right.size.y - text.size.y;

    let mut chat = q_chat.single_mut().unwrap();
    chat.max_height = Val::Px(height);
}

fn change_display(state: Res<State<MainState>>,mut main_node: Single<&mut Node,With<MainNode>>,mut field: Single<&mut TextFieldInfo,With<ChatField>>){
    if state.is_changed(){
        println!("State Change: {:?}",state);
        let state = state.get().clone();
        field.focus = state == MainState::None;
        main_node.display = if state == MainState::None {Display::DEFAULT} else { Display::None };
    }
}

fn scroll_bottom(mut is_send_text: ResMut<IsSendText>, mut q_chat: Query<&mut ScrollableContent,(Changed<ScrollableContent>, With<Chat>)>){
    if is_send_text.0 {
        for mut chat in q_chat.iter_mut(){
            chat.scroll_to_bottom();
            is_send_text.0 = false;
        }
    }
}

fn get_data(
    rt: Res<RuntimeResource>,
    server: Res<ServerResource>,
    talk_mpsc: Res<TalkMpsc> ,
    basic_infos: Res<BasicInfos>,
    write_mpsc: Res<WriteMpsc>,
    server_off: Res<ServerOffBroadcast>
) {
    let infos = basic_infos.clone();
    let addr = server.addr.clone();
    if let Some(rt) = rt.0.clone() {
        let tx = talk_mpsc.0.clone();
        let mut rx = write_mpsc.0.subscribe();
        let mut server_rx = server_off.0.subscribe();
        tokio_spawn(rt,async move{
            if let Ok(stream) = server_lib::join_server(infos.token, infos.name, addr).await {
                let (mut r_stream, mut w_stream) = stream.into_split();

                let r_tokio = tokio::spawn(async move{
                    loop {
                        let data = Data::read_data(&mut r_stream).await;
                        if let Ok(data) = data {
                            let _ = tx.send(data.clone()).await;
                            println!("Read(Client): {:?}",data);
                        }else { println!("Read Error(Client): {:?}",data); }
                    }
                });

                let w_tokio = tokio::spawn(async move {
                    loop {
                        while let Ok(data) = rx.recv().await {
                            let rst = Data::write_data(&mut w_stream,data.clone()).await;
                            println!("Write(Client): {:?}, {:?}",data,rst);
                        }
                    }
                });

                loop {
                    if let Ok(off) = server_rx.recv().await {
                        if off {
                            r_tokio.abort();
                            w_tokio.abort();
                            break
                        }
                    }
                }

            }else { println!("Fail Connect!") }
            println!("End");
        });
    }
}

pub fn join_after(
    mut mpsc: ResMut<JoinResultReceiver>,
    mut server: ResMut<ServerResource>,
    mut state: ResMut<NextState<MainState>>,
    mut talk_state: ResMut<NextState<TalkState>>
){
    if let Ok(sr) = mpsc.1.try_recv(){
        server.addr = sr.addr;
        server.is_host = sr.is_host;
        state.set(MainState::None);
        talk_state.set(TalkState::Display);
    }
}

fn on_modal(
    button: Res<ButtonInput<KeyCode>>,
    state: Res<State<ModalState>>,
    mut next_state: ResMut<NextState<ModalState>>,
    mut field: Single<&mut TextFieldInfo>
) {
    if button.just_pressed(KeyCode::Escape) {

        if state.get() == &ModalState::Display {
            next_state.set(ModalState::None);
            field.focus = true;
        }else {
            next_state.set(ModalState::Display);
            field.focus = false;
        }
    }
}