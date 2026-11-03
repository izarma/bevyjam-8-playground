// 3d model equivalent of the billboard material

use bevy::{
    asset::embedded_asset,
    ecs::system::SystemParam,
    pbr::{ExtendedMaterial, MaterialExtension},
    platform::collections::HashMap,
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

pub(crate) fn plugin(app: &mut App) {
    embedded_asset!(app, "model.wesl");
    app.add_plugins(MaterialPlugin::<PixelModelMaterial>::default())
        .init_resource::<PixelModelMaterials>()
        .add_systems(Update, swap_to_pixel_materials)
        .add_observer(restore_standard_materials);
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Default)]
pub struct PixelModel;

pub type PixelModelMaterial = ExtendedMaterial<StandardMaterial, PixelModelExtension>;

#[derive(Asset, AsBindGroup, Reflect, Clone, Default)]
pub struct PixelModelExtension {}

impl MaterialExtension for PixelModelExtension {
    fn fragment_shader() -> ShaderRef {
        "embedded://pixel_art/model.wesl".into()
    }
}

// the original standard material to replace if someone removes this material in the editor
#[derive(Component)]
struct StandardSource(Handle<StandardMaterial>);

/// one pixel material per standard material
#[derive(Resource, Default)]
struct PixelModelMaterials(HashMap<AssetId<StandardMaterial>, Handle<PixelModelMaterial>>);

#[derive(SystemParam)]
struct PixelMaterials<'w> {
    standard: Res<'w, Assets<StandardMaterial>>,
    pixel: ResMut<'w, Assets<PixelModelMaterial>>,
    cache: ResMut<'w, PixelModelMaterials>,
}

impl PixelMaterials<'_> {
    fn of(&mut self, source: &Handle<StandardMaterial>) -> Option<Handle<PixelModelMaterial>> {
        if let Some(material) = self.cache.0.get(&source.id()) {
            return Some(material.clone());
        }
        let material = self.pixel.add(ExtendedMaterial {
            base: StandardMaterial {
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                metallic: 0.0,
                ..self.standard.get(source)?.clone()
            },
            extension: PixelModelExtension::default(),
        });
        self.cache.0.insert(source.id(), material.clone());
        Some(material)
    }
}

fn swap_to_pixel_materials(
    mut commands: Commands,
    changed: Query<Entity, Changed<MeshMaterial3d<StandardMaterial>>>,
    added: Query<Entity, Added<PixelModel>>,
    standard_meshes: Query<&MeshMaterial3d<StandardMaterial>>,
    tree: Query<(Option<&ChildOf>, Has<PixelModel>)>,
    children: Query<&Children>,
    mut materials: PixelMaterials,
) {
    let under_model = |mut entity: Entity| loop {
        let Ok((parent, is_model)) = tree.get(entity) else {
            return false;
        };
        if is_model {
            return true;
        }
        let Some(parent) = parent else {
            return false;
        };
        entity = parent.parent();
    };
    let targets = changed.iter().filter(|&e| under_model(e)).chain(
        added
            .iter()
            .flat_map(|root| std::iter::once(root).chain(children.iter_descendants(root))),
    );
    for entity in targets {
        let Ok(MeshMaterial3d(source)) = standard_meshes.get(entity) else {
            continue;
        };
        let Some(material) = materials.of(source) else {
            continue;
        };
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((MeshMaterial3d(material), StandardSource(source.clone())));
    }
}

fn restore_standard_materials(
    removed: On<Remove<PixelModel>>,
    mut commands: Commands,
    children: Query<&Children>,
    sources: Query<&StandardSource>,
) {
    let root = removed.entity;
    for entity in std::iter::once(root).chain(children.iter_descendants(root)) {
        let Ok(StandardSource(source)) = sources.get(entity) else {
            continue;
        };
        commands
            .entity(entity)
            .remove::<(MeshMaterial3d<PixelModelMaterial>, StandardSource)>()
            .insert(MeshMaterial3d(source.clone()));
    }
}
