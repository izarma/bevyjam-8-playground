// worldspace yield preview ui for gatherables
use bevy::prelude::*;
use pixel_art::PixelSprite;
use plants::{Growing, Plant};

use super::{InReach, billboard::Billboards};
use crate::{
    GameplayScene, InControl,
    gameplay::herbs,
    ui::{
        input_prompt::{KEY_F, prompt},
        scale::texels,
    },
};

const CANVAS: UVec2 = UVec2::new(128, 96);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        show_panel
            .after(super::find_in_reach)
            .run_if(in_state(GameplayScene::Outdoors)),
    );
}

#[derive(Component)]
struct YieldPanel {
    plant: Entity,
}

fn show_panel(
    mut commands: Commands,
    in_reach: Option<Single<Entity, With<InReach>>>,
    in_control: Option<Res<State<InControl>>>,
    panels: Query<(Entity, &YieldPanel)>,
    plants: Query<(&Growing, &GlobalTransform, &PixelSprite)>,
    mut billboards: Billboards,
) {
    // hidden while a menu or minigame is up
    let target = in_reach
        .map(|plant| *plant)
        .filter(|_| in_control.is_some());
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
    let Ok((Growing(kind), at, sprite)) = plants.get(plant) else {
        return;
    };
    let Some(above) = billboards.above(sprite, at) else {
        return;
    };
    billboards
        .spawn(CANVAS, above, panel(*kind))
        .insert((Name::new("Yield Panel"), YieldPanel { plant }));
}

fn panel(plant: Plant) -> impl Scene {
    bsn! {
        #YieldCanvas
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
        }
        Pickable::IGNORE
        Children [
            // the card stays centered over the plant, the key floats beside it
            Node
            Children [
                @herbs::card(plant)
                --
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(100),
                    top: texels(5),
                    margin: UiRect::left(texels(2)),
                }
                Children [ @prompt(KEY_F) ]
            ]
        ]
    }
}
