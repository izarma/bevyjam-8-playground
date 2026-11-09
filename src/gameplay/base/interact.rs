use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_kira_audio::prelude::{Audio, AudioControl};
use bevy_yarnspinner::prelude::DialogueRunner;
use pixel_art::PixelSprite;

use super::{
    anim::IndoorCharAnimation,
    npc::{BodyOf, InDialogue},
    player_indoor::Touch,
};
use crate::{
    GameplayScene, InControl, PausableSystems,
    gameplay::{DialogueTarget, Met, Player, PlayerAnimation, PlayerAnimationSystems, Visitor},
    loading::UIAssets,
    ui::{
        billboard::{Billboards, Fade},
        font,
        input_prompt::{KEY_F, prompt},
        palette::LABEL_TEXT,
        scale::texels,
    },
};

const REACH: f32 = 2.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Gong>()
        .add_systems(
            Update,
            find_interactable_in_reach
                .in_set(PausableSystems)
                .run_if(in_state(GameplayScene::Base)),
        )
        .add_systems(
            Update,
            show_interaction_prompt
                .after(find_interactable_in_reach)
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

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Gong;

#[derive(Component)]
pub(super) struct InReach;

fn find_interactable_in_reach(
    mut commands: Commands,
    player: Option<Single<&GlobalTransform, With<Player>>>,
    targets: Query<(Entity, &GlobalTransform, Has<InReach>), Or<(With<BodyOf>, With<Gong>)>>,
) {
    let ground = |at: &GlobalTransform| Vec2::new(at.translation().x, at.translation().z);
    let nearest = player.and_then(|player| {
        let player = ground(&player);
        targets
            .iter()
            .map(|(npc, at, _)| (npc, ground(at).distance(player)))
            .filter(|(_, distance)| *distance <= REACH)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(npc, _)| npc)
    });
    for (npc, _, in_reach) in &targets {
        if in_reach && Some(npc) != nearest {
            commands.entity(npc).remove::<InReach>();
        } else if !in_reach && Some(npc) == nearest {
            commands.entity(npc).insert(InReach);
        }
    }
}

#[derive(Component)]
struct InteractionPrompt {
    target: Entity,
}

const INTERACTION_CANVAS: UVec2 = UVec2::new(64, 26);

fn show_interaction_prompt(
    in_reach: Option<Single<Entity, With<InReach>>>,
    in_control: Option<Res<State<InControl>>>,
    runners: Query<&DialogueRunner>,
    mut prompts: Query<(&InteractionPrompt, &mut Fade)>,
    targets: Query<
        (&GlobalTransform, Has<Gong>, Option<&Children>),
        Or<(With<BodyOf>, With<Gong>)>,
    >,
    sprites: Query<(&PixelSprite, &GlobalTransform)>,
    mut billboards: Billboards,
) {
    // hidden while a menu is up or a dialogue is running
    let talking = runners.iter().any(|runner| runner.is_running());
    let target = in_reach
        .map(|npc| *npc)
        .filter(|_| in_control.is_some() && !talking);
    let mut showing = false;
    for (shown, mut fade) in &mut prompts {
        if Some(shown.target) == target {
            fade.stay();
            showing = true;
        } else {
            fade.leave();
        }
    }
    let Some(npc) = target else {
        return;
    };
    if showing {
        return;
    }
    let Ok((at, gong, children)) = targets.get(npc) else {
        return;
    };
    let above = if gong {
        Some(at.translation() + Vec3::Y * 2.2)
    } else {
        children
            .into_iter()
            .flatten()
            .find_map(|&child| sprites.get(child).ok())
            .and_then(|(sprite, at)| billboards.above(sprite, at))
    };
    let Some(above) = above else {
        return;
    };
    billboards
        .spawn(
            INTERACTION_CANVAS,
            above,
            interaction_panel(if gong { "Ring" } else { "Talk" }),
            GameplayScene::Base,
        )
        .insert((
            Name::new("Interaction Prompt"),
            InteractionPrompt { target: npc },
        ));
}

fn interaction_panel(label: &'static str) -> impl Scene {
    bsn! {
        #TalkCanvas
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
        }
        Pickable::IGNORE
        Children [
            Node {
                align_items: AlignItems::Center,
            }
            Children [
                Text(label)
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

fn start_interaction(
    _: On<Start<Touch>>,
    targets: Query<(Entity, Has<Gong>), With<InReach>>,
    in_dialogue: Query<Entity, With<InDialogue>>,
    bodies: Query<&BodyOf>,
    visitors: Query<&Visitor>,
    mut runner: Query<&mut DialogueRunner>,
    mut target: ResMut<DialogueTarget>,
    mut commands: Commands,
    assets: Res<UIAssets>,
    audio: Res<Audio>,
) {
    let Some((npc, gong)) = targets.iter().next() else {
        return;
    };
    let Ok(mut runner) = runner.single_mut() else {
        return;
    };
    if runner.is_running() {
        return;
    }
    if gong {
        audio.play(assets.gong.clone());
        return;
    }
    let Some(body) = bodies.get(npc).ok() else {
        error!("Refusing to start dialogue with {npc:?}: no body to talk to");
        return;
    };
    let Some(node) = visitors.get(body.0).ok().map(Visitor::node) else {
        error!("Refusing to start dialogue with {npc:?}: no visitor to talk to");
        return;
    };
    if !runner.node_exists(node) {
        error!(
            "Refusing to start dialogue with {npc:?}: node \"{node}\" is missing from the compiled Yarn project"
        );
        return;
    }
    match runner.try_start_node(node) {
        Ok(_) => {
            info!("interacting with NPC {npc:?}");
            // Replace the previous visitor so first-match consumers can never pick it up.
            target.0 = Some(body.0);
            for other in &in_dialogue {
                if other != npc {
                    commands.entity(other).remove::<InDialogue>();
                }
            }
            commands.entity(npc).insert(InDialogue);
            commands.entity(body.0).insert(Met);
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
    added: On<Add<InDialogue>>,
    player: Query<&GlobalTransform, With<Player>>,
    npcs: Query<&GlobalTransform>,
    mut player_animations: Query<&mut PlayerAnimation>,
) {
    let (Ok(player), Ok(npc)) = (player.single(), npcs.get(added.entity)) else {
        return;
    };

    let player_pos = Vec2::new(player.translation().x, player.translation().z);
    let npc_pos = Vec2::new(npc.translation().x, npc.translation().z);

    if let Ok(mut animation) = player_animations.single_mut() {
        animation.face_towards(npc_pos - player_pos);
    }
}
