use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_ahoy::{input::AccumulatedInput, prelude::AhoySystems};
use bevy_yarnspinner::{events::DialogueCompleted, prelude::DialogueRunner};

use super::{
    InLineup, Player, Visitor,
    base::{BodyOf, NpcDoor, NpcSlot},
};
use crate::{
    GameState, GameplayScene, PausableSystems,
    camera::SceneCamera,
    loading_screen::{Arrived, Revealed, Travel},
    ui::{
        font,
        palette::{HEADER_TEXT, LABEL_TEXT},
        scale::texels,
    },
};

// The poem
const POEM_FADE_IN: f32 = 1.0;
const POEM_HOLD: f32 = 7.0;
const POEM_FADE_OUT: f32 = 1.0;
const POEM_SECONDS: f32 = POEM_FADE_IN + POEM_HOLD + POEM_FADE_OUT;
const POEM_DIM: f32 = 0.6;

// Release the intro camera once the guardian is within this distance of its slot.
const INTRO_ARRIVAL: f32 = 0.4;

const INTRO_WALK_X: f32 = -3.202_003_479_003_906;
const INTRO_WALK_Z: f32 = -2.410_432_815_551_758;
const INTRO_WALK_ARRIVAL: f32 = 0.15;

const CAM_X_MIN: f32 = -15.0;
const CAM_X_MAX: f32 = 15.0;
const CAM_REST_Y: f32 = 2.1;
const CAM_REST_Z: f32 = 6.2;
const CAM_TRACK_SPEED: f32 = 4.0;
const CAM_PAN_SPEED: f32 = 1.4;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Intro>()
        .add_observer(reveal_intro)
        .add_observer(door_finished)
        .add_systems(
            Update,
            (
                advance_poem,
                start_door,
                focus_door,
                follow_guardian,
                follow_walk,
            )
                .run_if(in_state(GameplayScene::Base)),
        )
        .add_systems(
            FixedPostUpdate,
            control_player_during_intro
                .before(AhoySystems::MoveCharacters)
                .in_set(PausableSystems)
                .run_if(in_state(GameplayScene::Base)),
        )
        .add_systems(OnExit(GameState::Gameplay), reset_intro);
}

#[derive(Resource, Default)]
pub(crate) struct Intro {
    stage: IntroStage,
    poem_elapsed: f32,
    door_started: bool,
    played: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum IntroStage {
    /// No intro is running yet
    #[default]
    Idle,
    Poem,
    Door,
    Guardian,
    Walk,
    Done,
}

impl Intro {
    /// The guardian stays outside until the intro reaches the point where it
    /// should walk in.
    pub(crate) fn guardian_may_enter(&self) -> bool {
        matches!(
            self.stage,
            IntroStage::Guardian | IntroStage::Walk | IntroStage::Done
        )
    }

    /// While true, the intro is playing and menus should stay closed.
    pub(crate) fn playing(&self) -> bool {
        !matches!(self.stage, IntroStage::Idle | IntroStage::Done)
    }

