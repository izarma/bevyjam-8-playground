use bevy::prelude::*;

use crate::{GameplayScene, PausableSystems, gameplay::Player, loading_screen::Travel};

// how close the player has to get to go inside
const REACH: f32 = 1.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        go_inside
            .in_set(PausableSystems)
            .run_if(in_state(GameplayScene::Outdoors)),
    );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct HouseDoor;

fn go_inside(
    player: Single<&GlobalTransform, With<Player>>,
    doors: Query<&GlobalTransform, With<HouseDoor>>,
    mut commands: Commands,
) {
    let ground = |at: &GlobalTransform| at.translation().xz();
    if doors
        .iter()
        .any(|door| ground(door).distance(ground(&player)) <= REACH)
    {
        commands.trigger(Travel::EnterHouse);
    }
}
