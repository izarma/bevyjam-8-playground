use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    ecs::system::SystemParam,
    prelude::*,
};
use jackdaw_runtime::{JackdawSceneMember, JackdawSceneRoot, TerrainHeights};
use jackdaw_scene_types::{Brush, Terrain};
use jackdaw_terrain::{RegionTerrainData, sidecar};
use pixel_art::PixelWater;

use super::FootstepSurface;
use crate::GameplayScene;

#[derive(Asset, TypePath)]
pub(super) struct FootstepTerrain(pub(super) RegionTerrainData);

#[derive(Default, TypePath)]
pub(super) struct FootstepTerrainLoader;

impl AssetLoader for FootstepTerrainLoader {
    type Asset = FootstepTerrain;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(FootstepTerrain(sidecar::load(&bytes)?))
    }

    fn extensions(&self) -> &[&str] {
        &["jdterrain"]
    }
}

#[derive(Component)]
pub(super) struct TerrainFootsteps(pub(super) Handle<FootstepTerrain>);

pub(super) fn load_terrain_surfaces(
    mut commands: Commands,
    assets: Res<AssetServer>,
    terrains: Query<(Entity, &Terrain), Without<TerrainFootsteps>>,
    parents: Query<&ChildOf>,
    roots: Query<&JackdawSceneRoot>,
    members: Query<&JackdawSceneMember>,
) {
    for (entity, terrain) in &terrains {
        if terrain.data_path.is_empty() {
            continue;
        }
        let root = std::iter::once(entity)
            .chain(parents.iter_ancestors(entity))
            .find_map(|ancestor| {
                roots
                    .get(ancestor)
                    .ok()
                    .or_else(|| roots.get(members.get(ancestor).ok()?.root).ok())
            });
        let Some(scene_path) = root.and_then(|root| root.0.path()) else {
            continue;
        };
        let Some(directory) = scene_path.path().parent() else {
            continue;
        };
        let path = directory.join(&terrain.data_path);
        commands.entity(entity).insert(TerrainFootsteps(
            assets.load(path.to_string_lossy().into_owned()),
        ));
    }
}

impl FootstepTerrain {
    pub(super) fn surface_at(
        &self,
        heights: &TerrainHeights,
        local: Vec3,
    ) -> Option<FootstepSurface> {
        let grid = heights.0.world_to_grid(local.xz());
        let resolution = heights.0.resolution;
        if resolution < 2
            || grid.min_element() < 0.0
            || grid.max_element() > (resolution - 1) as f32
        {
            return None;
        }
        let x = grid.x as u32;
        let z = grid.y as u32;
        let control = self.0.regions.grid_control(x, z);
        let auto = self.0.autoterrain.sanitized();
        let slot = if auto.enabled && !control.manual() {
            let cell = heights.0.cell_size();
            let dx = (heights.0.get_height((x + 1).min(resolution - 1), z)
                - heights.0.get_height(x.saturating_sub(1), z))
                / (((x + 1).min(resolution - 1) - x.saturating_sub(1)) as f32 * cell.x);
            let dz = (heights.0.get_height(x, (z + 1).min(resolution - 1))
                - heights.0.get_height(x, z.saturating_sub(1)))
                / (((z + 1).min(resolution - 1) - z.saturating_sub(1)) as f32 * cell.y);
            let slope = Vec2::new(dx, dz).length().atan().to_degrees();
            if slope >= (auto.slope_start_deg + auto.slope_end_deg) * 0.5 {
                auto.slope_slot
            } else {
                auto.base_slot
            }
        } else if control.blend() >= 128 {
            control.overlay_id()
        } else {
            control.base_id()
        };
        FootstepSurface::from_material(&self.0.materials.get(slot as usize)?.material)
    }
}

#[derive(SystemParam)]
pub(super) struct GroundSurfaces<'w, 's> {
    parents: Query<'w, 's, &'static ChildOf>,
    surfaces: Query<'w, 's, &'static FootstepSurface>,
    terrains: Query<
        'w,
        's,
        (
            &'static TerrainFootsteps,
            &'static TerrainHeights,
            &'static GlobalTransform,
        ),
    >,
    maps: Res<'w, Assets<FootstepTerrain>>,
    waters: Query<'w, 's, (&'static Brush, &'static GlobalTransform), With<PixelWater>>,
}

impl GroundSurfaces<'_, '_> {
    pub(super) fn at(
        &self,
        scene: GameplayScene,
        collider: Entity,
        point: Vec3,
    ) -> Option<FootstepSurface> {
        if scene == GameplayScene::Base {
            return Some(FootstepSurface::Wood);
        }
        if self
            .waters
            .iter()
            .any(|(brush, transform)| water_covers(brush, transform, point))
        {
            return Some(FootstepSurface::Water);
        }
        std::iter::once(collider)
            .chain(self.parents.iter_ancestors(collider))
            .find_map(|entity| {
                if let Ok(surface) = self.surfaces.get(entity) {
                    return Some(*surface);
                }
                let (handle, heights, transform) = self.terrains.get(entity).ok()?;
                let local = transform.affine().inverse().transform_point3(point);
                self.maps.get(&handle.0)?.surface_at(heights, local)
            })
    }
}

pub(super) fn water_covers(brush: &Brush, transform: &GlobalTransform, point: Vec3) -> bool {
    if brush.faces.is_empty() {
        return false;
    }
    let inverse = transform.affine().inverse();
    let local = inverse.transform_point3(point);
    let up = inverse.transform_vector3(Vec3::Y);
    brush.faces.iter().all(|face| {
        let plane = &face.plane;
        let upward = plane.normal.dot(up);
        upward < -1e-5 || plane.normal.dot(local) <= plane.distance + upward.max(0.0) * 0.02
    })
}