    /// While true, the intro owns the camera and `sync_camera` should stand down.
    pub(crate) fn controlling_camera(&self) -> bool {
        matches!(
            self.stage,
            IntroStage::Door | IntroStage::Guardian | IntroStage::Walk
        )
    }
}

#[derive(Component, Clone, Default)]
struct PoemOverlay;

#[derive(Component, Clone, Default)]
struct PoemLine;

#[derive(Component, Clone, Default)]
struct PoemSource;

/// Starts the intro the first time the screen reveals the base after leaving
/// the title screen.
fn reveal_intro(
    _: On<Revealed>,
    mut intro: ResMut<Intro>,
    mut commands: Commands,
    arrived: Query<&Arrived>,
) {
    let Ok(arrived) = arrived.single() else {
        return;
    };
    if intro.played || arrived.0 != Travel::Base {
        return;
    }
    intro.played = true;
    intro.stage = IntroStage::Poem;
    intro.poem_elapsed = 0.0;
    intro.door_started = false;
    spawn_poem(&mut commands);
}

fn spawn_poem(commands: &mut Commands) {
    commands.spawn_scene(bsn! {
        Name::new("Intro Poem")
        PoemOverlay
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: texels(4),
        }
        BackgroundColor(Color::NONE)
        GlobalZIndex(50)
        Pickable::IGNORE
        DespawnOnExit::<GameplayScene>(GameplayScene::Base)
        Children [
            PoemLine
            Text("He who accepts his state's reproach,")
            @font::text(1)
            TextColor(Color::NONE)
            Pickable::IGNORE
            --
            PoemLine
            Text("Is hailed therefore its altars' lord;")
            @font::text(1)
            TextColor(Color::NONE)
            Pickable::IGNORE
            --
            PoemLine
            Text("To him who bears men's direful woes")
            @font::text(1)
            TextColor(Color::NONE)
            Pickable::IGNORE
            --
            PoemLine
            Text("They all the name of King accord.")
            @font::text(1)
            TextColor(Color::NONE)
            Pickable::IGNORE
            --
            PoemSource
            Text("~ Lao Tzu, Tao Te Ching")
            @font::text(1)
            Node {
                margin: UiRect::left(texels(24)),
            }
            TextColor(Color::NONE)
            Pickable::IGNORE
        ]
    });
}

fn advance_poem(
    time: Res<Time>,
    mut intro: ResMut<Intro>,
    mut commands: Commands,
    overlays: Query<Entity, With<PoemOverlay>>,
    mut backdrops: Query<&mut BackgroundColor, With<PoemOverlay>>,
    mut lines: Query<&mut TextColor, With<PoemLine>>,
    mut sources: Query<&mut TextColor, (With<PoemSource>, Without<PoemLine>)>,
) {
    if intro.stage != IntroStage::Poem {
        return;
    }
    intro.poem_elapsed += time.delta_secs();
    let elapsed = intro.poem_elapsed;
    let alpha = if elapsed < POEM_FADE_IN {
        elapsed / POEM_FADE_IN
    } else if elapsed < POEM_FADE_IN + POEM_HOLD {
        1.0
    } else {
        ((POEM_SECONDS - elapsed) / POEM_FADE_OUT).clamp(0.0, 1.0)
    };
    for mut backdrop in &mut backdrops {
        backdrop.0 = Color::BLACK.with_alpha(POEM_DIM * alpha);
    }
    for mut line in &mut lines {
        line.0 = HEADER_TEXT.with_alpha(alpha);
    }
    for mut source in &mut sources {
        source.0 = LABEL_TEXT.with_alpha(alpha);
    }
    if elapsed >= POEM_SECONDS {
        for overlay in &overlays {
            commands.entity(overlay).despawn();
        }
        intro.stage = IntroStage::Door;
        intro.door_started = false;
    }
}

/// "Knock. Knock."
fn start_door(mut intro: ResMut<Intro>, mut runners: Query<&mut DialogueRunner>) {
    if intro.stage != IntroStage::Door || intro.door_started {
        return;
    }
    let Ok(mut runner) = runners.single_mut() else {
        return;
    };
    match runner.try_start_node("Door") {
        Ok(_) => intro.door_started = true,
        Err(error) => {
            error!("Failed to start the door intro dialogue: {error}");
            intro.stage = IntroStage::Guardian;
        }
    }
}

fn door_finished(_: On<DialogueCompleted>, mut intro: ResMut<Intro>) {
    if intro.stage == IntroStage::Door {
        intro.stage = IntroStage::Guardian;
    }
}

