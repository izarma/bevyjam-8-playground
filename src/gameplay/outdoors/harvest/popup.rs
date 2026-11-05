// popup text for inventory acquisitions
use bevy::prelude::*;
use plants::Plant;

use crate::{
    GameplayScene, PausableSystems,
    gameplay::herbs,
    ui::{billboard::Billboards, font, scale::TEXEL},
};

const CANVAS: UVec2 = UVec2::new(128, 16);
const RISE_SPEED: f32 = 0.6;
const SECONDS: f32 = 1.2;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        rise.in_set(PausableSystems)
            .run_if(in_state(GameplayScene::Outdoors)),
    );
}

#[derive(Component)]
struct Popup(Timer);

pub(super) fn spawn(billboards: &mut Billboards, at: Vec3, plant: Plant) {
    let text = format!("+1 {}", herbs::name(plant));
    billboards
        .spawn(
            CANVAS,
            at,
            bsn! {
                #PopupCanvas
                Node {
                    width: percent(100),
                    height: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexEnd,
                }
                Pickable::IGNORE
                Children [
                    Text(text)
                    @font::text(1)
                    TextColor(Color::WHITE)
                TextShadow {
                    offset: Vec2::splat(TEXEL),
                    color: Color::BLACK,
                }
                    Pickable::IGNORE
                ]
            },
            GameplayScene::Outdoors,
        )
        .insert((
            Name::new("Popup"),
            Popup(Timer::from_seconds(SECONDS, TimerMode::Once)),
        ));
}

fn rise(
    mut commands: Commands,
    time: Res<Time>,
    mut popups: Query<(Entity, &mut Popup, &mut Transform)>,
) {
    for (entity, mut popup, mut transform) in &mut popups {
        if popup.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.y += RISE_SPEED * time.delta_secs();
    }
}
