use bevy::prelude::*;

use crate::{
    GameplayScene,
    gameplay::{InLineup, Visitor},
    loading::NPCAssets,
    ui::{
        menus::journal::{TAB_PANEL_SIZE, TAB_SIZE, centered_crop, portrait_top, tab_icon},
        scale::{TEXEL, texels},
        widget,
    },
};

const SHAKE_SECONDS: f32 = 0.6;
const SHAKE_HZ: f32 = 12.0;
const SHAKE_RADIANS: f32 = 0.15;
const SHAKE_TEXELS: f32 = 1.5;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayScene::Outdoors), spawn_arrival_list)
        .add_systems(
            Update,
            shake_arrivals.run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_observer(arrive)
        .add_observer(depart);
}

/// Root of the row of arrival icons shown while outdoors.
#[derive(Component, Clone, Default)]
struct ArrivalList;

/// Marks a single arrival icon and remembers which visitor it represents.
#[derive(Component)]
struct ArrivalIcon(Entity);

/// Drives the one-shot attention shake.
#[derive(Component)]
struct Shake {
    elapsed: f32,
}

fn spawn_arrival_list(mut commands: Commands) {
    commands.spawn_scene(arrival_list());
}

fn arrival_list() -> impl Scene {
    bsn! {
        Name::new("Arrival List")
        ArrivalList
        Node {
            position_type: PositionType::Absolute,
            left: texels(4),
            bottom: texels(4),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            column_gap: texels(4),
        }
        Pickable::IGNORE
        DespawnOnExit::<GameplayScene>(GameplayScene::Outdoors)
    }
}

fn arrive(
    added: On<Add<InLineup>>,
    mut commands: Commands,
    scene: Res<State<GameplayScene>>,
    lists: Query<Entity, With<ArrivalList>>,
    visitors: Query<&Visitor>,
    assets: Res<NPCAssets>,
) {
    if *scene.get() != GameplayScene::Outdoors {
        return;
    }
    let Some(list) = lists.iter().next() else {
        return;
    };
    let Ok(visitor) = visitors.get(added.entity) else {
        return;
    };

    let (image, frame) = assets.sprite(visitor.name());
    let image = image.clone();
    let rect = centered_crop(frame, TAB_SIZE, TAB_SIZE, portrait_top(visitor.name()));
    commands.spawn_scene(arrival_icon(image, rect)).insert((
        ChildOf(list),
        ArrivalIcon(added.entity),
        Shake { elapsed: 0.0 },
    ));
}

fn depart(
    removed: On<Remove<InLineup>>,
    mut commands: Commands,
    icons: Query<(Entity, &ArrivalIcon)>,
) {
    for (icon, marker) in &icons {
        if marker.0 == removed.entity {
            commands.entity(icon).despawn();
        }
    }
}

fn arrival_icon(image: Handle<Image>, rect: Rect) -> impl Scene {
    bsn! {
        Name::new("Arrival")
        Node {
            width: texels(TAB_PANEL_SIZE),
            height: texels(TAB_PANEL_SIZE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        @widget::panel()
        Pickable::IGNORE
        Children [
            @tab_icon(image, rect)
        ]
    }
}

fn shake_arrivals(time: Res<Time>, mut icons: Query<(&mut Shake, &mut UiTransform)>) {
    for (mut shake, mut transform) in &mut icons {
        shake.elapsed += time.delta_secs();
        let progress = (shake.elapsed / SHAKE_SECONDS).min(1.0);
        // Decay to nothing so the icon settles in place after one shake.
        let amplitude = 1.0 - progress;
        let wobble = shake.elapsed * SHAKE_HZ * std::f32::consts::TAU;
        transform.translation = Val2::px(wobble.sin() * amplitude * SHAKE_TEXELS * TEXEL, 0.0);
        transform.rotation = Rot2::radians((wobble * 0.8).cos() * amplitude * SHAKE_RADIANS);
    }
}
