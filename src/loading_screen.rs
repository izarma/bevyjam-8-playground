use crate::{
    GameState, GameplayScene,
    scene_ready::Loading,
    ui::{
        font,
        scale::{pixels_per_texel, texels},
    },
};
use bevy::{
    asset::{AssetPath, embedded_asset, embedded_path},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    ui_render::prelude::{MaterialNode, UiMaterial, UiMaterialPlugin},
    window::PrimaryWindow,
};

const SWEEP_SECONDS: f32 = 0.6;

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "loading_screen.wesl");
    app.add_plugins(UiMaterialPlugin::<LoadingScreenMaterial>::default())
        .add_systems(Startup, spawn_loading_screen)
        .add_observer(plan_trip)
        .add_systems(Update, sweep);
}

#[derive(Event, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Travel {
    Title,
    Base,
    Outdoors,
    GameOver,
    // through the house door, arriving at the lab's entrance
    EnterHouse,
    // out the lab's exit, arriving at the outdoor entrance
    LeaveHouse,
}

impl Travel {
    fn arrive(self, game: &mut NextState<GameState>, scene: &mut NextState<GameplayScene>) {
        let (state, gameplay_scene) = match self {
            Self::Title => (GameState::Title, None),
            Self::GameOver => (GameState::GameOver, None),
            Self::Base | Self::EnterHouse => (GameState::Gameplay, Some(GameplayScene::Base)),
            Self::Outdoors | Self::LeaveHouse => {
                (GameState::Gameplay, Some(GameplayScene::Outdoors))
            }
        };
        game.set_if_different(state);
        if let Some(gameplay_scene) = gameplay_scene {
            scene.set_if_different(gameplay_scene);
        }
    }
}

// where the screen is taking us once it covers everything. the world holds
// still meanwhile
#[derive(Component)]
pub(crate) struct Trip(Travel);

// the last trip the screen finished
#[derive(Component)]
pub(crate) struct Arrived(pub(crate) Travel);

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct LoadingScreenMaterial {
    #[uniform(0)]
    swept: f32,
    #[uniform(0)]
    pixels_per_texel: f32,
}

impl UiMaterial for LoadingScreenMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("loading_screen.wesl")).with_source("embedded"),
        )
    }
}

#[derive(Component, Default, Clone)]
struct LoadingLabel;

fn spawn_loading_screen(
    mut commands: Commands,
    mut materials: ResMut<Assets<LoadingScreenMaterial>>,
) {
    commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
            }
            GlobalZIndex(i32::MAX)
            Pickable::IGNORE
            Children [
                #LoadingLabel
                LoadingLabel
                Text("Loading...")
                @font::text(1)
                TextColor(Color::WHITE)
                Node {
                    position_type: PositionType::Absolute,
                    right: texels(4),
                    bottom: texels(4),
                }
                Visibility::Hidden
                Pickable::IGNORE
            ]
        })
        .insert(MaterialNode(materials.add(LoadingScreenMaterial {
            swept: 0.0,
            pixels_per_texel: 1.0,
        })));
}

fn plan_trip(
    travel: On<Travel>,
    screen: Single<Entity, With<MaterialNode<LoadingScreenMaterial>>>,
    mut commands: Commands,
) {
    commands.entity(*screen).insert(Trip(*travel));
}

fn sweep(
    mut commands: Commands,
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    loading: Query<(), Loading>,
    screen: Single<(Entity, &MaterialNode<LoadingScreenMaterial>, Option<&Trip>)>,
    mut label: Single<&mut Visibility, With<LoadingLabel>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut materials: ResMut<Assets<LoadingScreenMaterial>>,
    mut game: ResMut<NextState<GameState>>,
    mut scene: ResMut<NextState<GameplayScene>>,
) {
    let (entity, screen, trip) = *screen;
    let loading = !loading.is_empty() || *state == GameState::Loading;
    label.set_if_neq(if loading {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    let Some(material) = materials.get(&screen.0) else {
        return;
    };
    let step = time.delta_secs() / SWEEP_SECONDS;
    let swept = if trip.is_some() || loading {
        (material.swept - step).max(0.0)
    } else {
        (material.swept + step).min(1.0)
    };
    if let Some(Trip(travel)) = trip
        && swept == 0.0
    {
        travel.arrive(&mut game, &mut scene);
        commands
            .entity(entity)
            .remove::<Trip>()
            .insert(Arrived(*travel));
    }
    let pixels_per_texel = pixels_per_texel(&window);
    if (material.swept != swept || material.pixels_per_texel != pixels_per_texel)
        && let Some(mut material) = materials.get_mut(&screen.0)
    {
        material.swept = swept;
        material.pixels_per_texel = pixels_per_texel;
    }
}
