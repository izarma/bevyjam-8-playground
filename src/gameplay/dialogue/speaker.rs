use bevy::prelude::*;
use bevy_yarnspinner::events::DialogueCompleted;

use super::SpeakerChangeEvent;
use crate::gameplay::{CharacterName, SpeakerSprite};

#[derive(Resource, Default)]
struct ActiveSpeaker {
    sprite: Option<Entity>,
    rest_y: f32,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ActiveSpeaker>()
        .add_observer(reset_speaker)
        .add_systems(
            Update,
            animate_speaker.in_set(super::YarnSpinnerDialogueViewSystemSet),
        );
}

/// Resolves [`SpeakerChangeEvent`]s against the world's characters and bobs the
/// active speaker's sprite while its line is being typed out.
fn animate_speaker(
    time: Res<Time>,
    mut events: MessageReader<SpeakerChangeEvent>,
    characters: Query<(Entity, &CharacterName)>,
    mut sprites: Query<(Entity, &ChildOf, &mut Transform), With<SpeakerSprite>>,
    mut active: ResMut<ActiveSpeaker>,
) {
    for event in events.read() {
        let Some((entity, _)) = characters
            .iter()
            .find(|(_, name)| name.0 == event.character_name.as_str())
        else {
            continue;
        };

        let Some((sprite, _, _)) = sprites
            .iter_mut()
            .find(|(_, parent, _)| parent.parent() == entity)
        else {
            continue;
        };

        if event.speaking {
            if active.sprite != Some(sprite) {
                if let Some(previous) = active.sprite {
                    if let Ok((_, _, mut transform)) = sprites.get_mut(previous) {
                        transform.translation.y = active.rest_y;
                    }
                }
            }
            active.rest_y = sprites
                .get(sprite)
                .map(|(_, _, transform)| transform.translation.y)
                .unwrap_or(0.0);
            active.sprite = Some(sprite);
        } else if active.sprite == Some(sprite) {
            if let Ok((_, _, mut transform)) = sprites.get_mut(sprite) {
                transform.translation.y = active.rest_y;
            }
            active.sprite = None;
        }
    }

    let Some(sprite) = active.sprite else {
        return;
    };
    let Ok((_, _, mut transform)) = sprites.get_mut(sprite) else {
        return;
    };
    let lift = (time.elapsed_secs() * 3.0).sin().powi(2) * 0.15;
    transform.translation.y = active.rest_y + lift;
}

fn reset_speaker(
    _: On<DialogueCompleted>,
    mut active: ResMut<ActiveSpeaker>,
    mut transforms: Query<&mut Transform>,
) {
    if let Some(sprite) = active.sprite {
        if let Ok(mut transform) = transforms.get_mut(sprite) {
            transform.translation.y = active.rest_y;
        }
    }
    active.sprite = None;
}
