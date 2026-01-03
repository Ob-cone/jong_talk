mod main_home;
mod talks;
mod setting;

use crate::main_home::main_plugin;
use crate::setting::setting_plugin;
use crate::talks::host::host_plugin;
use crate::talks::join::join_plugin;
use crate::talks::talk::{talk_plugin,};
use bevy::color::palettes::css::GRAY;
use bevy::prelude::{Click, Local, Pointer, Res, State, Trigger};
use bevy::{
    app::{App, AppExit, Last, Startup, Update},
    color::palettes::basic::{BLACK, WHITE},
    color::palettes::css::WHEAT,
    prelude::{
        AppExtStates,
        BackgroundColor,
        Button,
        Camera,
        Camera2d,
        Changed,
        Children,
        ClearColorConfig,
        Commands,
        Component,
        Entity,
        EventReader,
        Interaction,
        Query,
        ResMut,
        Resource,
        States,
        TextColor,
        With
    },
    utils::default,
    DefaultPlugins
};
use bevy_bc_ime_text_field::text_field::TextFieldInfo;
use bevy_bc_ime_text_field::ImeTextFieldPlugin;
use local_ip_address::local_ip;
use rand::distr::Alphanumeric;
use rand::{rng, Rng};
use server_lib::{set_up_tokio, Data};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;
use talks::modal::{modal_plugin, ModalState};
use tokio::runtime::Runtime;
use tokio::sync::{broadcast, mpsc};
use toml::{Table, Value};
use crate::talks::talk_struct::{EventButtonState, EventState};
use crate::talks::talk_update_data::InputDataEvent;

#[tokio::main]
async fn main() {

    let rt = set_up_tokio();

    let (j_tx, j_rx) = mpsc::channel::<ServerResource>(1);
    let (t_tx, t_rx) = mpsc::channel::<Data>(100);
    let (w_tx, _) = broadcast::channel::<Data>(100);
    let (s_tx, _) = broadcast::channel::<bool>(100);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(RuntimeResource(Some(Arc::new(rt))))
        .init_resource::<BasicInfos>()
        .init_resource::<ServerResource>()
        .init_state::<MainState>()
        .init_state::<ModalState>()
        .init_state::<TalkState>()
        .insert_resource(JoinResultReceiver(j_tx,j_rx))
        .insert_resource(TalkMpsc(t_tx,t_rx))
        .insert_resource(WriteMpsc(w_tx))
        .insert_resource(ServerOffBroadcast(s_tx))
        .insert_resource(EventButtonState(EventState::RPS))
        .init_resource::<ResUserList>()
        .init_resource::<LastState>()
        .insert_resource(IsSendText(false))
        .add_message::<InputDataEvent>()
        .add_systems(Startup,setup)
        .add_plugins(ImeTextFieldPlugin)
        .add_plugins((main_plugin,host_plugin,join_plugin,talk_plugin,setting_plugin,modal_plugin))
        .add_systems(Update,button_system)
        .add_systems(Update,update_last_state)
        .add_systems(Last,exit_program)
        .run();
}

fn exit_program(mut exit_events: EventReader<AppExit>,mut rt: ResMut<RuntimeResource>){
    let event = exit_events.read();
    if event.count() > 0 {
        if let Ok(runtime) = Arc::try_unwrap(rt.0.take().unwrap()) {
            runtime.shutdown_background();
        } else {
            println!("Fail");
        }
        println!("Exit Program");
    }
}

