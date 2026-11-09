use bevy::prelude::*;

use super::{
    Concoction, DialogueTarget, ShownHealth, Typewriter, YarnSpinnerDialogueViewSystemSet,
};
use crate::{
    gameplay::{
        Health,
        clock::Clock,
        herbs::{self, Element},
        visitors::{Ailing, Ailment},
    },
    ui::{
        font,
        health::{ailment_countdown, ailment_mark, bar_width, health_bar_color},
        palette::{HEADER_TEXT, LABEL_TEXT, LOSS_TEXT},
        scale::{TEXEL, texels},
        widget,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_health_bars.run_if(resource_exists::<Typewriter>),
            float_health_changes.run_if(resource_exists::<Typewriter>),
            show_ailment.run_if(resource_exists::<Typewriter>),
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

#[derive(Component, Clone, Default)]
struct AilmentTitle;

#[derive(Component, Clone, Default)]
struct AilmentCountdown;

#[derive(Component, Clone, Default)]
struct AilmentMark(Element);

pub(super) fn health_panel(health: Health, ailment: Option<(Ailment, f32)>) -> impl Scene {
    let title = ailment.map_or("Health".to_string(), |(ailment, _)| ailment.name());
    let hours = ailment
        .map(|(_, hours)| ailment_countdown(hours))
        .unwrap_or_default();
    bsn! {
        #DialogueHealth
        ShownHealth(health)
        // sized by what's in it, so the title stays on one line
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: texels(2),
        }
        Pickable::IGNORE
        Children [
            Node {
                justify_content: JustifyContent::SpaceBetween,
                column_gap: texels(6),
                padding: UiRect::all(texels(4)),
            }
            @widget::panel()
            Pickable::IGNORE
            Children [
                #DialogueHealthTitle
                AilmentTitle
                Text({ title })
                @font::text(1)
                TextColor(HEADER_TEXT)
                Pickable::IGNORE
                --
                AilmentCountdown
                Text({ hours })
                @font::text(1)
                TextColor(LABEL_TEXT)
                Pickable::IGNORE
            ]
            --
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: texels(1),
                padding: UiRect::all(texels(4)),
            }
            @widget::panel()
            Pickable::IGNORE
            Children [ { health_rows(health, ailment.map(|(ailment, _)| ailment)) } ]
        ]
    }
}

fn show_ailment(
    target: Res<DialogueTarget>,
    visitors: Query<&Ailing>,
    clock: Single<&Clock>,
    mut texts: Query<
        (
            &mut Text,
            Has<AilmentTitle>,
            Has<AilmentCountdown>,
            Option<&AilmentMark>,
        ),
        Or<(
            With<AilmentTitle>,
            With<AilmentCountdown>,
            With<AilmentMark>,
        )>,
    >,
) {
    let ailing = target.0.and_then(|visitor| visitors.get(visitor).ok());
    for (mut text, title, counting, marked) in &mut texts {
        let shown = match (ailing, title, counting, marked) {
            (Some(ailing), true, ..) => ailing.ailment.name(),
            (None, true, ..) => "Health".to_string(),
            (Some(ailing), _, true, _) => ailment_countdown(ailing.next - clock.hours),
            (Some(ailing), .., Some(&AilmentMark(element))) => {
                ailment_mark(ailing.ailment, element)
            }
            _ => String::new(),
        };
        if text.0 != shown {
            text.0 = shown;
        }
    }
}

fn health_rows(health: Health, ailment: Option<Ailment>) -> Vec<Box<dyn Scene>> {
    Element::ALL
        .into_iter()
        .map(|element| -> Box<dyn Scene> {
            let value = health.get(element);
            let width = bar_width(value);
            let next = ailment
                .map(|ailment| ailment_mark(ailment, element))
                .unwrap_or_default();
            Box::new(bsn! {
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: texels(3),
                }
                Pickable::IGNORE
                Children [
                    { vec![herbs::element_icon(element)] }
                    --
                    Node {
                        width: texels(64),
                        height: texels(5),
                        flex_shrink: 0.0,
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
                    --
                    AilmentMark(element)
                    Text({ next })
                    @font::text(1)
                    TextColor(LOSS_TEXT)
                    Node {
                        width: texels(16),
                        flex_shrink: 0.0,
                    }
                    Pickable::IGNORE
                ]
            })
        })
        .collect()
}

// shows the visitor health optionally with the currently selected concoction
fn update_health_bars(
    target: Res<DialogueTarget>,
    visitors: Query<&Health>,
    concoction: Option<Single<&Concoction>>,
    mut fills: Query<(&DialogueHealthFill, &mut Node, &mut BackgroundColor)>,
) {
    let Some(&health) = target.0.and_then(|visitor| visitors.get(visitor).ok()) else {
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
    target: Res<DialogueTarget>,
    visitors: Query<&Health>,
    mut shown: Single<&mut ShownHealth>,
    fills: Query<(&DialogueHealthFill, &ChildOf)>,
) {
    let Some(&health) = target.0.and_then(|visitor| visitors.get(visitor).ok()) else {
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
