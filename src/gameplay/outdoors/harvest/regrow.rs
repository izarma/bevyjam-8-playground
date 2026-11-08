// respawn timers for plants
use bevy::prelude::*;
use jackdaw_scene_types::node_id::SceneNodeId;
use pixel_art::PixelSprite;

use super::{
    super::biomes::{Fields, Harvested, REGROW_HOURS},
    InReach,
};
use crate::{
    GameplayScene, InControl,
    gameplay::clock::Clock,
    ui::{
        billboard::Billboards,
        font,
        palette::{GAIN_TEXT, LABEL_TEXT},
        scale::texels,
        widget,
    },
};

const CANVAS: UVec2 = UVec2::new(64, 24);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (show_panel, show_progress)
            .chain()
            .after(super::find_in_reach)
            .run_if(in_state(GameplayScene::Outdoors)),
    );
}

#[derive(Component)]
struct RegrowPanel {
    plant: Entity,
}

#[derive(Component, Clone, Default)]
struct RegrowFill;

#[derive(Component, Clone, Default)]
struct RegrowHours;

fn show_panel(
    mut commands: Commands,
    in_reach: Option<Single<Entity, With<InReach>>>,
    in_control: Option<Res<State<InControl>>>,
    panels: Query<(Entity, &RegrowPanel)>,
    plants: Query<(&GlobalTransform, &PixelSprite), With<Harvested>>,
    mut billboards: Billboards,
) {
    // hidden while a menu or minigame is up, and over plants ready to pick
    let target = in_reach
        .map(|plant| *plant)
        .filter(|&plant| in_control.is_some() && plants.contains(plant));
    for (panel, shown) in &panels {
        if Some(shown.plant) != target {
            commands.entity(panel).despawn();
        }
    }
    let Some(plant) = target else {
        return;
    };
    if panels.iter().any(|(_, shown)| shown.plant == plant) {
        return;
    }
    let Ok((at, sprite)) = plants.get(plant) else {
        return;
    };
    let Some(above) = billboards.above(sprite, at) else {
        return;
    };
    billboards
        .spawn(CANVAS, above, panel(), GameplayScene::Outdoors)
        .insert((Name::new("Regrow Panel"), RegrowPanel { plant }));
}

fn panel() -> impl Scene {
    bsn! {
        #RegrowCanvas
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
        }
        Pickable::IGNORE
        Children [
            Node {
                padding: UiRect::axes(texels(4), texels(2)),
                column_gap: texels(3),
                align_items: AlignItems::Center,
            }
            @widget::panel()
            Pickable::IGNORE
            Children [
                Node {
                    width: texels(24),
                    height: texels(4),
                }
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5))
                Pickable::IGNORE
                Children [
                    RegrowFill
                    Node {
                        width: percent(0),
                        height: percent(100),
                    }
                    BackgroundColor(GAIN_TEXT)
                    Pickable::IGNORE
                ]
                --
                RegrowHours
                Text("")
                @font::text(1)
                TextColor(LABEL_TEXT)
                Pickable::IGNORE
            ]
        ]
    }
}

fn show_progress(
    panels: Query<&RegrowPanel>,
    plants: Query<&ChildOf>,
    spots: Query<&SceneNodeId>,
    fields: Single<&Fields>,
    clock: Single<&Clock>,
    mut fill: Query<&mut Node, With<RegrowFill>>,
    mut hours: Query<&mut Text, With<RegrowHours>>,
) {
    let Some(ready) = panels
        .iter()
        .next()
        .and_then(|panel| plants.get(panel.plant).ok())
        .and_then(|spot| spots.get(spot.parent()).ok())
        .and_then(|&spot| fields.ready(spot))
    else {
        return;
    };
    let left = (ready - clock.hours).max(0.0);
    let grown = percent(100.0 * (1.0 - left / REGROW_HOURS));
    for mut node in &mut fill {
        if node.width != grown {
            node.width = grown;
        }
    }
    let shown = format!("{}h", left.ceil());
    for mut text in &mut hours {
        if text.0 != shown {
            text.0 = shown.clone();
        }
    }
}
