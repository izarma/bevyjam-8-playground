use avian3d::prelude::*;
use bevy::prelude::*;
use jackdaw_runtime::TerrainHeights;
use jackdaw_terrain::Heightmap;

const BIG_DROP: f32 = 1.5;
const DROP_REACH: f32 = 1.0;
const WALL_HEIGHT: f32 = 6.0;
const BURIED: f32 = 1.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, build_walls);
}

#[derive(Component)]
pub(super) struct DropWall(Entity);

fn build_walls(
    mut commands: Commands,
    terrains: Query<(Entity, &TerrainHeights), Changed<TerrainHeights>>,
    walls: Query<(Entity, &DropWall)>,
) {
    for (terrain, heights) in &terrains {
        let standing = walls
            .iter()
            .find(|(_, wall)| wall.0 == terrain)
            .map(|(entity, _)| entity);
        let Some((collider, offset)) = wall_collider(&heights.0) else {
            if let Some(entity) = standing {
                commands.entity(entity).despawn();
            }
            continue;
        };
        let wall = (
            collider,
            RigidBody::Static,
            Transform::from_translation(offset),
        );
        match standing {
            Some(entity) => {
                commands.entity(entity).insert(wall);
            }
            None => {
                commands.spawn((DropWall(terrain), wall, ChildOf(terrain)));
            }
        }
    }
}

fn wall_collider(heightmap: &Heightmap) -> Option<(Collider, Vec3)> {
    let resolution = heightmap.resolution as usize;
    if resolution < 2 {
        return None;
    }
    let cell = heightmap.size.x / (resolution - 1) as f32;
    let reach = (DROP_REACH / cell).ceil() as usize;
    let height = |x: usize, z: usize| heightmap.get_height(x as u32, z as u32);
    let lowest_along_z: Vec<Vec<f32>> = (0..resolution)
        .map(|x| {
            (0..resolution)
                .map(|z| {
                    (z.saturating_sub(reach)..=(z + reach).min(resolution - 1))
                        .map(|near| height(x, near))
                        .fold(f32::INFINITY, f32::min)
                })
                .collect()
        })
        .collect();
    let heights: Vec<Vec<f32>> = (0..resolution)
        .map(|x| {
            (0..resolution)
                .map(|z| {
                    let ground = height(x, z);
                    let lowest = (x.saturating_sub(reach)..=(x + reach).min(resolution - 1))
                        .map(|near| lowest_along_z[near][z])
                        .fold(f32::INFINITY, f32::min);
                    let edge = x == 0 || z == 0 || x == resolution - 1 || z == resolution - 1;
                    if edge || ground - lowest > BIG_DROP {
                        ground + WALL_HEIGHT
                    } else {
                        ground - BURIED
                    }
                })
                .collect()
        })
        .collect();
    let centre = heightmap.origin + heightmap.size / 2.0;
    Some((
        Collider::heightfield(heights, Vec3::new(heightmap.size.x, 1.0, heightmap.size.y)),
        Vec3::new(centre.x, 0.0, centre.y),
    ))
}
