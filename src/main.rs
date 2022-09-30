use bevy::prelude::*;
use bevy_hanabi::*;
use bevy_rapier3d::prelude::*;

mod atlas_loader;
mod gun;
mod health;
mod hit_flash;
mod input;
mod lifetime;
mod mesher;
mod named_collision_groups;
mod npc_spawner;
mod player;
mod projectile;
mod scene_spawner;
mod smoothed_follow;
mod utils;
mod voxel;
mod zombie;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(HanabiPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(VorldPlugin)
        .run();
}

pub struct VorldPlugin;

impl Plugin for VorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState {
            cursor_locked: false,
        });
        atlas_loader::init(app);
        voxel::init(app);
        input::insert_resources(app);
        app.add_event::<projectile::ProjectileImpactEvent>();
        app.add_event::<health::TakeDamageEvent>();
        app.add_startup_system(npc_spawner::setup);
        app.add_system(npc_spawner::handle_asset_load);
        app.add_system(npc_spawner::handle_find_animation_player_request);
        app.add_system(npc_spawner::handle_clone_model_materials_request);
        app.add_startup_system(scene_spawner::spawn_lighting);
        app.add_startup_system(gun::setup);

        app.add_system(grab_mouse);
        input::add_systems(app);
        player::add_systems(app);
        
        app.add_system(gun::shoot);
        app.add_startup_system(projectile::setup);
        app.add_system(projectile::detect_projectile_impact);
        app.add_system(health::handle_projectile_impact.after(projectile::detect_projectile_impact));
        app.add_system(lifetime::update);
        app.add_system(smoothed_follow::follow.after(player::update_look));

        app.add_system(hit_flash::update_hit_flash);
        app.add_system(hit_flash::handle_take_damage_event);

        app.add_system(zombie::seek_brains);
    }
}

pub struct GameState {
    pub cursor_locked: bool,
}

fn grab_mouse(
    mut windows: ResMut<Windows>,
    mut game_state: ResMut<GameState>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        window.set_cursor_visibility(false);
        window.set_cursor_lock_mode(true);
        game_state.cursor_locked = true;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        window.set_cursor_visibility(true);
        window.set_cursor_lock_mode(false);
        game_state.cursor_locked = false;
    }
}
