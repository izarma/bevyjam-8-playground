use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_yarnspinner::prelude::DialogueRunner;

use super::{
    BODY_HEIGHT,
    anim::IndoorCharAnimation,
    npc::{BodyOf, InDialogue},
    player_indoor::Touch,
};
use crate::{
    GameplayScene, InControl, PausableSystems,
    gameplay::{CharacterName, Met, Player, PlayerAnimation, PlayerAnimationSystems},
    ui::{
        billboard::Billboards,
        font,
        input_prompt::{KEY_F, prompt},
        palette::LABEL_TEXT,
        scale::texels,
    },
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
        show_talk_prompt
            .after(find_npc_in_reach)
            .run_if(in_state(GameplayScene::Base)),
    )
    .add_systems(
        Update,
        face_in_reach
            .before(PlayerAnimationSystems::UpdateState)
            .in_set(PausableSystems)
            .run_if(in_state(GameplayScene::Base)),
    )
    .add_observer(start_interaction)
    .add_observer(face_npc);
}

/// The nearest NPC the player can currently interact with.
#[derive(Component)]
pub(super) struct InReach;

fn find_npc_in_reach(
    mut commands: Commands,
    player: Option<Single<&GlobalTransform, With<Player>>>,
    npcs: Query<(Entity, &GlobalTransform, Has<InReach>), With<BodyOf>>,
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

/// World-space "Talk" prompt floating over the nearest interactable NPC.
#[derive(Component)]
struct TalkPrompt {
    npc: Entity,
}

const TALK_CANVAS: UVec2 = UVec2::new(64, 26);

fn show_talk_prompt(
    mut commands: Commands,
    in_reach: Option<Single<Entity, With<InReach>>>,
    in_control: Option<Res<State<InControl>>>,
    runners: Query<&DialogueRunner>,
    prompts: Query<(Entity, &TalkPrompt)>,
    npcs: Query<&GlobalTransform, With<BodyOf>>,
    mut billboards: Billboards,
) {
    // hidden while a menu is up or a dialogue is running
    let talking = runners.iter().any(|runner| runner.is_running());
    let target = in_reach
        .map(|npc| *npc)
        .filter(|_| in_control.is_some() && !talking);
    for (prompt, shown) in &prompts {
        if Some(shown.npc) != target {
            commands.entity(prompt).despawn();
        }
    }
    let Some(npc) = target else {
        return;
    };
    if prompts.iter().any(|(_, shown)| shown.npc == npc) {
        return;
    }
    let Ok(at) = npcs.get(npc) else {
        return;
    };
    let above = at.translation() + Vec3::Y * (BODY_HEIGHT / 2.5);
    billboards
        .spawn(TALK_CANVAS, above, talk_panel(), GameplayScene::Base)
        .insert((Name::new("Talk Prompt"), TalkPrompt { npc }));
}

fn talk_panel() -> impl Scene {
    bsn! {
        #TalkCanvas
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
        }
        Pickable::IGNORE
        Children [
            Node {
                align_items: AlignItems::Center,
            }
            Children [
                Text("Talk")
                @font::text(1)
                TextColor(LABEL_TEXT)
                Pickable::IGNORE
                --
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(100),
                    top: texels(5),
                    margin: UiRect::left(texels(2)),
                }
                --
                @prompt(KEY_F)
            ]
        ]
    }
}

/// Starts the Yarn node named after the in-reach NPC when the player interacts.
fn start_interaction(
    _: On<Start<Touch>>,
    npcs: Query<(Entity, &CharacterName), With<InReach>>,
    bodies: Query<&BodyOf>,
    mut runner: Query<&mut DialogueRunner>,
    mut commands: Commands,
) {
    let Some((npc, name)) = npcs.iter().next() else {
        return;
    };
    let Ok(mut runner) = runner.single_mut() else {
        return;
    };
    if runner.is_running() {
        return;
    }
    let node = name.0;
    if !runner.node_exists(node) {
        error!(
            "Refusing to start dialogue with {npc:?}: node \"{node}\" is missing from the compiled Yarn project"
        );
        return;
    }
    match runner.try_start_node(node) {
        Ok(_) => {
            info!("interacting with NPC {npc:?}");
            commands.entity(npc).insert(InDialogue);
            if let Ok(body) = bodies.get(npc) {
                commands.entity(body.0).insert(Met);
            }
        }
        Err(error) => {
            // the player can just interact again.
            error!("Failed to start dialogue with {npc:?}: {error}");
            runner.stop();
        }
    }
}

/// Turns an NPC with an active talk prompt toward the player, so it reads as
/// facing the player before and throughout the dialogue.
fn face_in_reach(
    player: Option<Single<&GlobalTransform, With<Player>>>,
    npcs: Query<(&GlobalTransform, &Children), With<InReach>>,
    mut npc_animations: Query<&mut IndoorCharAnimation>,
) {
    let Some(player) = player else {
        return;
    };
    let player_pos = Vec2::new(player.translation().x, player.translation().z);
    for (npc, children) in &npcs {
        let npc_pos = Vec2::new(npc.translation().x, npc.translation().z);
        let to_player = player_pos - npc_pos;
        for child in children {
            if let Ok(mut animation) = npc_animations.get_mut(*child) {
                animation.face_towards(to_player);
            }
        }
    }
}

/// Turns the player toward the in-dialogue NPC once, on the frame the
/// dialogue starts.
fn face_npc(
    _: On<Add<InDialogue>>,
    player: Query<&GlobalTransform, With<Player>>,
    npcs: Query<&GlobalTransform, With<InDialogue>>,
    mut player_animations: Query<&mut PlayerAnimation>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let Some(npc) = npcs.iter().next() else {
        return;
    };

    let player_pos = Vec2::new(player.translation().x, player.translation().z);
    let npc_pos = Vec2::new(npc.translation().x, npc.translation().z);

    if let Ok(mut animation) = player_animations.single_mut() {
        animation.face_towards(npc_pos - player_pos);
    }
}
