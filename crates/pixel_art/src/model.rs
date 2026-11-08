// 3d model equivalent of the billboard material

use bevy::{
    asset::{LoadState, embedded_asset},
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
        .add_systems(PostUpdate, swap_to_pixel_materials)
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
            base: self.standard.get(source)?.clone(),
            extension: PixelModelExtension::default(),
        });
        self.cache.0.insert(source.id(), material.clone());
        Some(material)
    }
}

#[derive(SystemParam)]
pub struct UnswappedModels<'w, 's> {
    standard_meshes: Query<'w, 's, (Entity, &'static MeshMaterial3d<StandardMaterial>)>,
    tree: Query<'w, 's, (Option<&'static ChildOf>, Has<PixelModel>)>,
    assets: Res<'w, AssetServer>,
}

impl UnswappedModels<'_, '_> {
    fn under_model(&self, mut entity: Entity) -> bool {
        loop {
            let Ok((parent, is_model)) = self.tree.get(entity) else {
                return false;
            };
            if is_model {
                return true;
            }
            let Some(parent) = parent else {
                return false;
            };
            entity = parent.parent();
        }
    }

    fn iter(&self) -> impl Iterator<Item = (Entity, &Handle<StandardMaterial>)> {
        self.standard_meshes
            .iter()
            .filter(|&(entity, MeshMaterial3d(source))| {
                self.under_model(entity)
                    && !matches!(
                        self.assets.get_load_state(source.id()),
                        Some(LoadState::Failed(_))
                    )
            })
            .map(|(entity, MeshMaterial3d(source))| (entity, source))
    }

    pub fn any(&self) -> bool {
        self.iter().next().is_some()
    }
}

fn swap_to_pixel_materials(
    mut commands: Commands,
    unswapped: UnswappedModels,
    mut materials: PixelMaterials,
) {
    for (entity, source) in unswapped.iter() {
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
            .try_remove::<(MeshMaterial3d<PixelModelMaterial>, StandardSource)>()
            .try_insert(MeshMaterial3d(source.clone()));
    }
}
