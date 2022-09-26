use bevy::prelude::*;

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn find_child_with_name(
    name: &str,
    children: &Children,
    traversal_query: &Query<(&Children, Option<&Name>)>,
) -> Option<Entity> {
    for child in children {
        if let Ok((children, name_option)) = traversal_query.get(*child) {
            if let Some(child_name) = name_option {
                if child_name.as_str() == name {
                    return Some(*child);
                }
            } 
            let result = find_child_with_name(name, children, traversal_query);
            if result.is_some() {
                return result;
            }
        }
    }
    return None;
}

pub fn find_child_entity_with_component<T: Component>(
    children: &Children,
    hierarchy_query: &Query<(&Children, Option<&T>)>
) -> Option<Entity> {
    for child in children {
        if let Ok((children, animation_player_option)) = hierarchy_query.get(*child) {
            if animation_player_option.is_some() {
                return Some(child.clone());
            } else {
                let result = find_child_entity_with_component(children, hierarchy_query);
                if result.is_some() {
                    return result;
                }
            }
        }
    }
    return None;
}