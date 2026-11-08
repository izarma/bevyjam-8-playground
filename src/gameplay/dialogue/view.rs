use bevy::{platform::time::Instant, prelude::*};
use bevy_yarnspinner::{
    events::{DialogueCompleted, DialogueStarted, PresentLine, PresentOptions},
    prelude::{DialogueOption, DialogueRunner, LocalizedLine, OptionId},
};
use unicode_segmentation::UnicodeSegmentation;

use super::{SpeakerChangeEvent, YarnSpinnerDialogueViewSystemSet, concoction::Concoction};
use crate::{
    GameplayScene,
    gameplay::{
        BodyOf, Health, InDialogue,
        clock::Clock,
        herbs::{self, Element},
        visitors::{Ailing, Ailment},
    },
    ui::{
        font,
        health::{bar_width, health_bar_color},
        input_prompt::{KEY_ESCAPE, KEY_LEFT, prompt},
        interaction::InteractionPalette,
        palette::{
            BUTTON_HOVERED_TINT, BUTTON_PRESSED_TINT, BUTTON_TEXT, HEADER_TEXT, LABEL_TEXT,
            LOSS_TEXT,
        },
        scale::{TEXEL, texels},
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
                float_health_changes.run_if(resource_exists::<Typewriter>),
                show_ailment.run_if(resource_exists::<Typewriter>),
                rise_health_changes,
                bob_continue.run_if(resource_exists::<Typewriter>),
            )
                .in_set(YarnSpinnerDialogueViewSystemSet),
        );
}

#[derive(Component, Clone, Default)]
pub(super) struct DialogueRoot;

#[derive(Component, Clone, Default)]
struct DialogueNameNode;

#[derive(Component, Clone, Default)]
struct DialogueTextNode;

#[derive(Component, Clone, Default)]
pub(super) struct DialogueOptionsNode;

#[derive(Component, Clone, Default)]
struct DialogueContinueNode;

#[derive(Component, Clone, Copy, Default)]
struct DialogueHealthFill(Element);

#[derive(Component, Clone, Default)]
struct ShownHealth(Health);

// the scrolling text indicating the impact a herb had
#[derive(Component, Clone, Default)]
struct RisingChange {
    age: f32,
}

const RISE_SECONDS: f32 = 1.0;
const RISE_TEXELS: f32 = 14.0;

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
    npc: Query<&BodyOf, With<InDialogue>>,
    visitors: Query<(&Health, Option<&Ailing>)>,
    clock: Single<&Clock>,
) {
    commands.init_resource::<Typewriter>();

    let health = npc
        .iter()
        .find_map(|body| visitors.get(body.0).ok())
        .map(|(&health, ailing)| -> Box<dyn Scene> {
            let ailment = ailing.map(|ailing| (ailing.ailment, ailing.next - clock.hours));
            Box::new(health_panel(health, ailment))
        })
        .into_iter()
        .collect::<Vec<_>>();
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
                { health }
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
                --
                #DialogueBack
                Node {
                    position_type: PositionType::Absolute,
                    left: texels(8),
                    bottom: texels(4),
                    column_gap: texels(1),
                }
                Pickable::IGNORE
                Children [
                    @prompt(KEY_LEFT)
                    --
                    @prompt(KEY_ESCAPE)
                ]
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

#[derive(Component, Clone, Default)]
struct AilmentTitle;

#[derive(Component, Clone, Default)]
struct AilmentCountdown;

#[derive(Component, Clone, Default)]
struct AilmentMark(Element);

fn countdown(hours: f32) -> String {
    format!("{}h", hours.ceil().max(0.0))
}

fn mark(ailment: Ailment, element: Element) -> String {
    ailment
        .effects()
        .into_iter()
        .find(|&(affected, _)| affected == element)
        .map(|(_, delta)| format!("{delta:+}"))
        .unwrap_or_default()
}

fn health_panel(health: Health, ailment: Option<(Ailment, f32)>) -> impl Scene {
    let title = ailment.map_or("Health".to_string(), |(ailment, _)| ailment.name());
    let hours = ailment
        .map(|(_, hours)| countdown(hours))
        .unwrap_or_default();
    bsn! {
        #DialogueHealth
        ShownHealth(health)
        // sized by what's in it, so the title stays on one line
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: texels(1),
            padding: UiRect::all(texels(4)),
        }
        @widget::panel()
        Pickable::IGNORE
        Children [
            Node {
                justify_content: JustifyContent::SpaceBetween,
                column_gap: texels(6),
                margin: UiRect {
                    bottom: texels(2),
                },
            }
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
            { health_rows(health, ailment.map(|(ailment, _)| ailment)) }
        ]
    }
}

fn show_ailment(
    npc: Query<&BodyOf, With<InDialogue>>,
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
    let ailing = npc.iter().find_map(|body| visitors.get(body.0).ok());
    for (mut text, title, counting, marked) in &mut texts {
        let shown = match (ailing, title, counting, marked) {
            (Some(ailing), true, ..) => ailing.ailment.name(),
            (None, true, ..) => "Health".to_string(),
            (Some(ailing), _, true, _) => countdown(ailing.next - clock.hours),
            (Some(ailing), .., Some(&AilmentMark(element))) => mark(ailing.ailment, element),
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
                .map(|ailment| mark(ailment, element))
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
