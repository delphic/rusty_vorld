use bevy::{render::mesh::Indices, render::{mesh::Mesh, render_resource::PrimitiveTopology}};

pub fn build_tile() -> Mesh {

    let vertices = [
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
        ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
        ([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
        ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
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