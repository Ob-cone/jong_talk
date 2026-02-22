use bevy::app::Update;
use bevy::prelude::{Added, App, ChildOf, Commands, Component, ComputedNode, Entity, Node, On, Pointer, Query, Val};

#[derive(Component)]
pub struct ScrollComponent;

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
            mut q_node: Query<(&mut Node,&ComputedNode)>,
            q_parent: Query<&ChildOf>
        |{

            let parent = q_parent.get(trigger.entity).unwrap();

            let mut parent_heigh = 0.0;

            if let Ok((_,p_com_node)) = q_node.get(parent.0){
                parent_heigh = p_com_node.size.y;
            }

            if let Ok((mut node,com_node))= q_node.get_mut(trigger.entity){
                if let Val::Px(num) = node.bottom{
                    let mut height = parent_heigh - com_node.size.y;
                    height = height.min(0.0)/2.0;
                    let mut val = num+trigger.y*25.0;
                    val = val.clamp(height,0.0);
                    node.bottom = Val::Px(val);
                }
            }
        });
    }
}