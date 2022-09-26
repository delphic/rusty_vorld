use bevy::prelude::*;
use crate::npc_spawner::NpcAssets;

use super::player::Player;
use super::utils;

#[derive(Debug, PartialEq, Eq)]
enum ZombieState {
    Idle,
    Seeking,
}

#[derive(Component)]
pub struct Zombie {
    state: ZombieState,
    animation_player_entity: Option<Entity>,
}

#[derive(Component)]
pub struct FindAnimationPlayerRequest;

impl Zombie {
    pub fn new() -> Self {
        Self { 
            state: ZombieState::Idle,
            animation_player_entity: None,
        }
    }
}

pub fn handle_find_animation_player_request(
    mut commands: Commands, 
    mut request_query: Query<(Entity, &mut Zombie), With<FindAnimationPlayerRequest>>,
    hierarchy_query: Query<(&Children, Option<&AnimationPlayer>)>,
) {
    for (entity, mut zombie) in request_query.iter_mut() {
        if let Ok((children, _)) = hierarchy_query.get(entity) {
            zombie.animation_player_entity = utils::find_child_entity_with_component(children, &hierarchy_query);
        }
        commands.entity(entity).remove::<FindAnimationPlayerRequest>();
    }
}

pub fn seek_brains(
    time: Res<Time>,
    npc_assets: Res<NpcAssets>,
    player_query: Query<&Transform, (With<Player>, Without<Zombie>)>,
    mut zombie_query: Query<(&mut Transform, &mut Zombie)>,
    mut animation_query: Query<&mut AnimationPlayer>,
) {
    if let Some(player_transform) = player_query.iter().last() {
        for (mut transform, mut zombie) in zombie_query.iter_mut() {
            let target_position = Vec3::new(player_transform.translation.x, transform.translation.y, player_transform.translation.z);
            let target_direction = target_position - transform.translation;
            let distance_sqr = target_direction.length_squared();
            let mut is_moving = false;

            if distance_sqr < 8.0 * 8.0 && distance_sqr > 1.0 {
                transform.look_at(target_position, Vec3::Y);
                transform.translation += target_direction.normalize() * time.delta_seconds();
                is_moving = true;
            }

            let new_state = match is_moving {
                true => ZombieState::Seeking,
                false => ZombieState::Idle,
            };
            
            if zombie.state != new_state {
                if let Some(animation_player_entity) = zombie.animation_player_entity {
                    if let Ok(mut animation_player) = animation_query.get_mut(animation_player_entity) {
                        if new_state == ZombieState::Seeking {
                            animation_player.play(npc_assets.walk_animation.clone_weak()).repeat();
                        } else {
                            animation_player.pause();
                        }
                    } else {
                        warn!("Unable to find animation player");
                    }
                }

                zombie.state = new_state;
            }
        }
    }
}