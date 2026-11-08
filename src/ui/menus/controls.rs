// menu inputs / non-player inputs
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_yarnspinner::prelude::DialogueRunner;

use crate::{GameState, Harvesting, Pause, gameplay::Concoction, ui::menus::Menu};

use super::pause::open_pause_menu;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<MenuControls>()
        .add_systems(Startup, spawn_controls)
        .add_observer(back)
        .add_observer(toggle_pause)
        .add_observer(toggle_inventory)
        .add_observer(toggle_journal);
}

#[derive(Component)]
struct MenuControls;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct Back;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct TogglePause;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct ToggleInventory;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct ToggleJournal;

fn spawn_controls(mut commands: Commands) {
    commands
        .spawn((Name::new("Menu Controls"), MenuControls))
        .queue_spawn_related_scenes::<Actions<MenuControls>>(bsn_list! {
            Action::<Back>::new()
            Bindings [ Binding(KeyCode::Escape) ]
            --
            Action::<TogglePause>::new()
            Bindings [ Binding(KeyCode::KeyP) ]
            --
            Action::<ToggleInventory>::new()
            Bindings [ Binding(KeyCode::KeyI) ]
            --
            Action::<ToggleJournal>::new()
            Bindings [ Binding(KeyCode::KeyJ) ]
        });
}

fn back(
    _: On<Start<Back>>,
    mut commands: Commands,
    menu: Res<State<Menu>>,
    screen: Res<State<GameState>>,
    harvesting: Option<Res<State<Harvesting>>>,
    mut runners: Query<&mut DialogueRunner>,
    mut concoctions: Query<&mut Concoction>,
) {
    let in_game = *screen.get() == GameState::Gameplay;
    let harvesting = harvesting.is_some_and(|harvesting| harvesting.0);
    let talking = runners.iter().any(|runner| runner.is_running());
    let brewing = concoctions
        .iter()
        .any(|concoction| concoction.brewing().is_some());
    match menu.get() {
        Menu::None if harvesting => commands.set_state(Harvesting(false)),
        Menu::None if brewing => {
            for mut concoction in &mut concoctions {
                concoction.hurry();
            }
        }
        Menu::None if talking => {
            for mut runner in &mut runners {
                runner.stop();
            }
        }
        Menu::None if in_game => open_pause_menu(&mut commands),
        Menu::Pause | Menu::Inventory | Menu::Journal => commands.set_state(Menu::None),
        Menu::Settings if in_game => commands.set_state(Menu::Pause),
        Menu::Settings | Menu::Credits => commands.set_state(Menu::Main),
        _ => {}
    }
}

fn toggle_pause(
    _: On<Start<TogglePause>>,
    mut commands: Commands,
    menu: Res<State<Menu>>,
    screen: Res<State<GameState>>,
) {
    if *screen.get() != GameState::Gameplay {
        return;
    }
    match menu.get() {
        Menu::None => open_pause_menu(&mut commands),
        _ => commands.set_state(Menu::None),
    }
}

fn toggle_inventory(
    _: On<Start<ToggleInventory>>,
    mut commands: Commands,
    menu: Res<State<Menu>>,
    screen: Res<State<GameState>>,
    harvesting: Option<Res<State<Harvesting>>>,
) {
    let harvesting = harvesting.is_some_and(|harvesting| harvesting.0);
    match menu.get() {
        Menu::None if *screen.get() == GameState::Gameplay && !harvesting => {
            commands.set_state(Pause(true));
            commands.set_state(Menu::Inventory);
        }
        Menu::Inventory => commands.set_state(Menu::None),
        _ => {}
    }
}

fn toggle_journal(
    _: On<Start<ToggleJournal>>,
    mut commands: Commands,
    menu: Res<State<Menu>>,
    screen: Res<State<GameState>>,
    harvesting: Option<Res<State<Harvesting>>>,
) {
    let harvesting = harvesting.is_some_and(|harvesting| harvesting.0);
    match menu.get() {
        Menu::None if *screen.get() == GameState::Gameplay && !harvesting => {
            commands.set_state(Pause(true));
            commands.set_state(Menu::Journal);
        }
        Menu::Journal => commands.set_state(Menu::None),
        _ => {}
    }
}
