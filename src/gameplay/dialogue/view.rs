use bevy::{platform::time::Instant, prelude::*};
use bevy_yarnspinner::{
    events::{DialogueCompleted, DialogueStarted, PresentLine, PresentOptions},
    prelude::{DialogueOption, DialogueRunner, LocalizedLine, OptionId},
};
use unicode_segmentation::UnicodeSegmentation;

use super::{SpeakerChangeEvent, YarnSpinnerDialogueViewSystemSet};
use crate::{
    GameplayScene,
    gameplay::{BALANCED, Health, InDialogue, herbs::Element},
    ui::{
        font,
        interaction::InteractionPalette,
        palette::{
            BUTTON_HOVERED_TINT, BUTTON_PRESSED_TINT, BUTTON_TEXT, GAIN_TEXT, HEADER_TEXT,
            LABEL_TEXT, LOSS_TEXT,
        },
        scale::texels,
        widget,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_dialogue_ui)
        .add_observer(despawn_dialogue_ui)
        .add_observer(present_line)
        .add_observer(present_options)
        .add_observer(select_option_clicked)
        .add_systems(
            Update,
            (
                write_text.run_if(resource_exists::<Typewriter>),
                continue_dialogue.run_if(resource_exists::<Typewriter>),
                select_option_by_key.run_if(resource_exists::<Typewriter>),
                update_health_bars.run_if(resource_exists::<Typewriter>),
                bob_continue.run_if(resource_exists::<Typewriter>),
            )
                .in_set(YarnSpinnerDialogueViewSystemSet),
        );
}

#[derive(Component, Clone, Default)]
struct DialogueRoot;

#[derive(Component, Clone, Default)]
struct DialogueNameNode;

#[derive(Component, Clone, Default)]
struct DialogueTextNode;

#[derive(Component, Clone, Default)]
struct DialogueOptionsNode;

#[derive(Component, Clone, Default)]
struct DialogueContinueNode;

#[derive(Component, Clone, Copy, Default)]
struct DialogueHealthFill(Element);

#[derive(Component, Clone)]
struct OptionButton(OptionId);

impl Default for OptionButton {
    fn default() -> Self {
        Self(OptionId(0))
    }
}

/// Options currently on screen, used to map number keys to [`OptionId`]s.
#[derive(Resource, Default)]
pub(super) struct PresentedOptions(Vec<DialogueOption>);

#[derive(Resource)]
pub(super) struct Typewriter {
    current_text: String,
    graphemes_left: Vec<String>,
    elapsed: f32,
    start: Instant,
    fast_typing: bool,
    character_name: Option<String>,
    last_before_options: bool,
}

impl Default for Typewriter {
    fn default() -> Self {
        Self {
            current_text: String::new(),
            graphemes_left: Vec::new(),
            elapsed: 0.0,
            start: Instant::now(),
            fast_typing: false,
            character_name: None,
            last_before_options: false,
        }
    }
}

impl Typewriter {
    fn set_line(&mut self, line: &LocalizedLine) {
        *self = Self {
            current_text: String::new(),
            graphemes_left: line
                .text_without_character_name()
                .graphemes(true)
                .map(|grapheme| grapheme.to_string())
                .collect(),
            character_name: line.character_name().map(|name| name.to_string()),
            last_before_options: line.is_last_line_before_options(),
            ..default()
        };
    }

    fn is_finished(&self) -> bool {
        self.graphemes_left.is_empty() && !self.current_text.is_empty()
    }

    /// Reveal the rest of the line much faster on subsequent updates.
    fn fast_forward(&mut self) {
        self.fast_typing = true;
    }

    fn update_current_text(&mut self) {
        if self.is_finished() {
            return;
        }
        self.elapsed += self.start.elapsed().as_secs_f32();
        self.start = Instant::now();
        let graphemes_per_second = if self.fast_typing { 120.0 } else { 40.0 };
        let take = (graphemes_per_second * self.elapsed).floor() as usize;
        let take = take.min(self.graphemes_left.len());
        self.elapsed -= take as f32 / graphemes_per_second;
        self.current_text.extend(self.graphemes_left.drain(..take));
    }
}

fn spawn_dialogue_ui(
    _: On<DialogueStarted>,
    mut commands: Commands,
    npc: Query<&Health, With<InDialogue>>,
) {
    commands.init_resource::<Typewriter>();
    let health = npc.iter().next().copied().unwrap_or_default();
    commands.spawn_scene(bsn! {
        #DialogueRoot
        DialogueRoot
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            row_gap: texels(2),
            padding: UiRect::top(texels(14)),
        }
        GlobalZIndex(0)
        Pickable::IGNORE
        DespawnOnExit::<GameplayScene>(GameplayScene::Base)
        Children [
            #DialogueTop
            Node {
                width: percent(90),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                margin: UiRect {
                    top: texels(2),
                    bottom: texels(6),
                },
            }
            Pickable::IGNORE
            Children [
                #DialogueOptions
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: texels(2),
                }
                DialogueOptionsNode
                Pickable::IGNORE
                --
                #DialogueHealth
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    width: percent(30),
                    row_gap: texels(1),
                    padding: UiRect::all(texels(4)),
                }
                @widget::panel()
                Pickable::IGNORE
                Children [
                    #DialogueHealthTitle
                    Text("Health")
                    @font::text(1)
                    TextColor(HEADER_TEXT)
                    Node {
                        margin: UiRect {
                            bottom: texels(2),
                        },
                    }
                    Pickable::IGNORE
                    --
                    { health_rows(health) }
                ]
            ]
            --
            #DialogueBox
            Node {
                width: percent(90),
                height: percent(25),
                padding: UiRect::new(texels(8), texels(8), texels(4), texels(4)),
            }
            @widget::panel()
            Pickable::IGNORE
            Children [
                #DialogueText
                Text({ String::new() })
                @font::text(1)
                TextColor(LABEL_TEXT)
                Node {
                    max_width: percent(100),
                }
                DialogueTextNode
                Pickable::IGNORE
                --
                #DialogueName
                Text({ String::new() })
                @font::text(1)
                TextColor(HEADER_TEXT)
                Node {
                    position_type: PositionType::Absolute,
                    right: texels(8),
                    bottom: texels(4),
                }
                DialogueNameNode
                Pickable::IGNORE
            ]
            --
            #DialogueContinue
            Text({ "▼".to_string() })
            @font::text(1)
            TextColor(LABEL_TEXT)
            DialogueContinueNode
            Visibility::Hidden
            Pickable::IGNORE
        ]
    });
}

