use std::collections::HashMap;
use std::time::Instant;
use bevy::prelude::{Component, Resource};

#[derive(Component)]
pub struct OnTalkState;

#[derive(Component)]
pub struct UserList;

#[derive(Component)]
pub struct Chat;

#[derive(Component)]
pub struct ChatParent;

#[derive(Component)]
pub struct TextNode;

#[derive(Component)]
pub struct RightNode;

#[derive(Component)]
pub struct Token(pub(crate) String);

#[derive(Component)]
pub struct MainNode;
#[derive(Component)]
pub struct ChatField;

#[derive(Resource,Debug)]
#[derive(PartialEq)]
pub struct EventButtonState(pub EventState);

#[derive(Component)]
pub struct EventStateChangeButton;
#[derive(Resource,Debug)]
pub struct OffList(pub HashMap<String,Vec<String>>);
#[derive(Resource,Debug)]
pub struct PingPong(pub Option<Instant>);

#[derive(Debug)]
#[derive(PartialEq)]
pub enum EventState{
    RPS,
    OFF
}