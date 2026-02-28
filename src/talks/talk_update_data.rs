use std::collections::HashSet;
use crate::talks::rps_game::{choice_to_string, RpsList, RpsModalResource, RpsModalType, RpsState};
use crate::talks::talk_struct::{Chat, EventButtonState, EventState, OffList, Token, UserList};
use crate::{BasicInfos, ButtonInfo, FailConnectMpsc, Font, IsSendText, ResUserList, TalkMpsc, WriteMpsc};
use bevy::asset::AssetServer;
use bevy::color::palettes::basic::{BLACK, WHITE};
use bevy::color::palettes::tailwind::{GRAY_600, RED_500};
use bevy::picking::Pickable;
use bevy::prelude::{default, AlignItems, BackgroundColor, BorderRadius, Button, Children, Click, Commands, Entity, Event, FlexDirection, JustifyContent, LineBreak, Message, MessageReader, MessageWriter, NextState, Node, On, Pointer, PositionType, Query, Res, ResMut, Text, TextColor, TextFont, TextLayout, TextSpan, UiRect, Val, With};
use bevy::text::Justify;
use bevy::ui::percent;
use server_lib::{Data, DataType, DataTypeKind, RPSType};
use std::sync::Arc;
use system_shutdown::shutdown;

#[derive(Event,Message)]
pub struct InputDataEvent(Data);



pub(crate) fn update_data(
    mut input_data_event: MessageWriter<InputDataEvent>,
    mut talk_mpsc: ResMut<TalkMpsc>,
){
    while let Ok(data) = talk_mpsc.1.try_recv(){
        input_data_event.write(InputDataEvent(data));
    }
}

pub(crate) fn rps_change(
    res_rps_list: Res<RpsList>,
    mut q_button: Query<(&mut ButtonInfo,&Token)>
){
    println!("바뀜R");
    let mut list = HashSet::<&String>::new();
    for (a,b) in res_rps_list.0.values(){
        list.insert(a);
        list.insert(b);
    }
    println!("RPS: {:?}",list);
    for (mut button,token) in q_button.iter_mut(){
        println!("C: {:?} {:?}",token.0.clone(),list.contains(&token.0));
        button.0 = !list.contains(&token.0);
    }
}

pub(crate) fn off_change(
    basic_infos: Res<BasicInfos>,
    res_off_list: Res<OffList>,
    mut q_button: Query<(&mut ButtonInfo,&Token)>
){
    println!("바뀜O");
    if let Some(list) = res_off_list.0.get(&basic_infos.token){
        for (mut button,token) in q_button.iter_mut(){
            button.0 = list.contains(&token.0);
        }
    }else {
        for (mut button,token) in q_button.iter_mut(){
            button.0 = false;
        }
    }
}

pub(crate) fn fail_event(
    mut commands: Commands,
    mut fail_mpsc: ResMut<FailConnectMpsc>,
    q_chat: Query<Entity,With<Chat>>,
    asset_server: Res<AssetServer>,
){
    while let Ok(is_fail_connect) = fail_mpsc.1.try_recv(){
        if is_fail_connect{
            let chat = q_chat.single().unwrap();
            let mut chat = commands.get_entity(chat).unwrap();
            chat.with_child((
                Text::new("--- Fail Connect! ---".to_string()),
                TextFont {
                    font: asset_server.load(Font::Bold.get()),
                    font_size: 20.0,
                    ..default()
                },
                TextLayout::new(Justify::Center,LineBreak::WordOrCharacter),
                TextColor(RED_500.into())
            ));
        }
    }
}

pub(crate) fn game_data_event(
    mut event: MessageReader<InputDataEvent>,
    mut res_rps_list: ResMut<RpsList>,
    mut res_off_list: ResMut<OffList>
){
    for event in event.read(){
        if let DataTypeKind::RPS = event.0.type_kind{
            let data = event.0.clone();
            if let DataType::RPS(rps) = data.inform{
                if let RPSType::Data(v,token) = rps{
                    let token1 = format!("{}",&data.token.unwrap());
                    let token2 = format!("{}",&token);
                    res_rps_list.0.insert(v,(token1,token2));
                }
            }
        }
        else if let DataTypeKind::OffData = event.0.type_kind {
            let data = event.0.clone();
            if let DataType::OffData(token) = data.inform{
                let token1 = format!("{}",&data.token.unwrap());
                let token2 = format!("{}",&token);
                res_off_list.0.entry(token1.clone()).or_insert(vec![]).push(token2.clone());
            }
        }
    }
}

