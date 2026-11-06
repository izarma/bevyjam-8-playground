use bevy::{prelude::*, ui::UiGlobalTransform, window::PrimaryWindow};
use pixel_art::PixelSprite;

use super::{Fades, HarvestScratch, Progress, Won};
use crate::{
    Harvesting, PausableSystems, Pause,
    ui::{
        input_prompt::{KEY_ESCAPE, KEY_LEFT, MOUSE, prompt},
        scale::{TEXEL, texels},
    },
};

// how many ui texels wide each sprite texel is
const TEXEL_SCALE: u32 = 8;
const GRAVITY: f32 = 900.0;
const MAX_KICK: f32 = 60.0;
const POP: f32 = 80.0;
const HAND_RADIUS: f32 = 1.5;
// chroma kept by everything you can't wipe
const FADED_CHROMA: f32 = 0.5;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        wipe.in_set(PausableSystems)
            .run_if(in_state(Harvesting(true))),
    )
    .add_systems(OnEnter(Pause(true)), forget_cursor)
    .add_systems(Update, fall);
}

#[derive(Component)]
struct Board {
    // the minigame this board scores for
    overlay: Entity,
    columns: u32,
    // the collection of still-in-place texels
    stuck: Vec<Option<Entity>>,
    texels: usize,
    last_cursor: Option<Vec2>,
}

#[derive(Component)]
struct Falling {
    position: Vec2,
    velocity: Vec2,
}

/// A board of the `growing` sprite where only the texels that differ from
/// `harvested` can be wiped, revealing the harvested ones.
pub(super) fn spawn(
    commands: &mut Commands,
    overlay: Entity,
    growing: &PixelSprite,
    harvested: &PixelSprite,
    images: &Assets<Image>,
) -> bool {
    let Some(image) = images.get(&growing.image) else {
        return false;
    };
    let rect = growing.rect(image.size());
    let after = harvested.rect(image.size());
    let size = rect.size();
    let board = commands
        .spawn((
            Name::new("Wipe Board"),
            Node {
                width: snapped((size.x * TEXEL_SCALE) as f32),
                height: snapped((size.y * TEXEL_SCALE) as f32),
                ..default()
            },
            ChildOf(overlay),
        ))
        .id();
    // input hint adjacent to the tooltip billboard
    let hint = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(100),
                bottom: px(0),
                margin: UiRect::left(texels(4)),
                ..default()
            },
            ChildOf(board),
        ))
        .id();
    commands
        .spawn_scene(prompt(MOUSE))
        .insert((Fades(Color::WHITE), ChildOf(hint)));
    let back = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: texels(4),
                bottom: texels(4),
                column_gap: texels(1),
                ..default()
            },
            ChildOf(overlay),
        ))
        .id();
    for key in [KEY_LEFT, KEY_ESCAPE] {
        commands
            .spawn_scene(prompt(key))
            .insert((Fades(Color::WHITE), ChildOf(back)));
    }

    let texel = |rect: URect, x: u32, y: u32| {
        let x = if growing.flip_x {
            rect.max.x - 1 - x
        } else {
            rect.min.x + x
        };
        image.get_color_at(x, rect.min.y + y)
    };
    let shown = |color: Color| color.alpha() > 0.0;
    let mut stuck = vec![None; (size.x * size.y) as usize];
    for y in 0..size.y {
        for x in 0..size.x {
            let (Ok(before), Ok(harvested)) = (texel(rect, x, y), texel(after, x, y)) else {
                return false;
            };
            let at = Vec2::new(x as f32, y as f32) * TEXEL_SCALE as f32;
            let unchanged = before == harvested || !shown(before) && !shown(harvested);
            if unchanged {
                if shown(before) {
                    commands.spawn((square(at), Fades(faded(before)), ChildOf(board)));
                }
                continue;
            }
            if shown(harvested) {
                commands.spawn((square(at), Fades(faded(harvested)), ChildOf(board)));
            }
            if shown(before) {
                stuck[(y * size.x + x) as usize] = Some(
                    commands
                        .spawn((square(at), Fades(before), ChildOf(board)))
                        .id(),
                );
            }
        }
    }
    commands.entity(board).insert(Board {
        overlay,
        columns: size.x,
        texels: stuck.iter().flatten().count(),
        stuck,
        last_cursor: None,
    });
    true
}

fn faded(color: Color) -> Color {
    let color = Oklcha::from(color);
    color.with_chroma(color.chroma * FADED_CHROMA).into()
}

fn square(at: Vec2) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: snapped(at.x),
            top: snapped(at.y),
            width: snapped(TEXEL_SCALE as f32),
            height: snapped(TEXEL_SCALE as f32),
            ..default()
        },
        ImageNode::solid_color(Color::NONE),
        Pickable::IGNORE,
    )
}

// keep them on the grid
fn snapped(texels: f32) -> Val {
    px(texels.round() * TEXEL)
}

fn wipe(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut boards: Query<(&mut Board, &ComputedNode, &UiGlobalTransform)>,
    mut progress: Query<&mut Progress>,
) {
    let cursor = window
        .cursor_position()
        .map(|logical| logical * window.scale_factor());
    for (mut board, node, transform) in &mut boards {
        let from = board.last_cursor.or(cursor);
        board.last_cursor = cursor;
        let (Some(from), Some(to)) = (from, cursor) else {
            continue;
        };

        let cell = node.size().x / board.columns as f32;
        if cell <= 0.0 {
            continue;
        }

        // the hand "swept" from its previouss position to its current one
        let top_left = transform.translation - node.size() / 2.0;
        let sweep = Segment2d::new((from - top_left) / cell, (to - top_left) / cell);
        let columns = board.columns;
        let mut scratched = false;
        for (index, slot) in board.stuck.iter_mut().enumerate() {
            let spot = UVec2::new(index as u32 % columns, index as u32 / columns).as_vec2();
            let center = spot + 0.5;
            if sweep.closest_point(center).distance(center) > HAND_RADIUS {
                continue;
            }
            if let Some(texel) = slot.take() {
                scratched = true;
                commands.entity(texel).insert((
                    Falling {
                        position: spot * TEXEL_SCALE as f32,
                        velocity: Vec2::new(rand::random_range(-MAX_KICK..MAX_KICK), -POP),
                    },
                    ZIndex(1),
                ));
            }
        }
        if scratched {
            commands.trigger(HarvestScratch);
        }
        let left = board.stuck.iter().flatten().count();
        if let Ok(mut progress) = progress.get_mut(board.overlay) {
            progress.done = 1.0 - left as f32 / board.texels.max(1) as f32;
        }
        if left == 0 {
            commands.trigger(Won {
                entity: board.overlay,
            });
        }
    }
}

// reset the minigame's view of the cursor between pauses
fn forget_cursor(mut boards: Query<&mut Board>) {
    for mut board in &mut boards {
        board.last_cursor = None;
    }
}

fn fall(time: Res<Time>, mut texels: Query<(&mut Falling, &mut Node)>) {
    let dt = time.delta_secs();
    for (mut falling, mut node) in &mut texels {
        falling.velocity.y += GRAVITY * dt;
        let step = falling.velocity * dt;
        falling.position += step;
        node.left = snapped(falling.position.x);
        node.top = snapped(falling.position.y);
    }
}
