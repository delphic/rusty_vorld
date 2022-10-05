use bevy::prelude::*;

use super::health::TakeDamageEvent;

#[derive(Component)]
pub struct HitFlash {
    pub duration: f32,
    pub material: Handle<StandardMaterial>,
    pub target_color: Color,
    pub from_color: Color,
    pub elasped: f32,
}

#[derive(Component)]
pub struct HitFlashSupport {
    pub material: Handle<StandardMaterial>,
    pub base_color: Color,
    pub flash_color: Color,
}

pub struct HitFlashPlugin;

impl Plugin for HitFlashPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_hit_flash);
        app.add_system(handle_take_damage_event);
    }
}

fn handle_take_damage_event(
    mut commands: Commands,
    mut take_damage_event_reader: EventReader<TakeDamageEvent>,
    hit_support_query: Query<&HitFlashSupport>,
) {
    for event in take_damage_event_reader.iter() {
        if let Ok(hit_flash_support) = hit_support_query.get(event.entity) {
            commands.entity(event.entity).insert(HitFlash {
                duration: 0.25,
                elasped: 0.0,
                material: hit_flash_support.material.clone(),
                from_color: hit_flash_support.base_color,
                target_color: hit_flash_support.flash_color
            });
        }
    }
}

fn update_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut flash_query: Query<(Entity, &mut HitFlash)>,
) {
    for (entity, mut flash) in flash_query.iter_mut() {
        flash.elasped += time.delta_seconds();
        if flash.elasped <= flash.duration {
            if let Some(material) = material_assets.get_mut(&flash.material) {
                let ratio = 
                    if flash.elasped < 0.25 * flash.duration { 
                        flash.elasped / (0.25 * flash.duration)
                    } else if flash.elasped > 0.75 * flash.duration {
                        1.0 - (flash.elasped - 0.75 * flash.duration) / (0.25 * flash.duration)
                    } else {
                        1.0
                    };
                material.base_color = Color::from(Vec4::from(flash.from_color).lerp(Vec4::from(flash.target_color), ratio)); 
            }
        } else {
            if let Some(material) = material_assets.get_mut(&flash.material) {
                material.base_color = flash.from_color;
            }
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}