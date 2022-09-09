use super::atlas_loader::AtlasTexture;
use super::mesher;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Cardinal {
    Forward = 0,
    Back = 1,
    Up = 2,
    Down = 3,
    Right = 4,
    Left = 5,
}

pub struct VoxelConfig {
    pub id_to_tile: HashMap<u8, HashMap<Cardinal, u32>>,
}

// HACK: Atlas Texture Resource not immediately available so running setup as a system
// based on this struct as resource, should investigate startup stages to deal with this
// this properly
pub struct VoxelLoadStage {
    initialized: bool,
}

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_ARRAY_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub struct Chunk {
    pub voxels: [u8; CHUNK_ARRAY_SIZE],
}

impl Chunk {
    fn add_voxel(&mut self, id: u8, x: usize, y: usize, z: usize) {
        // Technically only need 4 bits for chunk size 16
        self.voxels[x + CHUNK_SIZE * z + CHUNK_SIZE * CHUNK_SIZE * y] = id;
    }
}

pub fn init(app: &mut App) {
    let mut look_up = HashMap::new();
    look_up.insert(
        1,
        HashMap::from([
            (Cardinal::Forward, 1),
            (Cardinal::Back, 1),
            (Cardinal::Up, 0),
            (Cardinal::Down, 2),
            (Cardinal::Right, 1),
            (Cardinal::Left, 1),
        ]),
    );

    app.insert_resource(VoxelConfig { id_to_tile: look_up });
    app.insert_resource(VoxelLoadStage { initialized: false });
    app.add_system(setup); // TODO: Should be startup system c.f. VoxelLoadStage
}

pub fn setup(
    mut commands: Commands,
    atlas: Res<AtlasTexture>,
    voxel_config: Res<VoxelConfig>,
    mut load_stage: ResMut<VoxelLoadStage>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if load_stage.initialized {
        return;
    }

    load_stage.initialized = true;
    let mut chunk = Chunk {
        voxels: [0; CHUNK_ARRAY_SIZE],
    };
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            chunk.add_voxel(1, x, 0, z);
        }
    }
    let mut tile_meshes = mesher::build_chunk_meshes(&chunk, &voxel_config);

    while let Some((tile_id, mesh)) = tile_meshes.pop() {
        commands.spawn_bundle(MaterialMeshBundle {
            mesh: meshes.add(mesh),
            material: atlas.materials[&tile_id].clone(),
            transform: Transform::from_xyz(0.0, 5.0, -4.0),
            ..default()
        });
    }
}
