use crate::main_home::get_main_home_back_button;
use crate::talks::talk::join_after;
use crate::{click_textfield, despawn_screen, BasicInfos, Font, JoinResultReceiver, MainState, RuntimeResource, ServerResource};
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::color::palettes::basic::{BLACK, WHITE};
use bevy::prelude::{default, in_state, AlignItems, BackgroundColor, BorderRadius, Button, Changed, Click, Commands, Component, FlexDirection, IntoScheduleConfigs, JustifyContent, Node, OnEnter, OnExit, Overflow, Pointer, Query, Res, Text, TextColor, TextFont, Trigger, Update, Val, With};
use bevy::tasks::IoTaskPool;
use bevy::text::TextLayoutInfo;
use bevy_bc_ime_text_field::text_field::{TextField, TextFieldInfo};
use bevy_bc_ime_text_field::text_field_style::TextFieldStyle;

pub fn join_plugin(app: &mut App){
    app.add_systems(OnEnter(MainState::Join), setup)
        .add_systems(Update,text_fixed.run_if(in_state(MainState::Join)))
        .add_systems(Update,join_after.run_if(in_state(MainState::Join)))
        .add_systems(OnExit(MainState::Join),despawn_screen::<OnJoinState>);
}

#[derive(Component)]
struct OnJoinState;

#[derive(Component)]
struct HasTextField;

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
        OnJoinState
    )).with_children(|parent| {

        get_main_home_back_button(parent,asset_server.clone());

        parent.spawn((
            Node {
                width: Val::Px(540.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(WHITE.into()),
            BorderRadius::all(Val::Px(15.0)),
            HasTextField
        )).with_child((
            TextField::default(),
            TextFieldInfo {
                placeholder: Some("Addr".to_string()),
                focus: true,
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
        )).observe(click_textfield);

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
            Text::new("Join"),
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
                          tx: Res<JoinResultReceiver>, |
            {
                if let Ok(field) = text_field.single(){
                    let addr = field.text.clone();
                    println!("Addr: {:?}",addr);

                    let _rt = rt.clone();
                    let _info = basic_infos.clone();
                    let tx = tx.0.clone();

                    IoTaskPool::get().spawn(async move{
                        let resource = ServerResource {
                            addr: addr.clone(),
                            is_host: false
                        };
                        let _ = tx.send(resource).await;
                    }).detach();
                };
            });

    });

}

fn text_fixed(text: Query<&TextLayoutInfo,Changed<TextField>>, mut node: Query<&mut Node,With<HasTextField>>){
    if let Ok(layout) = text.single(){
        if let Ok(mut node) = node.single_mut(){
            node.justify_content = if layout.size.x > 540.0 { JustifyContent::End }
            else { JustifyContent::Center };
        }
    }
}

