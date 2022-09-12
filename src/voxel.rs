use super::atlas_loader::AtlasTexture;
use super::mesher;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Direction {
    Forward = 0,
    Back = 1,
    Up = 2,
    Down = 3,
    Left = 4,
    Right = 5,
}

pub struct VoxelConfig {
    // NOTE: direction is from the perspective of the voxel, not the observer (i.e. forward not front or perhaps not "left as I look at it" if front is the forward direction)
    pub id_to_tile: HashMap<u8, HashMap<Direction, u32>>,
}

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_ARRAY_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub struct Chunk {
    pub voxels: [u8; CHUNK_ARRAY_SIZE],
}

impl Chunk {
    fn add_voxel(&mut self, id: u8, x: usize, y: usize, z: usize) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.voxels[x + CHUNK_SIZE * z + CHUNK_SIZE * CHUNK_SIZE * y] = id;
        } else {
            panic!("Received add_voxel instruction outside chunk bounds");
        }
    }
}

#[repr(u8)]
#[allow(dead_code)]
enum BlockIds {
    Air = 0,
    Grass = 1,
    StoneBlocks = 2,
}

pub fn init(app: &mut App) {
    let mut look_up = HashMap::new();
    // GRASS
    look_up.insert(
        BlockIds::Grass as u8,
        HashMap::from([
            (Direction::Forward, 1),
            (Direction::Back, 1),
            (Direction::Up, 0),
            (Direction::Down, 2),
            (Direction::Left, 1),
            (Direction::Right, 1),
        ])
    );
    // STONE BLOCKS
    look_up.insert(
        BlockIds::StoneBlocks as u8,
        HashMap::from([
            (Direction::Forward, 4),
            (Direction::Back, 4),
            (Direction::Up, 5),
            (Direction::Down, 5),
            (Direction::Left, 4),
            (Direction::Right, 4),
        ])
    );

    app.insert_resource(VoxelConfig { id_to_tile: look_up });
    app.add_startup_system(setup);
}

pub fn setup(
    mut commands: Commands,
    atlas: Res<AtlasTexture>,
    voxel_config: Res<VoxelConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut chunk = Chunk {
        voxels: [0; CHUNK_ARRAY_SIZE],
    };
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            chunk.add_voxel(BlockIds::Grass as u8, x, 0, z);
        }
    }

    for x in 4..=12 {
        for z in 4..=12 {
            for y in 1..4 {
                if (x == 4 || x == 12 || z == 4 || z == 12) && !((x == 8 || x == 9) && z == 4 && y <= 2) {
                    chunk.add_voxel(BlockIds::StoneBlocks as u8, x, y, z);
                }
            }
            chunk.add_voxel(BlockIds::StoneBlocks as u8, x, 4, z);
        }
    }

    let mut tile_meshes = mesher::build_chunk_meshes(&chunk, &voxel_config);

    while let Some((tile_id, mesh)) = tile_meshes.pop() {
        let mut entity_commands = commands.spawn();
        if let Some(collider) = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh) {
            entity_commands.insert(collider);
        } else {
            error!("Unable to generate mesh collider");
        }
        entity_commands.insert_bundle(MaterialMeshBundle {
            mesh: meshes.add(mesh),
            material: atlas.materials[&tile_id].clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
    }
}
