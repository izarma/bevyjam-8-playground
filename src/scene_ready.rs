use avian3d::prelude::Collider;
use bevy::{
    asset::LoadState,
    ecs::system::SystemParam,
    pbr::{PendingMeshMaterialQueues, PendingPrepassMeshMaterialQueues, PendingShadowQueues},
    prelude::*,
    render::{ExtractSchedule, MainWorld, RenderApp, render_resource::PipelineCache},
    world_serialization::{WorldAssetRoot, WorldInstance, WorldInstanceSpawner},
};
use jackdaw_runtime::{AwaitingModelSettings, JackdawSceneRoot, TerrainReady};
use jackdaw_scene_types::Terrain;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Last, settle);
    app.sub_app_mut(RenderApp).add_systems(
        ExtractSchedule,
        (
            report_drawn,
            #[cfg(debug_assertions)]
            warn_late_pipelines,
        )
            .chain(),
    );
}

#[derive(Component)]
pub(crate) struct SceneSettled;

#[derive(Component)]
pub(crate) struct SceneDrawn;

// a scene not yet drawn in full
pub(crate) type Loading = (With<JackdawSceneRoot>, Without<SceneDrawn>);

// every terrain carries its final material and has its ground collider
#[derive(SystemParam)]
struct Terrains<'w, 's> {
    terrains: Query<'w, 's, (Has<TerrainReady>, Option<&'static Children>), With<Terrain>>,
    colliders: Query<'w, 's, (), With<Collider>>,
}

impl Terrains<'_, '_> {
    fn ready(&self) -> bool {
        self.terrains.iter().all(|(surfaced, children)| {
            surfaced
                && children.is_some_and(|children| {
                    children.iter().any(|child| self.colliders.contains(child))
                })
        })
    }
}

fn settle(
    mut commands: Commands,
    unsettled: Query<(Entity, &JackdawSceneRoot), Without<SceneSettled>>,
    asset_server: Res<AssetServer>,
    models: Query<(&WorldAssetRoot, Option<&WorldInstance>)>,
    spawner: Res<WorldInstanceSpawner>,
    awaiting: Query<(), With<AwaitingModelSettings>>,
    terrains: Terrains,
) {
    if unsettled.is_empty() {
        return;
    }
    let models_spawned = models.iter().all(|(root, instance)| {
        instance.is_some_and(|instance| spawner.instance_is_ready(**instance))
            || matches!(asset_server.load_state(&root.0), LoadState::Failed(_))
    });
    if !models_spawned || !awaiting.is_empty() || !terrains.ready() {
        return;
    }
    for (entity, scene) in &unsettled {
        if matches!(
            asset_server.load_state(&scene.0),
            LoadState::Loaded | LoadState::Failed(_)
        ) {
            commands.entity(entity).insert(SceneSettled);
        }
    }
}

fn report_drawn(
    mut main_world: ResMut<MainWorld>,
    pipelines: Res<PipelineCache>,
    meshes: Res<PendingMeshMaterialQueues>,
    prepasses: Res<PendingPrepassMeshMaterialQueues>,
    shadows: Res<PendingShadowQueues>,
    mut settled_then: Local<Vec<Entity>>,
) {
    let waiting = pipelines.waiting_pipelines().next().is_some()
        || [&meshes.0, &prepasses.0, &shadows.0]
            .into_iter()
            .flat_map(|queues| queues.0.values())
            .any(|view| !view.current_frame.is_empty());
    if !waiting {
        for &scene in settled_then.iter() {
            if let Ok(mut scene) = main_world.get_entity_mut(scene) {
                scene.insert(SceneDrawn);
            }
        }
    }
    *settled_then = main_world
        .query_filtered::<Entity, (With<SceneSettled>, Without<SceneDrawn>)>()
        .iter(&main_world)
        .collect();
}

#[cfg(debug_assertions)]
fn warn_late_pipelines(
    mut main_world: ResMut<MainWorld>,
    pipelines: Res<PipelineCache>,
    mut known: Local<usize>,
) {
    use bevy::material::descriptor::PipelineDescriptor;

    let drawn = main_world
        .query_filtered::<(), (With<JackdawSceneRoot>, With<SceneDrawn>)>()
        .iter(&main_world)
        .next()
        .is_some();
    let undrawn = main_world
        .query_filtered::<(), (With<JackdawSceneRoot>, Without<SceneDrawn>)>()
        .iter(&main_world)
        .next()
        .is_some();
    if drawn && !undrawn {
        for pipeline in pipelines.pipelines().skip(*known) {
            let label = match &pipeline.descriptor {
                PipelineDescriptor::RenderPipelineDescriptor(d) => d.label.clone(),
                PipelineDescriptor::ComputePipelineDescriptor(d) => d.label.clone(),
                PipelineDescriptor::MeshPipelineDescriptor(d) => d.label.clone(),
            };
            warn!("pipeline compiled during play: {label:?}");
        }
    }
    *known = pipelines.pipelines().count();
}
