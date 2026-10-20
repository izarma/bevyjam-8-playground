use bevy::prelude::*;
use bevy_kira_audio::prelude::{Audio, AudioControl};

use crate::engine::loading::MenuAssets;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(apply_interaction_palette_on_click);
    app.add_observer(apply_interaction_palette_on_over);
    app.add_observer(apply_interaction_palette_on_out);
    app.add_observer(apply_interaction_palette_on_release);

    app.add_observer(play_sound_effect_on_click);
    app.add_observer(play_sound_effect_on_over);
}

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn apply_interaction_palette_on_click(
    click: On<PointerClick>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(click.event_target()) else {
        return;
    };

    *bg = palette.pressed.into();
}

fn apply_interaction_palette_on_release(
    click: On<PointerRelease>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(click.event_target()) else {
        return;
    };

    *bg = palette.hovered.into();
}

fn apply_interaction_palette_on_over(
    over: On<PointerOver>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(over.event_target()) else {
        return;
    };

    *bg = palette.hovered.into();
}

fn apply_interaction_palette_on_out(
    out: On<PointerOut>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(out.event_target()) else {
        return;
    };

    *bg = palette.none.into();
}

fn play_sound_effect_on_click(
    on: On<PointerClick>,
    ui_asset: If<Res<MenuAssets>>,
    audio: Res<Audio>,
    interaction_entities: Query<Entity, With<InteractionPalette>>,
) {
    if interaction_entities.contains(on.event_target()) {
        audio.play(ui_asset.button_click.clone());
    }
}

fn play_sound_effect_on_over(
    on: On<PointerOver>,
    ui_asset: If<Res<MenuAssets>>,
    audio: Res<Audio>,
    interaction_entities: Query<Entity, With<InteractionPalette>>,
) {
    if interaction_entities.contains(on.event_target()) {
        audio.play(ui_asset.button_hover.clone());
    }
}
