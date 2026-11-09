use bevy::prelude::*;

use crate::{
    GameplayScene,
    gameplay::{InLineup, Met, Returning, ShopClosed, ShopOpened, Visitor},
    loading::NPCAssets,
    ui::{
        font,
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
            (sync_arrivals, show_notice, shake_arrivals).run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_observer(announce_opening)
        .add_observer(announce_closing);
}

/// Root of the row of arrival icons shown while outdoors.
#[derive(Component, Clone, Default)]
struct ArrivalList;

/// Marks a single arrival icon and remembers which visitor it represents.
#[derive(Component)]
struct ArrivalIcon(Entity);

const NOTICE_SECONDS: f32 = 3.0;
const FADE_SECONDS: f32 = 1.0;

#[derive(Component, Clone, Default)]
struct Notice {
    left: f32,
}

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
        Children [
            Notice
            Text("")
            @font::text(1)
            TextColor(Color::WHITE)
            TextShadow {
                offset: Vec2::splat(TEXEL),
                color: Color::BLACK,
            }
            Node {
                display: Display::None,
            }
            Pickable::IGNORE
        ]
    }
}

fn sync_arrivals(
    mut commands: Commands,
    list: Single<Entity, With<ArrivalList>>,
    waiting: Query<(Entity, &Visitor, Ref<Returning>), (With<InLineup>, Without<Met>)>,
    icons: Query<(Entity, &ArrivalIcon)>,
    assets: Res<NPCAssets>,
) {
    for (icon, shown) in &icons {
        if !waiting.contains(shown.0) {
            commands.entity(icon).despawn();
        }
    }
    for (visitor, who, new) in &waiting {
        if icons.iter().any(|(_, shown)| shown.0 == visitor) {
            continue;
        }
        let (image, frame) = assets.sprite(who.name());
        let rect = centered_crop(frame, TAB_SIZE, TAB_SIZE, portrait_top(who.name()));
        let mut icon = commands.spawn_scene(arrival_icon(image.clone(), rect));
        icon.insert((ChildOf(*list), ArrivalIcon(visitor)));
        if new.is_added() {
            icon.insert(Shake { elapsed: 0.0 });
        }
    }
}

fn announce(notices: &mut Query<(&mut Notice, &mut Text)>, message: String) {
    for (mut notice, mut text) in notices {
        notice.left = NOTICE_SECONDS + FADE_SECONDS;
        text.0 = message.clone();
    }
}

fn announce_opening(_: On<ShopOpened>, mut notices: Query<(&mut Notice, &mut Text)>) {
    announce(&mut notices, "Your shop is open for the day.".into());
}

fn announce_closing(_: On<ShopClosed>, mut notices: Query<(&mut Notice, &mut Text)>) {
    announce(&mut notices, "Your shop has closed for the day.".into());
}

fn show_notice(
    time: Res<Time>,
    notice: Single<(&mut Notice, &mut Node, &mut TextColor, &mut TextShadow)>,
) {
    let (mut notice, mut node, mut color, mut shadow) = notice.into_inner();
    notice.left = (notice.left - time.delta_secs()).max(0.0);
    let display = if notice.left > 0.0 {
        Display::Flex
    } else {
        Display::None
    };
    if node.display != display {
        node.display = display;
    }
    let alpha = (notice.left / FADE_SECONDS).min(1.0);
    if color.0.alpha() != alpha {
        color.0.set_alpha(alpha);
        shadow.color.set_alpha(alpha);
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
