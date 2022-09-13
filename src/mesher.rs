use std::{collections::HashMap, convert::TryInto};
use super::voxel::{CHUNK_SIZE, Direction};
use super::voxel;
use bevy::{
    prelude::*,
    render::mesh::Indices,
    render::{mesh::Mesh, render_resource::PrimitiveTopology},
};

fn insert_tile(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    direction: Direction,
    position: (usize, usize, usize),
) {
    // One could argue that forward should be -z and invert left and right,
    // as cameras look in the negative z direction and it's more intuative to think of a camera as looking 'forward'.
    let vertices = [
        // forward
        ([0.0, 0.0, 1.0], [0.0, 1.0]),
        ([1.0, 0.0, 1.0], [1.0, 1.0]),
        ([1.0, 1.0, 1.0], [1.0, 0.0]),
        ([0.0, 1.0, 1.0], [0.0, 0.0]),
        // back
        ([0.0, 0.0, 0.0], [1.0, 1.0]),
        ([0.0, 1.0, 0.0], [1.0, 0.0]),
        ([1.0, 1.0, 0.0], [0.0, 0.0]),
        ([1.0, 0.0, 0.0], [0.0, 1.0]),
        // up
        ([0.0, 1.0, 0.0], [0.0, 0.0]),
        ([0.0, 1.0, 1.0], [0.0, 1.0]),
        ([1.0, 1.0, 1.0], [1.0, 1.0]),
        ([1.0, 1.0, 0.0], [1.0, 0.0]),
        // down
        ([0.0, 0.0, 0.0], [1.0, 0.0]),
        ([1.0, 0.0, 0.0], [0.0, 0.0]),
        ([1.0, 0.0, 1.0], [0.0, 1.0]),
        ([0.0, 0.0, 1.0], [1.0, 1.0]),
        // left
        ([1.0, 0.0, 0.0], [1.0, 1.0]),
        ([1.0, 1.0, 0.0], [1.0, 0.0]),
        ([1.0, 1.0, 1.0], [0.0, 0.0]),
        ([1.0, 0.0, 1.0], [0.0, 1.0]),
        // right
        ([0.0, 0.0, 0.0], [0.0, 1.0]),
        ([0.0, 0.0, 1.0], [1.0, 1.0]),
        ([0.0, 1.0, 1.0], [1.0, 0.0]),
        ([0.0, 1.0, 0.0], [0.0, 0.0]),
    ];

    for _ in 0..4 {
        normals.push(match direction {
            Direction::Forward => [0.0, 0.0, 1.0],
            Direction::Back => [0.0, 0.0, -1.0],
            Direction::Left => [1.0, 0.0, 0.0],
            Direction::Right => [-1.0, 0.0, 0.0],
            Direction::Up => [0.0, 1.0, 0.0],
            Direction::Down => [0.0, -1.0, 0.0],
        });
    }

    let n : u32 = positions.len().try_into().unwrap();
    let position_offset = Vec3::new(position.0 as f32, position.1 as f32, position.2 as f32);
    let index_offset = direction as usize * 4;
    for i in 0..4 {
        positions.push((position_offset + Vec3::from(vertices[i + index_offset].0)).to_array());
        uvs.push(vertices[i + index_offset].1);
    }

    let quad_indices : Vec<u32> = vec![0, 1, 2, 0, 2, 3];
    for i in quad_indices {
        indices.push(n + i);
    }
}

fn request_tile(
    look_up: &[[u32; 6]; 256],
    voxel: u8,
    direction: Direction,
    position: (usize, usize, usize),
    tile_requests: &mut HashMap<u32, Vec<(Direction, (usize, usize, usize))>>,
) {
    let tile_id = look_up[voxel as usize][direction as usize];
    if let Some(positions) = tile_requests.get_mut(&tile_id) {
        positions.push((direction, position));
    } else {
        tile_requests.insert(tile_id, Vec::from([(direction, position)]));
    }
}

/// Builds a Vec of meshes one per tile id required for the chunk
/// Currently material per tile id as set by uniform, alternative is packing tile info into custom vertex format
pub fn build_chunk_meshes(chunk_slice: voxel::VorldSlice, look_up: [[u32; 6]; 256]) -> Vec<(u32, Mesh)> {
    // Build map of tiles required with direction and position
    let mut tile_requests : HashMap<u32, Vec<(voxel::Direction, (usize, usize, usize))>> = HashMap::new();
    let chunk = chunk_slice.chunk;
    for i in 0..chunk.voxels.len() {
        let voxel = chunk.voxels[i]; 
        if voxel != 0 {
            let x = i % voxel::CHUNK_SIZE;
            let y = i / (CHUNK_SIZE * CHUNK_SIZE);
            let z = (i / CHUNK_SIZE) % voxel::CHUNK_SIZE;

            // TODO: Just take usize position - it's not like u8 can't overflow the bounds
            let position = (x, y, z);

            if (x == 0 && (chunk_slice.right_chunk.is_none() || chunk_slice.right_chunk.unwrap().get_voxel(CHUNK_SIZE - 1, y, z) == 0))
            || (x != 0 && chunk.voxels[i-1] == 0) {
                request_tile(&look_up, voxel, Direction::Right, position, &mut tile_requests);
            }
            if (y == 0 && (chunk_slice.down_chunk.is_none() || chunk_slice.down_chunk.unwrap().get_voxel(x, CHUNK_SIZE - 1, z) == 0))
            || (y != 0 && chunk.voxels[i - CHUNK_SIZE * CHUNK_SIZE] == 0) {
                request_tile(&look_up, voxel, Direction::Down, position, &mut tile_requests);
            }
            if (z == 0 && (chunk_slice.back_chunk.is_none() || chunk_slice.back_chunk.unwrap().get_voxel(x, y, CHUNK_SIZE - 1) == 0))
            || (z != 0 && chunk.voxels[i - CHUNK_SIZE] == 0) {
                request_tile(&look_up, voxel, Direction::Back, position, &mut tile_requests);
            }
            if (x == 15 && (chunk_slice.left_chunk.is_none() || chunk_slice.left_chunk.unwrap().get_voxel(0, y, z) == 0))
            || (x != 15 && chunk.voxels[i+1] == 0) {
                request_tile(&look_up, voxel, Direction::Left, position, &mut tile_requests)
            }
            if (y == 15 && (chunk_slice.up_chunk.is_none() || chunk_slice.up_chunk.unwrap().get_voxel(x, 0, z) == 0))
            || (y != 15 && chunk.voxels[i+ CHUNK_SIZE*CHUNK_SIZE] == 0) {
                request_tile(&look_up, voxel, Direction::Up, position, &mut tile_requests)
            }
            if (z == 15 && (chunk_slice.forward_chunk.is_none() || chunk_slice.forward_chunk.unwrap().get_voxel(x, y, 0) == 0))
            || (z != 15 && chunk.voxels[i + CHUNK_SIZE] == 0) {
                request_tile(&look_up, voxel, Direction::Forward, position, &mut tile_requests)
            }
        }
    }

    let mut meshes : Vec<(u32, Mesh)> = Vec::new();

    for tile_id in tile_requests.keys() {
        let mut positions : Vec<[f32; 3]> = Vec::new();
        let mut normals : Vec<[f32;3]> = Vec::new();
        let mut uvs : Vec<[f32; 2]> = Vec::new();
        let mut indices : Vec<u32> = Vec::new();
        
        let requests = &tile_requests[tile_id];
        for request in requests {
            insert_tile(&mut positions, &mut normals, &mut uvs, &mut indices, request.0, request.1);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        meshes.push((*tile_id, mesh));
    }

    meshes
}


