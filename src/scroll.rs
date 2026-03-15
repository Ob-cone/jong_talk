use bevy::app::Update;
use bevy::prelude::{Added, App, ChildOf, Commands, Component, ComputedNode, Entity, Node, On, Pointer, Query, Val, With};
use bevy::window::{PrimaryWindow, Window};

#[derive(Component)]
pub enum ScrollComponent {
    Top,    // Info처럼 위에서 시작
    Bottom, // Chat처럼 아래에서 시작
}

pub fn scroll_plugin(app: &mut App){
    app.add_systems(Update,add_observe);
}

fn add_observe(
    mut commands: Commands,
    q_scroll: Query<Entity,Added<ScrollComponent>>
){
    for scroll in q_scroll{
        commands.entity(scroll).observe(|
            trigger: On<Pointer<bevy::prelude::Scroll>>,
            mut q_node: Query<(&mut Node, &ComputedNode)>,
            q_parent: Query<&ChildOf>,
            q_scroll: Query<&ScrollComponent>,
            q_window: Query<&Window,With<PrimaryWindow>>
        |{
            let scale = if let Ok(window) = q_window.single(){
                window.scale_factor()
            }else { 1.0 };
            let scroll_type = q_scroll.get(trigger.entity).unwrap();

            let parent = q_parent.get(trigger.entity).unwrap();
            let mut parent_height = 0.0;
            if let Ok((_, p_com_node)) = q_node.get(parent.0) {
                parent_height = p_com_node.size.y;
            }

            if let Ok((mut node, com_node)) = q_node.get_mut(trigger.entity) {

                match scroll_type {
                    ScrollComponent::Top => {
                        if let Val::Px(num) = node.top {
                            let mut height = com_node.size.y - parent_height;
                            height = height.max(0.0) / scale;
                            let mut val = num + trigger.y * 25.0;
                            val = val.clamp(-height, 0.0);
                            node.top = Val::Px(val);
                        }
                    }
                    ScrollComponent::Bottom => {
                        if let Val::Px(num) = node.bottom{
                            let mut height = parent_height - com_node.size.y;
                            height = height.min(0.0)/scale;
                            let mut val = num-trigger.y*25.0;
                            val = val.clamp(height,0.0);
                            node.bottom = Val::Px(val);
                        }
                    }
                }
            }
        });
    }
}