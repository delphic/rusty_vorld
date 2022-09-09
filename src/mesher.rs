use std::{collections::HashMap, convert::TryInto};
use super::voxel::{CHUNK_SIZE, Cardinal};
use super::voxel;
use bevy::{
    prelude::*,
    render::mesh::Indices,
    render::{mesh::Mesh, render_resource::PrimitiveTopology},
};

// pub fn build_tile() -> Mesh {
//     let vertices = [
//         ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
//         ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
//         ([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
//         ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
//     ];

//     let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
//     let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
//     let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

//     let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

//     let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
//     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
//     mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
//     mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
//     mesh.set_indices(Some(indices));
//     mesh
// }

fn insert_tile(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    direction: Cardinal,
    position: (u8, u8, u8),
) {
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
        // right
        ([1.0, 0.0, 0.0], [1.0, 1.0]),
        ([1.0, 1.0, 0.0], [1.0, 0.0]),
        ([1.0, 1.0, 1.0], [0.0, 0.0]),
        ([1.0, 0.0, 1.0], [0.0, 1.0]),
        // left
        ([0.0, 0.0, 0.0], [0.0, 1.0]),
        ([0.0, 0.0, 1.0], [1.0, 1.0]),
        ([0.0, 1.0, 1.0], [1.0, 0.0]),
        ([0.0, 1.0, 0.0], [0.0, 0.0]),
    ];

    for _ in 0..4 {
        normals.push(match direction {
            Cardinal::Forward => [0.0, 0.0, 1.0],
            Cardinal::Back => [0.0, 0.0, -1.0],
            Cardinal::Left => [-1.0, 0.0, 0.0],
            Cardinal::Right => [1.0, 0.0, 0.0],
            Cardinal::Up => [0.0, 1.0, 0.0],
            Cardinal::Down => [0.0, -1.0, 0.0],
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
    config: &voxel::VoxelConfig,
    voxel: u8,
    direction: Cardinal,
    position: (u8, u8, u8),
    tile_requests: &mut HashMap<u32, Vec<(Cardinal, (u8, u8, u8))>>,
) {
    if let Some(lookup) = config.id_to_tile.get(&voxel) {
        if let Some(tile_id) = lookup.get(&direction) {
            if let Some(positions) = tile_requests.get_mut(tile_id) {
                positions.push((direction, position));
            } else {
                tile_requests.insert(*tile_id, Vec::from([(direction, position)]));
            }
        }
    }
}

pub fn build_chunk_meshes(chunk: &voxel::Chunk, config: &voxel::VoxelConfig) -> Vec<(u32, Mesh)> {
    // Build map of tiles required with direction and position
    let mut tile_requests : HashMap<u32, Vec<(voxel::Cardinal, (u8, u8, u8))>> = HashMap::new();
    for i in 0..chunk.voxels.len() {
        let voxel = chunk.voxels[i]; 
        if voxel != 0 {
            let x = (i % voxel::CHUNK_SIZE).try_into().unwrap();
            let y = (i / (CHUNK_SIZE * CHUNK_SIZE)).try_into().unwrap();
            let z = ((i / CHUNK_SIZE) % voxel::CHUNK_SIZE).try_into().unwrap();
            let position = (x, y, z);

            if x == 0 || chunk.voxels[i-1] == 0 {
                request_tile(config, voxel, Cardinal::Left, position, &mut tile_requests);
            }
            if y == 0 || chunk.voxels[i - CHUNK_SIZE*CHUNK_SIZE] == 0 {
                request_tile(config, voxel, Cardinal::Down, position, &mut tile_requests);
            }
            if z == 0 || chunk.voxels[i - CHUNK_SIZE] == 0 {
                request_tile(config, voxel, Cardinal::Back, position, &mut tile_requests);
            }
            if x == 15 || chunk.voxels[i+1] == 0 {
                request_tile(config, voxel, Cardinal::Right, position, &mut tile_requests)
            }
            if y == 15 || chunk.voxels[i+ CHUNK_SIZE*CHUNK_SIZE] == 0 {
                request_tile(config, voxel, Cardinal::Up, position, &mut tile_requests)
            }
            if z == 15 || chunk.voxels[i + CHUNK_SIZE] == 0 {
                request_tile(config, voxel, Cardinal::Forward, position, &mut tile_requests)
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


