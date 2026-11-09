use bevy::prelude::*;

use crate::{
    GameState, Survivors,
    gameplay::VISITORS,
    loading::UIAssets,
    ui::{
        font,
        input_prompt::{KEY_ESCAPE, MOUSE_LEFT, prompt},
        menus::Menu,
        palette::{HEADER_TEXT, LABEL_TEXT},
        scale::texels,
    },
};

use super::{close_menu, credits::credits, open_game_over_menu};

const SCROLL_TEXELS_PER_SECOND: f32 = 14.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::GameOver), open_game_over_menu);
    app.add_systems(OnExit(GameState::GameOver), close_menu);
    app.add_systems(OnEnter(Menu::GameOver), spawn_ending);
    app.add_systems(Update, scroll_credits.run_if(in_state(Menu::GameOver)));
}

#[derive(Component, Clone, Default)]
struct CreditsHint;

#[derive(Component, Clone, Default)]
struct Scrolling {
    started: bool,
    elapsed: f32,
}

fn epitaph(saved: usize) -> &'static str {
    match saved {
        0 => {
            "Thus forever lost. Forever forgotten. Echoes unrecognisable within endless sea of screams."
        }
        VISITORS => "Thus all the troubled spirits find their way home.",
        _ => "Some spirits ascend. Others forever lost.",
    }
}

fn spawn_ending(
    mut commands: Commands,
    survivors: Option<Res<Survivors>>,
    textures: Res<UIAssets>,
) {
    let saved = survivors.map_or(0, |survivors| survivors.0.len());
    let words = epitaph(saved);
    let logo = textures.bevy.clone();
    commands.spawn_scene(bsn! {
        Name::new("Ending")
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            overflow: Overflow::clip(),
        }
        GlobalZIndex(2)
        Pickable::IGNORE
        DespawnOnExit<Menu>(Menu::GameOver)
        Children [
            Scrolling
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                top: texels(16),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: texels(6),
            }
            Pickable::IGNORE
            Children [
                Text({ words.to_string() })
                @font::text(1)
                TextColor(HEADER_TEXT)
                TextLayout {
                    justify: Justify::Center,
                }
                Node {
                    max_width: percent(70),
                    margin: UiRect::bottom(percent(56)),
                }
                Pickable::IGNORE
                --
                @credits(logo)
            ]
            --
            Node {
                position_type: PositionType::Absolute,
                right: texels(6),
                bottom: texels(6),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: texels(2),
            }
            Pickable::IGNORE
            Children [
                CreditsHint
                @hint(MOUSE_LEFT, "Credits")
                --
                @hint(KEY_ESCAPE, "Title")
            ]
        ]
    });
}

fn hint(key: u32, label: &'static str) -> impl Scene {
    bsn! {
        Node {
            align_items: AlignItems::Center,
            column_gap: texels(3),
        }
        Pickable::IGNORE
        Children [
            @prompt(key)
            --
            Text(label)
            @font::text(1)
            TextColor(LABEL_TEXT)
            Pickable::IGNORE
        ]
    }
}

fn scroll_credits(
    time: Res<Time>,
    mut hints: Query<&mut Node, With<CreditsHint>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut scrolling: Query<(&mut Scrolling, &mut UiTransform, &ComputedNode)>,
) {
    let pressed = mouse.just_pressed(MouseButton::Left)
        || keys.any_just_pressed([KeyCode::Space, KeyCode::Enter])
        || touches.any_just_pressed();
    for (mut scroll, mut transform, node) in &mut scrolling {
        if !scroll.started {
            scroll.started = pressed;
            for mut hint in &mut hints {
                if pressed {
                    hint.display = Display::None;
                }
            }
            continue;
        }
        scroll.elapsed += time.delta_secs();
        let height = node.size().y * node.inverse_scale_factor();
        let moved = (scroll.elapsed * SCROLL_TEXELS_PER_SECOND * crate::ui::scale::TEXEL)
            .min((height - 40.0 * crate::ui::scale::TEXEL).max(0.0));
        let translation = Val2::px(0.0, -moved);
        if transform.translation != translation {
            transform.translation = translation;
        }
    }
}