pub(crate) fn name_event(
    mut event: MessageReader<InputDataEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    basic_infos: Res<BasicInfos>,
    mut res_user_list: ResMut<ResUserList>,
    q_user_list: Query<(Entity,Option<&Children>),With<UserList>>,
    q_chat: Query<Entity,With<Chat>>,
){
    for event in event.read(){
        if let DataTypeKind::Name = event.0.type_kind{

            let data = event.0.clone();

            let base_node = Node {
                row_gap: Val::Px(5.0),
                margin: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            };

            let base_font = TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 20.0,
                ..default()
            };
            let (user_list,_) = q_user_list.single().unwrap();
            let mut userlist = commands.get_entity(user_list).unwrap();
            if let DataType::Name(name) = data.inform {
                let name = format!("{}",&name);
                let token = format!("{}",&data.token.unwrap());
                userlist.with_children(|p| {
                    p.spawn((
                        base_node,
                        Token(token.clone())
                    )).with_children(|p| {
                        p.spawn((
                            Text::new(name.clone()),
                            base_font.clone(),
                            TextColor(BLACK.into()),
                        ));

                        if basic_infos.token != token {
                            p.spawn((
                                Button,
                                ButtonInfo(true),
                                Node {
                                    width: Val::Px(30.0),
                                    height: Val::Px(30.0),
                                    position_type: PositionType::Absolute,
                                    justify_content: JustifyContent::Center,
                                    border_radius: BorderRadius::all(Val::Px(10.0)),
                                    align_items: AlignItems::Center,
                                    right: Val::ZERO,
                                    ..default()
                                },
                                BackgroundColor(WHITE.into()),
                                Token(token.clone()),
                            )).with_child((
                                Text::new("V"),
                                TextFont {
                                    font: asset_server.load(Font::Bold.get()),
                                    ..default()
                                },
                                TextColor(BLACK.into()),
                                Token(token.clone()),
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false
                                }
                            )).observe(|
                                trigger: On<Pointer<Click>>,
                                base: Res<BasicInfos>,
                                q_token: Query<&Token>,
                                q_button_info: Query<&ButtonInfo>,
                                state: Res<EventButtonState>,
                                write_mpsc: Res<WriteMpsc>,
                                mut next_rps_state: ResMut<NextState<RpsState>>,
                                mut rps_type: ResMut<RpsModalResource>
                            | {
                                if let Ok(button_info) = q_button_info.get(trigger.entity) {
                                    if !button_info.0 {
                                        return;
                                    }
                                }

                                if let Ok(token) = q_token.get(trigger.entity){
                                    match state.0 {
                                        EventState::RPS => {
                                            *rps_type = RpsModalResource(RpsModalType::Send(token.0.clone()));
                                            next_rps_state.set(RpsState::Display);
                                        }
                                        EventState::OFF => {
                                            let tx = write_mpsc.0.clone();
                                            let _ = tx.send(Data {
                                                token: Some(Arc::new(base.token.clone())),
                                                type_kind: DataTypeKind::IsOff,
                                                inform: DataType::IsOff(Arc::new(token.0.clone()))
                                            });
                                        }
                                    }
                                }
                            });
                        }

                    });
                });
                res_user_list.0.insert(token,name.clone());

                let chat = q_chat.single().unwrap();
                let mut chat = commands.get_entity(chat).unwrap();
                let msg = format!("--- Join {} ---",name);
                chat.with_child((
                    Text::new(msg),
                    base_font,
                    TextLayout::new(Justify::Center,LineBreak::WordOrCharacter),
                    TextColor(GRAY_600.into())
                ));

            }
        }
    }
}

pub(crate) fn remove_event(
    mut event: MessageReader<InputDataEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    res_user_list: ResMut<ResUserList>,
    q_user_list: Query<(Entity,Option<&Children>),With<UserList>>,
    q_token: Query<&Token>,
    q_chat: Query<Entity,With<Chat>>,
){
    for event in event.read(){
        if let DataTypeKind::Remove = event.0.type_kind{
            let data = event.0.clone();
            let token = format!("{}",&data.token.unwrap());
            let name = format!("{}",&res_user_list.0[&token]);
            println!("Out: {}",token);

            let base_font = TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 20.0,
                ..default()
            };

            let (_,children) = q_user_list.single().unwrap();
            if let Some(children) = children {
                for child in children{
                    let child_token = q_token.get(child.clone()).unwrap();
                    if child_token.0 == token{
                        commands.get_entity(child.clone()).unwrap().despawn();
                    }
                }
            }

            let chat = q_chat.single().unwrap();
            let mut chat = commands.get_entity(chat).unwrap();
            let msg = format!("--- Leave {} ---",name);
            chat.with_child((
                Text::new(msg),
                base_font,
                TextLayout::new(Justify::Center,LineBreak::WordOrCharacter),
                TextColor(GRAY_600.into())
            ));
        }
    }
}

