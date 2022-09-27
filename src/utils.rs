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
    hierarchy_query: &Query<(&Children, Option<&T>)>,
    component_query: &Query<&T>, 
) -> Option<Entity> {
    for child in children {
        if let Ok((children, component_option)) = hierarchy_query.get(*child) {
            if component_option.is_some() {
                return Some(child.clone());
            } else {
                let result = find_child_entity_with_component(children, hierarchy_query, component_query);
                if result.is_some() {
                    return result;
                }
            }
        } else if let Ok(_) = component_query.get(*child) {
            return Some(child.clone())
        }
    }
    return None;
}

pub fn find_children_with_component<T: Component>(
    result: &mut Vec<Entity>,
    children: &Children,
    hierarchy_query: &Query<(&Children, Option<&T>)>,
    component_query: &Query<&T>,
) {
    for child in children {
        if let Ok((children, component_option)) = hierarchy_query.get(*child) {
            if component_option.is_some() {
                result.push(*child);
            } 
            find_children_with_component(result, children, hierarchy_query, component_query);
        } else if let Ok(_) = component_query.get(*child) {
            result.push(*child);
        }
    }
}