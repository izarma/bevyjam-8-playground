//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use plants::Plant;

use crate::{
    GameState, Survivors,
    gameplay::{
        Concoction, DAY_END, DAY_START, DialogueTarget, Health, Remedy, Visitor, clock::Clock,
        inventory::Inventory,
    },
    loading_screen::Travel,
};

pub(super) fn plugin(app: &mut App) {
    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );

    // Debug-only win/lose triggers
    app.add_systems(
        Update,
        win.run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F1))),
    );
    app.add_systems(
        Update,
        lose.run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F2))),
    );

    // Debug-only NPC health editing.
    app.add_systems(
        Update,
        decrease_npc_health
            .run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F3))),
    );
    app.add_systems(
        Update,
        increase_npc_health
            .run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F4))),
    );

    app.add_systems(
        Update,
        grant_herbs.run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F5))),
    );
    app.add_systems(
        Update,
        prepare_remedy
            .run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F6))),
    );
    app.add_systems(
        Update,
        pass_hour.run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F8))),
    );
    app.add_systems(
        Update,
        force_spawn_npcs
            .run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F7))),
    );
}

fn grant_herbs(mut inventory: Single<&mut Inventory>) {
    for plant in Plant::ALL {
        *inventory.plants.entry(plant).or_default() += 3;
    }
    println!("Granted 3 of every herb");
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<GlobalUiDebugOptions>) {
    println!("Debug UI toggled");
    options.toggle();
}

fn win(mut commands: Commands, visitors: Query<&Visitor>) {
    commands.insert_resource(Survivors(visitors.iter().map(Visitor::name).collect()));
    commands.trigger(Travel::GameOver);
}

fn lose(mut commands: Commands) {
    commands.insert_resource(Survivors::default());
    commands.trigger(Travel::GameOver);
}

fn decrease_npc_health(target: Res<DialogueTarget>, visitors: Query<&mut Health>) {
    adjust_npc_health(target, visitors, -1);
}

fn increase_npc_health(target: Res<DialogueTarget>, visitors: Query<&mut Health>) {
    adjust_npc_health(target, visitors, 1);
}

fn adjust_npc_health(target: Res<DialogueTarget>, mut visitors: Query<&mut Health>, delta: i8) {
    let Some(mut health) = target.0.and_then(|visitor| visitors.get_mut(visitor).ok()) else {
        println!("Health editor: no NPC in dialogue");
        return;
    };
    health.offset(delta);
    println!("Health editor: all stats {delta:+}");
}

fn pass_hour(mut clock: Single<&mut Clock>) {
    clock.bank += 1.0;
    println!("Passed an hour");
}

fn prepare_remedy(
    target: Res<DialogueTarget>,
    remedies: Query<&Remedy>,
    mut concoctions: Query<&mut Concoction>,
    mut inventory: Single<&mut Inventory>,
) {
    if let Some(visitor) = target.0
        && let Ok(remedy) = remedies.get(visitor)
        && let Ok(mut concoction) = concoctions.single_mut()
    {
        concoction.prepare_remedy(remedy, &mut inventory);
    }
}

fn force_spawn_npcs(
    mut visitors: Query<&mut Health, With<Visitor>>,
    mut clock: Single<&mut Clock>,
) {
    for mut health in &mut visitors {
        if !health.dead() {
            health.make_unwell();
        }
    }
    let hour = clock.hours.rem_euclid(24.0);
    if !(DAY_START..DAY_END).contains(&hour) {
        clock.hours += (DAY_START - hour).rem_euclid(24.0);
    }
    println!("Force-spawned all living NPCs");
}