pub(crate) fn message_event(
    mut event: MessageReader<InputDataEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    basic_infos: Res<BasicInfos>,
    res_user_list: ResMut<ResUserList>,
    mut is_send_text: ResMut<IsSendText>,
    q_chat: Query<Entity,With<Chat>>,
){
    for event in event.read(){
        if let DataTypeKind::Message = event.0.type_kind{
            let data = event.0.clone();
            println!("Msg: {:?}",data.inform);

            let base_node = Node {
                row_gap: Val::Px(5.0),
                width: percent(100),
                margin: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            };

            let base_font = TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 20.0,
                ..default()
            };

            let chat = q_chat.single().unwrap();
            let mut chat = commands.get_entity(chat).unwrap();

            if let DataType::Message(msg) = data.inform {
                let token = format!("{}",&data.token.unwrap());
                let name = res_user_list.0[&token].clone();
                let msg = format!("<{}>\n{}",&name,&msg);
                chat.with_children(|p| {
                    p.spawn(base_node).with_child((
                        Text::new(msg),
                        TextLayout::new(Justify::Left,LineBreak::WordOrCharacter),
                        base_font,
                        TextColor(BLACK.into())
                    ));
                });
                if token == basic_infos.token {
                    is_send_text.0 = true;
                }
            }
        }
    }
}