fn despawn_dialogue_ui(
    _: On<DialogueCompleted>,
    mut commands: Commands,
    root: Single<Entity, With<DialogueRoot>>,
) {
    commands.entity(*root).despawn();
    commands.remove_resource::<PresentedOptions>();
    commands.remove_resource::<Typewriter>();
}

fn present_line(
    event: On<PresentLine>,
    mut name: Single<&mut Text, (With<DialogueNameNode>, Without<DialogueTextNode>)>,
    mut indicator: Single<&mut Visibility, With<DialogueContinueNode>>,
    mut typewriter: ResMut<Typewriter>,
    mut speaker_change: MessageWriter<SpeakerChangeEvent>,
) {
    name.0 = event.line.character_name().unwrap_or_default().to_string();
    if let Some(character_name) = event.line.character_name() {
        speaker_change.write(SpeakerChangeEvent {
            character_name: character_name.to_string(),
            speaking: true,
        });
    }
    typewriter.set_line(&event.line);
    **indicator = Visibility::Hidden;
}

fn write_text(
    mut typewriter: ResMut<Typewriter>,
    mut line: Single<&mut Text, With<DialogueTextNode>>,
    mut indicator: Single<&mut Visibility, With<DialogueContinueNode>>,
    mut was_finished: Local<bool>,
    mut speaker_change: MessageWriter<SpeakerChangeEvent>,
) {
    typewriter.update_current_text();
    line.0 = typewriter.current_text.clone();
    let finished = typewriter.is_finished();
    if finished && !*was_finished {
        if !typewriter.last_before_options {
            **indicator = Visibility::Inherited;
        }
        if let Some(character_name) = typewriter.character_name.as_deref() {
            speaker_change.write(SpeakerChangeEvent {
                character_name: character_name.to_string(),
                speaking: false,
            });
        }
    }
    *was_finished = finished;
}

fn bob_continue(
    time: Res<Time>,
    visibility: Single<&Visibility, With<DialogueContinueNode>>,
    mut style: Single<&mut Node, With<DialogueContinueNode>>,
) {
    if *visibility == Visibility::Hidden {
        return;
    }
    let pixels = (time.elapsed_secs() * 3.0).sin().powi(2) * 5.0;
    style.bottom = Val::Px(pixels);
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
) {
    let advance = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left)
        || touches.any_just_pressed();
    if advance && !typewriter.is_finished() {
        typewriter.fast_forward();
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
        }
    }
}

fn select_option_by_key(
    keys: Res<ButtonInput<KeyCode>>,
    presented: Option<Res<PresentedOptions>>,
    mut runners: Query<&mut DialogueRunner>,
    mut commands: Commands,
    options_node: Option<Single<Entity, With<DialogueOptionsNode>>>,
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
    if let Ok(mut runner) = runners.single_mut() {
        let _ = runner.select_option(id);
    }
    clear_options(&mut commands, *options_node);
}

fn clear_options(commands: &mut Commands, options_node: Entity) {
    commands.entity(options_node).despawn_related::<Children>();
    commands.remove_resource::<PresentedOptions>();
}

/// Health values span from [`MIN_HEALTH`] to [`MAX_HEALTH`], with
/// [`BALANCED`] as the healthy middle.
const MIN_HEALTH: u8 = 1;
const MAX_HEALTH: u8 = 10;

fn health_rows(health: Health) -> Vec<Box<dyn Scene>> {
    [
        Element::Wood,
        Element::Fire,
        Element::Earth,
        Element::Metal,
        Element::Water,
    ]
    .into_iter()
    .map(|element| -> Box<dyn Scene> {
        let value = health.get(element);
        let width = value.clamp(MIN_HEALTH, MAX_HEALTH) as f32 / MAX_HEALTH as f32 * 100.0;
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

fn update_health_bars(
    npc: Query<&Health, (With<InDialogue>, Changed<Health>)>,
    mut fills: Query<(&DialogueHealthFill, &mut Node, &mut BackgroundColor)>,
) {
    let Some(health) = npc.iter().next().copied() else {
        return;
    };
    for (fill, mut node, mut background) in &mut fills {
        let value = health.get(fill.0);
        let width = value.clamp(MIN_HEALTH, MAX_HEALTH) as f32 / MAX_HEALTH as f32 * 100.0;
        node.width = percent(width);
        *background = BackgroundColor(health_bar_color(value));
    }
}

/// Green at the balanced value (5), fading to red at either extreme (1 or 10).
fn health_bar_color(value: u8) -> Color {
    let value = value.clamp(MIN_HEALTH, MAX_HEALTH);
    let t = if value >= BALANCED {
        f32::from(value - BALANCED) / f32::from(MAX_HEALTH - BALANCED)
    } else {
        f32::from(BALANCED - value) / f32::from(BALANCED - MIN_HEALTH)
    };
    GAIN_TEXT.mix(&LOSS_TEXT, t)
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
