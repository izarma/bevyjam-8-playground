use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_yarnspinner::prelude::DialogueRunner;

use super::{
    anim::IndoorCharAnimation,
    npc::{InDialogue, Npc},
    player_indoor::Touch,
};
use crate::{
    GameplayScene, PausableSystems,
    gameplay::{Player, PlayerAnimation, PlayerAnimationSystems},
};

const REACH: f32 = 2.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        find_npc_in_reach
            .in_set(PausableSystems)
            .run_if(in_state(GameplayScene::Base)),
    )
    .add_systems(
        Update,
        face_each_other
            .before(PlayerAnimationSystems::UpdateState)
            .in_set(PausableSystems)
            .run_if(in_state(GameplayScene::Base)),
    )
    .add_observer(start_interaction);
}

/// The nearest NPC the player can currently interact with.
#[derive(Component)]
struct InReach;

fn find_npc_in_reach(
    mut commands: Commands,
    player: Option<Single<&GlobalTransform, With<Player>>>,
    npcs: Query<(Entity, &GlobalTransform, Has<InReach>), With<Npc>>,
) {
    let ground = |at: &GlobalTransform| Vec2::new(at.translation().x, at.translation().z);
    let nearest = player.and_then(|player| {
        let player = ground(&player);
        npcs.iter()
            .map(|(npc, at, _)| (npc, ground(at).distance(player)))
            .filter(|(_, distance)| *distance <= REACH)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(npc, _)| npc)
    });
    for (npc, _, in_reach) in &npcs {
        if in_reach && Some(npc) != nearest {
            commands.entity(npc).remove::<InReach>();
        } else if !in_reach && Some(npc) == nearest {
            commands.entity(npc).insert(InReach);
        }
    }
}

/// Yarn node shown when the player interacts with an NPC.
const NPC_DIALOGUE_NODE: &str = "HelloWorld";

fn start_interaction(
    _: On<Start<Touch>>,
    npcs: Query<Entity, With<InReach>>,
    mut runner: Query<&mut DialogueRunner>,
    mut commands: Commands,
) {
    let Some(npc) = npcs.iter().next() else {
        return;
    };
    let Ok(mut runner) = runner.single_mut() else {
        return;
    };
    if runner.is_running() {
        return;
    }
    if !runner.node_exists(NPC_DIALOGUE_NODE) {
        error!(
            "Refusing to start dialogue with {npc:?}: node \"{NPC_DIALOGUE_NODE}\" is missing from the compiled Yarn project"
        );
        return;
    }
    match runner.try_start_node(NPC_DIALOGUE_NODE) {
        Ok(_) => {
            info!("interacting with NPC {npc:?}");
            commands.entity(npc).insert(InDialogue);
        }
        Err(error) => {
            // the player can just interact again.
            error!("Failed to start dialogue with {npc:?}: {error}");
            runner.stop();
        }
    }
}

/// Turns the player and the in-dialogue NPC toward each other once, on the
/// frame the dialogue starts, so their sprites read as facing one another.
fn face_each_other(
    runners: Query<&DialogueRunner>,
    player: Query<&GlobalTransform, With<Player>>,
    npcs: Query<(&GlobalTransform, &Children), With<InDialogue>>,
    mut player_animations: Query<&mut PlayerAnimation>,
    mut npc_animations: Query<&mut IndoorCharAnimation>,
    mut applied: Local<bool>,
) {
    let running = runners.iter().any(|runner| runner.is_running());
    if !running || *applied {
        *applied = running;
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let Some((npc, children)) = npcs.iter().next() else {
        return;
    };

    let player_pos = Vec2::new(player.translation().x, player.translation().z);
    let npc_pos = Vec2::new(npc.translation().x, npc.translation().z);

    if let Ok(mut animation) = player_animations.single_mut() {
        animation.face_towards(npc_pos - player_pos);
    }

    let to_player = player_pos - npc_pos;
    for child in children {
        if let Ok(mut animation) = npc_animations.get_mut(*child) {
            animation.face_towards(to_player);
        }
    }
    *applied = true;
}
