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