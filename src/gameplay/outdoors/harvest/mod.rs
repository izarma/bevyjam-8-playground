use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use pixel_art::PixelSprite;
use plants::Growing;

use super::player::Harvest;
use crate::{
    GameplayScene, Pause,
    gameplay::{inventory::Inventory, player::Player},
    ui::menus::Menu,
};

mod wipe;
mod yields;

const REACH: f32 = 1.5;
const FADE_SECONDS: f32 = 0.3;
const OVERLAY_ALPHA: f32 = 0.6;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((wipe::plugin, yields::plugin))
        .add_systems(
            Update,
            find_in_reach.run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_observer(start_harvest)
        .add_observer(collect)
        .add_systems(OnExit(Menu::Minigame), close_overlays)
        .add_systems(Update, fade_overlay);
}

// the nearest plant you can harvest
#[derive(Component)]
struct InReach;

#[derive(EntityEvent)]
struct Won {
    entity: Entity,
}

#[derive(Component)]
struct Fades(Color);

#[derive(Component)]
struct Overlay {
    plant: Entity,
    shown: f32,
    closing: bool,
}

fn find_in_reach(
    mut commands: Commands,
    player: Option<Single<&GlobalTransform, With<Player>>>,
    plants: Query<(Entity, &GlobalTransform, Has<InReach>), With<Growing>>,
) {
    let ground = |at: &GlobalTransform| Vec2::new(at.translation().x, at.translation().z);
    let nearest = player.and_then(|player| {
        let player = ground(&player);
        plants
            .iter()
            .map(|(plant, at, _)| (plant, ground(at).distance(player)))
            .filter(|(_, distance)| *distance <= REACH)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(plant, _)| plant)
    });
    for (plant, _, in_reach) in &plants {
        if in_reach && Some(plant) != nearest {
            commands.entity(plant).remove::<InReach>();
        } else if !in_reach && Some(plant) == nearest {
            commands.entity(plant).insert(InReach);
        }
    }
}

fn start_harvest(
    _: On<Start<Harvest>>,
    mut commands: Commands,
    plants: Query<(Entity, &PixelSprite), With<InReach>>,
    images: Res<Assets<Image>>,
    mut next_pause: ResMut<NextState<Pause>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    let Some((plant, sprite)) = plants.iter().next() else {
        return;
    };

    let overlay = commands
        .spawn((
            Name::new("Harvest Overlay"),
            Overlay {
                plant,
                shown: 0.0,
                closing: false,
            },
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(2),
            DespawnOnExit(GameplayScene::Outdoors),
        ))
        .id();

    if !wipe::spawn(&mut commands, overlay, sprite, &images) {
        commands.entity(overlay).despawn();
        return;
    }
    next_pause.set(Pause(true));
    next_menu.set(Menu::Minigame);
}

fn collect(
    won: On<Won>,
    mut commands: Commands,
    overlays: Query<&Overlay>,
    plants: Query<&Growing>,
    mut inventory: Single<&mut Inventory>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    next_menu.set(Menu::None);
    let Ok(overlay) = overlays.get(won.entity) else {
        return;
    };
    let Ok(Growing(plant)) = plants.get(overlay.plant) else {
        return;
    };
    
    commands.entity(overlay.plant).despawn();
    let count = inventory.plants.entry(*plant).or_default();
    *count += 1;
    info!("harvested {plant:?}, carrying {count}");
}

fn close_overlays(mut overlays: Query<&mut Overlay>) {
    for mut overlay in &mut overlays {
        overlay.closing = true;
    }
}

fn fade_overlay(
    mut commands: Commands,
    time: Res<Time>,
    mut overlays: Query<(Entity, &mut Overlay, &mut BackgroundColor), Without<Fades>>,
    mut faders: Query<(&Fades, &mut BackgroundColor)>,
) {
    let step = time.delta_secs() / FADE_SECONDS;
    for (entity, mut overlay, mut background) in &mut overlays {
        overlay.shown = if overlay.closing {
            overlay.shown - step
        } else {
            (overlay.shown + step).min(1.0)
        };
        if overlay.shown <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        background.0 = Color::BLACK.with_alpha(OVERLAY_ALPHA * overlay.shown);
        for (fades, mut background) in &mut faders {
            background.0 = fades.0.with_alpha(fades.0.alpha() * overlay.shown);
        }
    }
}