/// Frames the camera on the door while the knock is being answered, so the
/// "Knock. Knock." happens at the door rather than wherever the player stands.
fn focus_door(
    intro: Res<Intro>,
    door: Query<&GlobalTransform, With<NpcDoor>>,
    mut cameras: Query<&mut Transform, SceneCamera>,
    time: Res<Time>,
) {
    if intro.stage != IntroStage::Door {
        return;
    }
    let Ok(door) = door.single() else {
        return;
    };
    ease_camera(
        &mut cameras,
        door.translation().x,
        time.delta_secs(),
        CAM_PAN_SPEED,
    );
}

/// Pans the camera to the guardian as it walks in and follows it to its slot,
/// then hands control back to the normal camera.
fn follow_guardian(
    mut intro: ResMut<Intro>,
    guardian: Query<(Entity, &Visitor, Option<&InLineup>)>,
    bodies: Query<(&BodyOf, &GlobalTransform)>,
    slots: Query<(&NpcSlot, &GlobalTransform)>,
    mut cameras: Query<&mut Transform, SceneCamera>,
    time: Res<Time>,
) {
    if intro.stage != IntroStage::Guardian {
        return;
    }
    let Some((guardian_entity, _, lineup)) = guardian.iter().find(|(_, who, _)| who.guardian())
    else {
        return;
    };
    let Some(slot_pos) = lineup.and_then(|lineup| {
        slots
            .iter()
            .find(|(slot, _)| slot.0 == lineup.0)
            .map(|(_, at)| at.translation().xz())
    }) else {
        return;
    };
    let Some((_, at)) = bodies.iter().find(|(body, _)| body.0 == guardian_entity) else {
        return;
    };
    let at = at.translation();
    if at.xz().distance_squared(slot_pos) <= INTRO_ARRIVAL.powi(2) {
        intro.stage = IntroStage::Walk;
        return;
    }
    ease_camera(&mut cameras, at.x, time.delta_secs(), CAM_TRACK_SPEED);
}

fn follow_walk(
    mut intro: ResMut<Intro>,
    player: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    mut cameras: Query<&mut Transform, SceneCamera>,
    time: Res<Time>,
) {
    if intro.stage != IntroStage::Walk {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let target = Vec2::new(INTRO_WALK_X, INTRO_WALK_Z);
    if player.translation.xz().distance_squared(target) <= INTRO_WALK_ARRIVAL.powi(2) {
        intro.stage = IntroStage::Done;
        return;
    }
    ease_camera(&mut cameras, INTRO_WALK_X, time.delta_secs(), CAM_PAN_SPEED);
}

fn ease_camera(cameras: &mut Query<&mut Transform, SceneCamera>, x: f32, delta: f32, speed: f32) {
    let target_x = x.clamp(CAM_X_MIN, CAM_X_MAX);
    let t = 1.0 - (-speed * delta).exp();
    for mut camera in cameras.iter_mut() {
        camera.translation.x = camera.translation.x.lerp(target_x, t);
        camera.translation.y = camera.translation.y.lerp(CAM_REST_Y, t);
        camera.translation.z = camera.translation.z.lerp(CAM_REST_Z, t);
    }
}

/// Owns the player's input for the whole intro
fn control_player_during_intro(
    intro: Res<Intro>,
    mut player: Query<(&mut AccumulatedInput, &mut LinearVelocity, &Transform), With<Player>>,
) {
    for (mut input, mut velocity, transform) in &mut player {
        match intro.stage {
            IntroStage::Idle | IntroStage::Done => {}
            IntroStage::Walk => {
                let to_target = Vec2::new(INTRO_WALK_X, INTRO_WALK_Z) - transform.translation.xz();
                let reached = to_target.length_squared() <= INTRO_WALK_ARRIVAL.powi(2);
                input.last_movement = if reached {
                    None
                } else {
                    let direction = to_target.normalize();
                    Some(Vec2::new(direction.x, -direction.y))
                };
            }
            _ => {
                input.last_movement = None;
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
        }
    }
}

fn reset_intro(mut intro: ResMut<Intro>) {
    *intro = Intro::default();
}