fn setup(mut commands: Commands,mut res: ResMut<BasicInfos>){
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(WHEAT.into()),
            ..default()
        }
    ));

    let file = File::open("config.toml");
    let mut is_read = false;

    if let Ok(mut file) = file{
        let mut text = String::new();

        if let Ok(_) = file.read_to_string(&mut text){
            if let Ok(toml) = text.parse::<Table>(){
                is_read = true;
                if let Some(txt) = toml["User"]["token"].as_str() {
                    res.token = txt.to_string();
                };
                if let Some(txt) = toml["User"]["name"].as_str() {
                    res.name = txt.to_string();
                };
            }
        }
    }

    if !is_read {

        let mut toml = Table::new();

        let mut user = Table::new();

        let token: String = rng()
            .sample_iter(Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();

        res.token = token.clone();

        let name: String = rng()
            .sample_iter(Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        res.name = name.clone();

        user.insert("token".to_string(),Value::String(token));
        user.insert("name".to_string(),Value::String(name));

        toml.insert("User".to_string(),Value::Table(user));

        let mut file = File::create("config.toml").unwrap();
        let _ = file.write(toml.to_string().as_bytes());
    }
    println!("----- App Start -----");
    println!("token: {:?}",res.token);
    println!("name: {:?}",res.name);
}

#[derive(Resource,Default, Clone)]
struct BasicInfos {
    token: String,
    name: String,
}

#[derive(Resource)]
struct JoinResultReceiver(mpsc::Sender<ServerResource>,mpsc::Receiver<ServerResource>);

#[derive(Resource)]
struct TalkMpsc(mpsc::Sender<Data>,mpsc::Receiver<Data>);

#[derive(Resource)]
struct WriteMpsc(broadcast::Sender<Data>);

#[derive(Resource)]
struct ServerOffBroadcast(broadcast::Sender<bool>);

#[derive(Resource,Default)]
struct ResUserList(HashMap<String,String>);

#[derive(Resource,Clone)]
struct RuntimeResource(Option<Arc<Runtime>>);

#[derive(Resource)]
struct IsSendText(bool);

#[derive(Resource)]
#[derive(Default)]
struct LastState(Option<MainState>);

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MainState {
    #[default]
    MainHome,
    Setting,
    Host,
    Join,
    None
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum TalkState {
    Display,
    #[default]
    None
}

fn update_last_state(mut last: ResMut<LastState>, now: Res<State<MainState>>, mut local: Local<Option<MainState>>){
    let current = now.get().clone();
    if local.as_ref() != Some(&current) {
        if let Some(last_s) = local.take() {
            last.0 = Some(last_s);
        }
        *local = Some(current);
    }
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

enum Font{
    Bold,
    Medium
}

impl Font {
    fn get(self) -> &'static str {
        match self {
            Font::Bold => "Paperlogy-7Bold.ttf",
            Font::Medium => "Paperlogy-5Medium.ttf"
        }
    }
}

fn button_system(
    mut q_interaction: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &Children,
            Option<&ButtonInfo>
        ),
        (Changed<Interaction>, With<Button>)
    >,
    mut q_text: Query<&mut TextColor>
) {
    for (interaction, mut bg, children,info) in &mut q_interaction{
        let mut text = q_text.get_mut(children[0]).unwrap();
        match info {
            Some(ButtonInfo(false)) => {
                text.0 = GRAY.into();
            }
            _ => {
                match interaction {
                    Interaction::Pressed => {
                        bg.0 = BLACK.into();
                        text.0 = BLACK.into();
                    }
                    Interaction::Hovered => {
                        bg.0 = BLACK.into();
                        text.0 = WHITE.into();
                    }
                    Interaction::None => {
                        bg.0 = WHITE.into();
                        text.0 = BLACK.into();
                    }
                }
            }
        }
    }
}

fn click_textfield(trigger: Trigger<Pointer<Click>>, mut child: Query<&Children>, mut field: Query<&mut TextFieldInfo>){
    if let Ok(children) = child.get_mut(trigger.entity){
        if let Ok(mut field) = field.get_mut(children[0]){
            field.focus = true;
        }
    }
}

fn get_ip() -> String{
    let ip = local_ip().unwrap();
    ip.to_string().trim().to_string()
}

#[derive(Resource,Default)]
struct ServerResource{
    addr: String,
    is_host: bool
}

#[derive(Component)]
struct ButtonInfo(bool);