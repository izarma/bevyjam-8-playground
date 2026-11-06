mod health;
mod interaction;

use health::health_rows;

use bevy::{platform::time::Instant, prelude::*};
use bevy_yarnspinner::{
    events::{DialogueCompleted, DialogueStarted, PresentLine},
    prelude::{DialogueOption, LocalizedLine},
};
use unicode_segmentation::UnicodeSegmentation;

use super::{SpeakerChangeEvent, YarnSpinnerDialogueViewSystemSet, concoction::Concoction};
use crate::{
    GameplayScene,
    gameplay::{BodyOf, Health, InDialogue},
    ui::{
        font,
        input_prompt::{KEY_ESCAPE, KEY_LEFT, prompt},
        palette::{HEADER_TEXT, LABEL_TEXT},
        scale::texels,
        widget,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((health::plugin, interaction::plugin))
        .add_observer(spawn_dialogue_ui)
        .add_observer(despawn_dialogue_ui)
        .add_observer(present_line)
        .add_systems(
            Update,
            (
                write_text.run_if(resource_exists::<Typewriter>),
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

#[derive(Component, Clone, Default)]
struct ShownHealth(Health);

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
    visitors: Query<&Health>,
) {
    commands.init_resource::<Typewriter>();
    let health = npc
        .iter()
        .find_map(|body| visitors.get(body.0).ok())
        .copied()
        .unwrap_or_default();
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
                ShownHealth(health)
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
