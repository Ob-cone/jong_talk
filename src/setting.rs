use crate::main_home::get_main_home_back_button;
use crate::{despawn_screen, BasicInfos, Font, MainState};
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::color::palettes::basic::{BLACK, WHITE};
use bevy::prelude::{default, AlignItems, BackgroundColor, BorderRadius, Commands, Component, FlexDirection, JustifyContent, Node, OnEnter, OnExit, Query, Res, ResMut, TextFont, Trigger, Val};
use bevy_bc_ime_text_field::event::EnterEvent;
use bevy_bc_ime_text_field::text_field::{TextField, TextFieldInfo};
use bevy_bc_ime_text_field::text_field_style::TextFieldStyle;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use toml::{Table, Value};

pub fn setting_plugin(app: &mut App){
    app.add_systems(OnEnter(MainState::Setting), setup)
        .add_systems(OnExit(MainState::Setting),despawn_screen::<OnSettingState>);
}

#[derive(Component)]
struct OnSettingState;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    basic_infos: Res<BasicInfos>
){
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
        OnSettingState
    )).with_children(|p| {
        
        get_main_home_back_button(p,asset_server.clone());

        let name = format!("Name({})",basic_infos.name);

        p.spawn((
            Node {
                width: Val::Px(600.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(WHITE.into()),
            BorderRadius::all(Val::Px(15.0))
        )).with_children(|p| {
            p.spawn((
                TextField::default(),
                TextFieldInfo {
                    focus: true,
                    placeholder: Some(name),
                    max_length: Some(15),
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
            )).observe(|trigger: Trigger<EnterEvent>, mut basic_infos: ResMut<BasicInfos>,mut q_field: Query<(&mut TextField,&mut TextFieldInfo)>|{
                if let Ok((mut field, mut layout)) = q_field.get_mut(trigger.entity) {
                    if !trigger.text_field.text.trim().is_empty(){
                        basic_infos.name = field.text.trim().to_string();
                        let name = format!("Name({})",basic_infos.name);
                        layout.placeholder = Some(name);

                        let mut file = OpenOptions::new()
                            .create(true)
                            .read(true)
                            .write(true)
                            .open("config.toml").unwrap();
                        let mut text = String::new();
                        if let Ok(_) = file.read_to_string(&mut text){
                            if let Ok(mut toml) = text.parse::<Table>(){
                                toml["User"]["name"] = Value::String(basic_infos.name.clone());
                                let _ = file.seek(SeekFrom::Start(0));
                                let _ = file.set_len(0);
                                let r = writeln!(file, "{}", toml.to_string());
                                println!("{:?}",r)
                            }
                        }

                    }
                    field.text = "".to_string();
                }

            });
        });

    });
}