use bevy::prelude::*;
use bevy_kira_audio::prelude::{Audio, AudioControl};
use bevy_yarnspinner::{
    events::PresentOptions,
    prelude::{DialogueOption, DialogueRunner, OptionId},
};

use super::{
    DialogueContinueNode, DialogueOptionsNode, PresentedOptions, Typewriter,
    YarnSpinnerDialogueViewSystemSet,
};
use crate::{
    loading::{NPCAssets, UIAssets},
    ui::{
        font,
        interaction::InteractionPalette,
        palette::{BUTTON_HOVERED_TINT, BUTTON_PRESSED_TINT, BUTTON_TEXT},
        scale::texels,
        widget,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(present_options)
        .add_observer(select_option_clicked)
        .add_systems(
            Update,
            (
                continue_dialogue.run_if(resource_exists::<Typewriter>),
                select_option_by_key.run_if(resource_exists::<Typewriter>),
            )
                .in_set(YarnSpinnerDialogueViewSystemSet),
        );
}

#[derive(Component, Clone)]
struct OptionButton(OptionId);

impl Default for OptionButton {
    fn default() -> Self {
        Self(OptionId(0))
    }
}

fn present_options(
    event: On<PresentOptions>,
    mut commands: Commands,
    options_node: Single<Entity, With<DialogueOptionsNode>>,
    mut indicator: Single<&mut Visibility, With<DialogueContinueNode>>,
) {
    let options_node = *options_node;
    clear_options(&mut commands, options_node);

    let options: Vec<DialogueOption> = event
        .options
        .iter()
        .filter(|option| option.is_available)
        .cloned()
        .collect();
    commands.insert_resource(PresentedOptions(options.clone()));

    for (index, option) in options.iter().enumerate() {
        let text = option.line.text_without_character_name();
        let id = option.id;
        commands.spawn_scene(bsn! {
            #DialogueOptionButton
            ChildOf(options_node)
            OptionButton(id)
            Node {
                padding: UiRect::new(texels(6), texels(6), texels(2), texels(2)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
            @widget::panel()
            InteractionPalette {
                none: Color::WHITE,
                hovered: BUTTON_HOVERED_TINT,
                pressed: BUTTON_PRESSED_TINT,
            }
            @on(select_option_clicked)
            Children [
                #DialogueOptionText
                Text({ format!("{}. {}", index + 1, text) })
                @font::text(1)
                TextColor(BUTTON_TEXT)
                Pickable::IGNORE
            ]
        });
    }

    **indicator = Visibility::Hidden;
}

fn select_option_clicked(
    click: On<PointerClick>,
    buttons: Query<&OptionButton>,
    mut runners: Query<&mut DialogueRunner>,
    mut commands: Commands,
    options_node: Single<Entity, With<DialogueOptionsNode>>,
) {
    let Ok(button) = buttons.get(click.entity) else {
        return;
    };
    let id = button.0;
    if let Ok(mut runner) = runners.single_mut() {
        let _ = runner.select_option(id);
    }
    clear_options(&mut commands, *options_node);
}

fn continue_dialogue(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut runners: Query<&mut DialogueRunner>,
    mut typewriter: ResMut<Typewriter>,
    presented_options: Option<Res<PresentedOptions>>,
    assets: Res<NPCAssets>,
    audio: Res<Audio>,
) {
    let advance = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left)
        || touches.any_just_pressed();
    if advance && !typewriter.is_finished() {
        typewriter.fast_forward();
        if keys.just_pressed(KeyCode::Space) {
            audio
                .play(assets.dialogue_continue.clone())
                .with_volume(3.0);
        }
        return;
    }
    let auto_advance =
        typewriter.last_before_options && typewriter.is_finished() && presented_options.is_none();
    if !advance && !auto_advance {
        return;
    }
    for mut runner in &mut runners {
        if runner.is_running() && !runner.is_waiting_for_option_selection() {
            runner.continue_in_next_update();
            if keys.just_pressed(KeyCode::Space) {
                audio
                    .play(assets.dialogue_continue.clone())
                    .with_volume(3.0);
            }
        }
    }
}

fn select_option_by_key(
    keys: Res<ButtonInput<KeyCode>>,
    presented: Option<Res<PresentedOptions>>,
    mut runners: Query<&mut DialogueRunner>,
    mut commands: Commands,
    options_node: Option<Single<Entity, With<DialogueOptionsNode>>>,
    assets: Res<UIAssets>,
    audio: Res<Audio>,
) {
    let Some(options_node) = options_node else {
        return;
    };
    let Some(presented) = presented else {
        return;
    };
    let Some(index) = number_key_pressed(&keys) else {
        return;
    };
    let Some(option) = presented.0.get(index) else {
        return;
    };
    let id = option.id;
    if let Ok(mut runner) = runners.single_mut()
        && runner.select_option(id).is_ok()
    {
        audio.play(assets.button_click.clone()).with_volume(3.0);
    }
    clear_options(&mut commands, *options_node);
}

fn clear_options(commands: &mut Commands, options_node: Entity) {
    commands.entity(options_node).despawn_related::<Children>();
    commands.remove_resource::<PresentedOptions>();
}

fn number_key_pressed(keys: &ButtonInput<KeyCode>) -> Option<usize> {
    const NUMBER_KEYS: [KeyCode; 9] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    const NUMPAD_KEYS: [KeyCode; 9] = [
        KeyCode::Numpad1,
        KeyCode::Numpad2,
        KeyCode::Numpad3,
        KeyCode::Numpad4,
        KeyCode::Numpad5,
        KeyCode::Numpad6,
        KeyCode::Numpad7,
        KeyCode::Numpad8,
        KeyCode::Numpad9,
    ];
    NUMBER_KEYS
        .iter()
        .chain(NUMPAD_KEYS.iter())
        .position(|key| keys.just_pressed(*key))
        .map(|index| index % 9)
}