pub(crate) fn rps_event(
    mut event: MessageReader<InputDataEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    res_user_list: Res<ResUserList>,
    mut res_rps_list: ResMut<RpsList>,
    q_chat: Query<Entity,With<Chat>>,
    base: Res<BasicInfos>,
    mut next_rps_state: ResMut<NextState<RpsState>>,
    mut rpstype: ResMut<RpsModalResource>,
    mut res_off_list: ResMut<OffList>
){
    for event in event.read(){
        if let DataTypeKind::RPS = event.0.type_kind{
            let data = event.0.clone();
            if let DataType::RPS(rps) = data.inform{
                match rps {
                    RPSType::Send(id, token) => {
                        let send_token = format!("{}",data.token.unwrap());
                        let send_name = format!("{}",&res_user_list.0[&send_token]);

                        let accept_token = format!("{}",token);
                        let accept_name = format!("{}",&res_user_list.0[&accept_token]);

                        let chat = q_chat.single().unwrap();
                        let mut chat = commands.get_entity(chat).unwrap();

                        let base_node = Node {
                            row_gap: Val::Px(5.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        };

                        let base_font = TextFont {
                            font: asset_server.load(Font::Bold.get()),
                            font_size: 20.0,
                            ..default()
                        };

                        chat.with_children(|p| {
                            p.spawn(base_node).with_children(|p| {
                                p.spawn((
                                    Text::new(send_name.clone()),
                                    TextLayout::new(Justify::Center,LineBreak::WordOrCharacter),
                                    base_font.clone(),
                                    TextColor(BLACK.into())
                                )).with_children(|p| {
                                    p.spawn((
                                        TextSpan::new(" asked "),
                                        base_font.clone(),
                                        TextColor(GRAY_600.into())
                                    ));

                                    p.spawn((
                                        TextSpan::new(accept_name.clone()),
                                        base_font.clone(),
                                        TextColor(BLACK.into())
                                    ));

                                    p.spawn((
                                        TextSpan::new(" to play rock-paper-scissors."),
                                        base_font.clone(),
                                        TextColor(GRAY_600.into())
                                    ));

                                });
                            });
                        });
                        res_rps_list.0.insert(id.clone(),(send_token.clone(),accept_token.clone()));
                        if base.token == accept_token {
                            *rpstype = RpsModalResource(RpsModalType::Accept(id,send_name.clone()));
                            next_rps_state.set(RpsState::Display);
                        }

                    }
                    RPSType::Accept(id, is_accept) => {
                        println!("RPS: {:?} Id: {:?}",res_rps_list.0,id);
                        if let Some((send_token,accept_token)) = res_rps_list.0.get(&id){

                            let send_name = format!("{}",&res_user_list.0[send_token]);
                            let accept_name = format!("{}",&res_user_list.0[accept_token]);

                            let chat = q_chat.single().unwrap();
                            let mut chat = commands.get_entity(chat).unwrap();

                            let base_node = Node {
                                row_gap: Val::Px(5.0),
                                margin: UiRect::all(Val::Px(10.0)),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            };

                            let base_font = TextFont {
                                font: asset_server.load(Font::Bold.get()),
                                font_size: 20.0,
                                ..default()
                            };

                            chat.with_children(|p| {
                                p.spawn(base_node).with_children(|p| {
                                    p.spawn((
                                        Text::new(accept_name.clone()),
                                        TextLayout::new(Justify::Center,LineBreak::WordOrCharacter),
                                        base_font.clone(),
                                        TextColor(BLACK.into())
                                    )).with_children(|p| {
                                        p.spawn((
                                            TextSpan::new(if is_accept {
                                                " accepted "
                                            } else { " declined " }),
                                            base_font.clone(),
                                            TextColor(GRAY_600.into())
                                        ));

                                        p.spawn((
                                            TextSpan::new(send_name.clone()),
                                            base_font.clone(),
                                            TextColor(BLACK.into())
                                        ));

                                        p.spawn((
                                            TextSpan::new("'s game request."),
                                            base_font.clone(),
                                            TextColor(GRAY_600.into())
                                        ));

                                    });
                                });
                            });
                            if !is_accept {
                                res_rps_list.0.remove(&id);
                            }
                        }
                    }
                    RPSType::Game(id) => {
                        if let Some((send,accept)) = res_rps_list.0.get(&id){
                            if (send == &base.token) || (accept == &base.token) {
                                *rpstype = RpsModalResource(RpsModalType::Game(id));
                                next_rps_state.set(RpsState::Display);
                            }
                        }
                    }
                    RPSType::Result(id,choice,win) => {
                        println!("Id: {:?}, Choice:{:?},Win: {:?}",id,choice,win);
                        if let Some((send_token,accept_token)) = res_rps_list.0.get(&id){
                            let send_name = format!("{}",&res_user_list.0[send_token]);
                            let accept_name = format!("{}",&res_user_list.0[accept_token]);

                            let win_name = if win.to_string() == "None"{
                                "None".to_string()
                            }
                            else {
                                res_user_list.0[&win.to_string()].clone()
                            };

                            let not_win_token = if send_token.to_string() == win.to_string(){
                                accept_token.to_string()
                            }else { win.to_string() };

                            if win.to_string() != "None"{
                                res_off_list.0.entry(win.to_string()).or_insert(vec![]).push(not_win_token.clone());
                                println!("Off List: {:?}",res_off_list.0);
                            }

                            let chat = q_chat.single().unwrap();
                            let mut chat = commands.get_entity(chat).unwrap();

                            let base_node = Node {
                                row_gap: Val::Px(5.0),
                                margin: UiRect::all(Val::Px(10.0)),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            };

                            let base_font = TextFont {
                                font: asset_server.load(Font::Bold.get()),
                                font_size: 20.0,
                                ..default()
                            };

                            chat.with_children(|p| {
                                p.spawn(base_node).with_children(|p| {
                                    p.spawn((
                                        Text::new(send_name.clone()),
                                        TextLayout::new(Justify::Center,LineBreak::WordOrCharacter),
                                        base_font.clone(),
                                        TextColor(BLACK.into())
                                    )).with_children(|p| {
                                        p.spawn((
                                            TextSpan::new(": "),
                                            base_font.clone(),
                                            TextColor(GRAY_600.into())
                                        ));

                                        p.spawn((
                                            TextSpan::new(choice_to_string(choice[0])),
                                            base_font.clone(),
                                            TextColor(BLACK.into())
                                        ));

                                        p.spawn((
                                            TextSpan::new(" vs "),
                                            base_font.clone(),
                                            TextColor(GRAY_600.into())
                                        ));

                                        p.spawn((
                                            TextSpan::new(accept_name.clone()),
                                            base_font.clone(),
                                            TextColor(BLACK.into())
                                        ));

                                        p.spawn((
                                            TextSpan::new(": "),
                                            base_font.clone(),
                                            TextColor(GRAY_600.into())
                                        ));

                                        p.spawn((
                                            TextSpan::new(choice_to_string(choice[1])),
                                            base_font.clone(),
                                            TextColor(BLACK.into())
                                        ));

                                        p.spawn((
                                            TextSpan::new(" -> Win: "),
                                            base_font.clone(),
                                            TextColor(GRAY_600.into())
                                        ));

                                        p.spawn((
                                            TextSpan::new(win_name),
                                            base_font.clone(),
                                            TextColor(BLACK.into())
                                        ));

                                    });
                                });
                            });

                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn off_event(
    mut event: MessageReader<InputDataEvent>,
    base: Res<BasicInfos>,
){
    for event in event.read(){
        if event.0.type_kind == DataTypeKind::Off {
            let data = event.0.clone();
            if let DataType::Off(token) = data.inform{
                if token.to_string() == base.token{
                    match shutdown() {
                        Ok(_) => println!("Shutting down..."),
                        Err(error) => eprintln!("Failed to shut down: {}", error),
                    }
                }
            }
        }
    }
}


