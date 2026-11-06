use bevy::prelude::*;

use super::{Concoction, ShownHealth, Typewriter, YarnSpinnerDialogueViewSystemSet};
use crate::{
    gameplay::{BALANCED, BodyOf, Health, InDialogue, MAX_HEALTH, MIN_HEALTH, herbs::Element},
    ui::{
        font,
        palette::{GAIN_TEXT, LABEL_TEXT, LOSS_TEXT},
        scale::{TEXEL, texels},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_health_bars.run_if(resource_exists::<Typewriter>),
            float_health_changes.run_if(resource_exists::<Typewriter>),
            rise_health_changes,
        )
            .in_set(YarnSpinnerDialogueViewSystemSet),
    );
}

#[derive(Component, Clone, Copy, Default)]
struct DialogueHealthFill(Element);

// the scrolling text indicating the impact a herb had
#[derive(Component, Clone, Default)]
struct RisingChange {
    age: f32,
}

const RISE_SECONDS: f32 = 1.0;
const RISE_TEXELS: f32 = 14.0;

pub(super) fn health_rows(health: Health) -> Vec<Box<dyn Scene>> {
    Element::ALL
        .into_iter()
        .map(|element| -> Box<dyn Scene> {
            let value = health.get(element);
            let width = bar_width(value);
            Box::new(bsn! {
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                }
                Pickable::IGNORE
                Children [
                    Text({ element.name().to_string() })
                    @font::text(1)
                    TextColor(LABEL_TEXT)
                    Pickable::IGNORE
                    --
                    Node {
                        width: texels(64),
                        height: texels(5),
                    }
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5))
                    Pickable::IGNORE
                    Children [
                        Node {
                            width: percent(width),
                            height: percent(100),
                        }
                        DialogueHealthFill(element)
                        BackgroundColor(health_bar_color(value))
                        Pickable::IGNORE
                    ]
                ]
            })
        })
        .collect()
}

// shows the visitor health optionally with the currently selected concoction
fn update_health_bars(
    npc: Query<&BodyOf, With<InDialogue>>,
    visitors: Query<&Health>,
    concoction: Option<Single<&Concoction>>,
    mut fills: Query<(&DialogueHealthFill, &mut Node, &mut BackgroundColor)>,
) {
    let Some(&health) = npc.iter().find_map(|body| visitors.get(body.0).ok()) else {
        return;
    };
    let health = concoction.map_or(health, |concoction| concoction.preview(health));
    for (fill, mut node, mut background) in &mut fills {
        let value = health.get(fill.0);
        let width = bar_width(value);
        node.width = percent(width);
        *background = BackgroundColor(health_bar_color(value));
    }
}

fn float_health_changes(
    mut commands: Commands,
    npc: Query<&BodyOf, With<InDialogue>>,
    visitors: Query<&Health>,
    mut shown: Single<&mut ShownHealth>,
    fills: Query<(&DialogueHealthFill, &ChildOf)>,
) {
    let Some(&health) = npc.iter().find_map(|body| visitors.get(body.0).ok()) else {
        return;
    };
    if shown.0 == health {
        return;
    }
    for (&DialogueHealthFill(element), bar) in &fills {
        let (now, before) = (health.get(element), shown.0.get(element));
        if now == before {
            continue;
        }
        let change = i16::from(now) - i16::from(before);
        let bar = bar.parent();
        commands.spawn_scene(bsn! {
            #RisingChange
            ChildOf(bar)
            RisingChange
            Text({ format!("{change:+}") })
            @font::text(1)
            TextColor(health_bar_color(now))
            Node {
                position_type: PositionType::Absolute,
                left: percent(45),
                top: px(0),
            }
            Pickable::IGNORE
        });
    }
    shown.0 = health;
}

fn rise_health_changes(
    mut commands: Commands,
    time: Res<Time>,
    mut changes: Query<(Entity, &mut RisingChange, &mut Node, &mut TextColor)>,
) {
    for (entity, mut change, mut node, mut color) in &mut changes {
        change.age += time.delta_secs();
        let k = change.age / RISE_SECONDS;
        if k >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        node.top = px(-k * RISE_TEXELS * TEXEL);
        color.0.set_alpha(1.0 - k * k);
    }
}

fn bar_width(value: u8) -> f32 {
    f32::from(value - MIN_HEALTH) / f32::from(MAX_HEALTH - MIN_HEALTH) * 100.0
}

// green or red based on distance from balanced
fn health_bar_color(value: u8) -> Color {
    let value = value.clamp(MIN_HEALTH, MAX_HEALTH);
    let t = if value >= BALANCED {
        f32::from(value - BALANCED) / f32::from(MAX_HEALTH - BALANCED)
    } else {
        f32::from(BALANCED - value) / f32::from(BALANCED - MIN_HEALTH)
    };
    GAIN_TEXT.mix(&LOSS_TEXT, t)
}
