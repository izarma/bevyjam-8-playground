use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_ahoy::{AhoySystems, CharacterControllerState};
use bevy_kira_audio::prelude::*;

use crate::{
    GameState, GameplayScene, PausableSystems,
    gameplay::{Player, PlayerAnimation, PlayerAnimationSystems},
    loading::FootstepAssets,
};

mod surfaces;

use surfaces::{FootstepTerrain, FootstepTerrainLoader, GroundSurfaces, load_terrain_surfaces};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<FootstepSurface>()
        .init_resource::<FootstepSequence>()
        .init_asset::<FootstepTerrain>()
        .init_asset_loader::<FootstepTerrainLoader>()
        .add_systems(Update, load_terrain_surfaces)
        .add_systems(
            FixedPostUpdate,
            play_jump_sounds
                .after(AhoySystems::MoveCharacters)
                .in_set(PausableSystems)
                .run_if(in_state(GameState::Gameplay)),
        )
        .add_systems(
            Update,
            play_footsteps
                .after(PlayerAnimationSystems::UpdateState)
                .in_set(PausableSystems)
                .run_if(in_state(GameState::Gameplay)),
        );
}

#[derive(Component, Reflect, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component, Default)]
pub(crate) enum FootstepSurface {
    #[default]
    Dirt,
    Wood,
    WetDirt,
    Grass,
    Water,
    Stone,
    Sand,
}

impl FootstepSurface {
    fn from_material(material: &str) -> Option<Self> {
        match material {
            "material_1.bsn" => Some(Self::Dirt),
            "material_2.bsn" => Some(Self::Grass),
            "tiles_material/tiles_material.bsn" => Some(Self::Stone),
            "material_sand.bsn" => Some(Self::Sand),
            _ => None,
        }
    }

    fn clips(self, assets: &FootstepAssets) -> &[Handle<AudioSource>] {
        match self {
            Self::Dirt => &assets.dirt,
            Self::Wood => &assets.wood,
            Self::WetDirt => &assets.wet_dirt,
            Self::Grass => &assets.grass,
            Self::Water => &assets.water,
            Self::Stone => &assets.concrete,
            Self::Sand => &assets.sand,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Dirt => 0,
            Self::Wood => 1,
            Self::WetDirt => 2,
            Self::Grass => 3,
            Self::Water => 4,
            Self::Stone => 5,
            Self::Sand => 6,
        }
    }
}

#[derive(Resource, Default)]
struct FootstepSequence([usize; 7]);

impl FootstepSequence {
    fn play(
        &mut self,
        surface: FootstepSurface,
        assets: &FootstepAssets,
        audio: &Audio,
        volume_offset_db: f32,
    ) {
        let clips = surface.clips(assets);
        if let Some(index) = self.next(surface, clips.len()) {
            audio
                .play(clips[index].clone())
                .with_volume(volume_offset_db);
        }
    }

    fn next(&mut self, surface: FootstepSurface, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }
        let cursor = &mut self.0[surface.index()];
        let index = *cursor % len;
        *cursor = (index + 1) % len;
        Some(index)
    }
}

fn play_footsteps(
    assets: Res<FootstepAssets>,
    audio: Res<Audio>,
    animations: Query<(&PlayerAnimation, &ChildOf)>,
    players: Query<&CharacterControllerState, With<Player>>,
    ground_surfaces: GroundSurfaces,
    mut sequence: ResMut<FootstepSequence>,
    scene: Res<State<GameplayScene>>,
) {
    for (animation, parent) in &animations {
        if !animation.step_frame() {
            continue;
        }
        let Ok(player) = players.get(parent.parent()) else {
            continue;
        };
        let Some(ground) = &player.grounded else {
            continue;
        };
        let surface = ground_surfaces.at(*scene.get(), ground.entity, ground.point1);
        let Some(surface) = surface else {
            continue;
        };
        sequence.play(surface, &assets, &audio, 0.0);
    }
}

#[derive(Default)]
struct JumpFootsteps {
    player: Option<Entity>,
    grounded: bool,
    surface: Option<FootstepSurface>,
}

#[derive(Debug, PartialEq)]
enum JumpFootfall {
    Takeoff(Option<FootstepSurface>),
    Landing(Option<FootstepSurface>),
}

impl JumpFootsteps {
    fn update(
        &mut self,
        player: Entity,
        grounded: bool,
        surface: Option<FootstepSurface>,
        rising: bool,
    ) -> Option<JumpFootfall> {
        let footfall = if self.player != Some(player) {
            None
        } else if self.grounded && !grounded && rising {
            Some(JumpFootfall::Takeoff(self.surface))
        } else if !self.grounded && grounded {
            Some(JumpFootfall::Landing(surface))
        } else {
            None
        };
        self.player = Some(player);
        self.grounded = grounded;
        self.surface = surface;
        footfall
    }
}

fn play_jump_sounds(
    assets: Res<FootstepAssets>,
    audio: Res<Audio>,
    players: Query<(Entity, &CharacterControllerState, &LinearVelocity), With<Player>>,
    ground_surfaces: GroundSurfaces,
    mut sequence: ResMut<FootstepSequence>,
    scene: Res<State<GameplayScene>>,
    mut motion: Local<JumpFootsteps>,
) {
    let Ok((entity, player, velocity)) = players.single() else {
        *motion = JumpFootsteps::default();
        return;
    };
    let surface = player
        .grounded
        .as_ref()
        .and_then(|ground| ground_surfaces.at(*scene.get(), ground.entity, ground.point1));
    let rising = velocity.y > player.platform_velocity.y;
    let Some(footfall) = motion.update(entity, player.grounded.is_some(), surface, rising) else {
        return;
    };
    let (surface, volume_offset_db) = match footfall {
        JumpFootfall::Takeoff(surface) => {
            audio.play(assets.jump.clone()).with_volume(-3.0);
            (surface, 0.0)
        }
        JumpFootfall::Landing(surface) => (surface, 2.0),
    };
    if let Some(surface) = surface {
        sequence.play(surface, &assets, &audio, volume_offset_db);
    }
}
