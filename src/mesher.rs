use bevy::{render::mesh::Indices, render::{mesh::Mesh, render_resource::PrimitiveTopology}};

pub fn build_tile(tile_index: u8, atlas_size: u8) -> Mesh {

    let v_factor = 1.0 / atlas_size as f32;
    let v_offset = tile_index as f32 * v_factor;

    let vertices = [
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0 * v_factor + v_offset]),
        ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0 * v_factor + v_offset]),
        ([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0 * v_factor + v_offset]),
        ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0 * v_factor + v_offset]),
    ];

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3,]);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}