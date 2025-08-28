use crate::main_home::get_main_home_back_button;
use crate::talks::talk::join_after;
use crate::{despawn_screen, get_ip, BasicInfos, Font, JoinResultReceiver, MainState, RuntimeResource, ServerOffBroadcast, ServerResource};
use bevy::app::{App, Update};
use bevy::asset::AssetServer;
use bevy::color::palettes::css::{BLACK, WHITE};
use bevy::prelude::{in_state, AlignItems, Button, Click, Commands, Component, FlexDirection, IntoScheduleConfigs, JustifyContent, OnEnter, OnExit, Pointer, Query, Res, Text, Trigger};
use bevy::tasks::IoTaskPool;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{BackgroundColor, BorderRadius, Node, Overflow, Val};
use bevy::utils::default;
use bevy_bc_ime_text_field::text_field::{TextField, TextFieldInfo};
use bevy_bc_ime_text_field::text_field_style::TextFieldStyle;
use server_lib::{server_host, tokio_spawn};
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn host_plugin(app: &mut App){
    app.add_systems(OnEnter(MainState::Host),setup)
        .add_systems(Update,join_after.run_if(in_state(MainState::Host)))
        .add_systems(OnExit(MainState::Host),despawn_screen::<OnHostState>);
}

#[derive(Component)]
struct OnHostState;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>){

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },
        OnHostState
    )).with_children(|parent| {

        get_main_home_back_button(parent,asset_server.clone());

        parent.spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(WHITE.into()),
            BorderRadius::all(Val::Px(15.0))
        )).with_child((
            TextField::default(),
            TextFieldInfo {
                focus: true,
                placeholder: Some("Port".to_string()),
                max_length: Some(5),
                ..default()
            },
            TextFieldStyle {
                font: TextFont {
                    font: asset_server.load(Font::Medium.get()),
                    font_size: 42.0,
                    ..default()
                },
                color: BLACK.into(),
                ..default()
            }
        ));

        parent.spawn((
            Button,
            Node {
                width: Val::Px(180.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::hidden(),
                ..default()
            },
            BackgroundColor(WHITE.into()),
            BorderRadius::all(Val::Px(15.0))
        )).with_child((
            Text::new("Host"),
            TextFont {
                font: asset_server.load(Font::Bold.get()),
                font_size: 36.0,
                ..default()
            },
            TextColor(BLACK.into())
        )).observe(|_: Trigger<Pointer<Click>>,
                    text_field: Query<&TextField>,
                    rt: Res<RuntimeResource>,
                    basic_infos: Res<BasicInfos>,
                    tx: Res<JoinResultReceiver>,
                    server_off: Res<ServerOffBroadcast>
        | {
            if let Ok(field) = text_field.single(){
                
                let port = &field.text;
                let num_port;
                if let Ok(num) = port.parse::<u32>(){
                    num_port = num % 65536;
                }
                else {
                    let mut hasher = DefaultHasher::new();
                    port.hash(&mut hasher);
                    num_port = (hasher.finish() as u32) % 65536;
                }

                let addr = format!("{}:{}",get_ip(),num_port);

                if let Some(rt) = rt.0.clone() {
                    let _addr = addr.clone();
                    let tx = server_off.0.clone();
                    tokio_spawn(rt, async {
                        let rst = server_host(_addr,tx).await;
                        if rst.is_err() {println!("Err!");}
                    });
                }

                let _rt = rt.clone();
                let _info = basic_infos.clone();
                let tx = tx.0.clone();

                IoTaskPool::get().spawn(async move{
                    let resource = ServerResource {
                        addr: addr.clone(),
                        is_host: true
                    };
                    let _ = tx.send(resource).await;
                }).detach();


            }
        });

    });

}