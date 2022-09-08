use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{ 
        render_resource::{AsBindGroup, ShaderRef},
        texture::ImageSampler
    }
};
use super::mesher;

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "b93807cc-8804-4849-a524-1ea18c409a3e"]
struct ArrayTextureMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    array_texture: Handle<Image>,
    #[uniform(2)]
    layer: f32,
}

impl Material for ArrayTextureMaterial {
    fn fragment_shader() -> ShaderRef {
         "shaders/voxel.wgsl".into()
    }
}

pub struct AtlasTexture {
    is_loaded: bool,
    image_handle: Handle<Image>,
    layers: u32,
}

pub fn init(app: &mut App) {
    app
        .add_plugin(MaterialPlugin::<ArrayTextureMaterial>::default())
        .add_startup_system(setup)
        .add_system(handle_atlas_load);
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
) {
    let atlas_handle = asset_server.load("images/atlas.png");
    let atlas_layers = 23;

    commands.insert_resource(AtlasTexture {
        is_loaded: false,
        image_handle: atlas_handle.clone(),
        layers: atlas_layers,
    });

    for i in 0..atlas_layers {
        // TODO: Should cache these in a look up on the atlas resource for other things to use
        let material =  materials.add(ArrayTextureMaterial {
            array_texture: atlas_handle.clone(),
            layer: i as f32
        });

        // Technically not part of an atlas loader - but you know we're testing right now
        let tile_mesh = meshes.add(mesher::build_tile());
        commands.spawn_bundle(MaterialMeshBundle {
            mesh: tile_mesh,
            material: material,
            transform: Transform::from_xyz(i as f32 * 1.5 - 20.0, 1.0, -4.0),
            ..default()
        });
    }
}

fn handle_atlas_load(
    mut image_assets: ResMut<Assets<Image>>,
    mut atlas: ResMut<AtlasTexture>
) {
    if !atlas.is_loaded {
        if let Some(image) = image_assets.get_mut(&atlas.image_handle) {
            atlas.is_loaded = true;
            image.reinterpret_stacked_2d_as_array(atlas.layers);
            image.sampler_descriptor = ImageSampler::nearest();
            // NOTE: trying to set sampler in response to EventReader<AssetEvent<Image>> AssetEvent::Created is ineffective
        }
    }
}



