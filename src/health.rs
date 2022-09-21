use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub max_health: u32,
    pub current_health: u32,
}