#![allow(dead_code)]
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
            RenderPipelineDescriptor, SpecializedMeshPipelineError, ShaderRef,
        },
    },
    sprite::{Material2d, Material2dKey, MaterialMesh2dBundle, COLOR_MATERIAL_SHADER_HANDLE, ColorMaterialUniform},
};

#[derive(TypeUuid, Clone, AsBindGroup, Debug)]
#[uuid = "9548cd40-3262-4c8f-ad68-b4d778be26f0"]
#[uniform(0, ColorMaterialUniform)]
pub struct Glow1Material {
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl bevy::render::render_resource::AsBindGroupShaderType<ColorMaterialUniform> for Glow1Material {
    fn as_bind_group_shader_type(&self, _images: &bevy::render::render_asset::RenderAssets<Image>) -> ColorMaterialUniform {
        let mut flags = bevy::sprite::ColorMaterialFlags::NONE;
        if self.texture.is_some() {
            flags |= bevy::sprite::ColorMaterialFlags::TEXTURE;
        }

        ColorMaterialUniform {
            color: self.color.as_linear_rgba_f32().into(),
            flags: flags.bits(),
        }
    }
}

impl Material2d for Glow1Material {
    fn fragment_shader() -> ShaderRef {
        COLOR_MATERIAL_SHADER_HANDLE.typed().into()
    }
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                });
            }
        }
        Ok(())
    }
}

impl From<Handle<Image>> for Glow1Material {
    fn from(value: Handle<Image>) -> Self {
        Self {
            color: Color::default(),
            texture: Some(value),
        }
    }
}

pub fn startup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<Glow1Material>>,
    mut materials2: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    let mut mesh = Mesh::from(shape::Quad::default());
    // let (x, y, w, h) = (277, 113, 42, 33);

    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.5, 1.], [0.5, 0.5], [1., 0.5], [1., 1.]]);
    commands.spawn(
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh.clone()).into(),
            material: materials.add(Glow1Material {
                color: Color::rgba(1., 1., 1., 0.5),
                texture: Some(server.load("org/unit/693/f/693_f.png")),
            }),
            transform: Transform::from_xyz(0., 0., 1.).with_scale(Vec3::new(300., 200., 0.)),
            ..default()
        },
    );
    let texture_handle: Handle<Image> = server.load("org/unit/693/c/693_c.png");
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        material: materials2.add(ColorMaterial::from(texture_handle)),
        transform: Transform::from_xyz(0., 100., 1.).with_scale(Vec3::splat(300.)),
        ..default()
    });
}

pub fn system() {}
#[cfg(test)]
mod test {
    use super::*;
    // use std::any::Any;
    #[test]
    fn typeid() {}

    #[test]
    fn material() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_startup_system(startup)
            .add_system(system);
    }
}
