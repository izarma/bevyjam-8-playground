// worldspace yield preview ui for gatherables
use bevy::{
    camera::{ImageRenderTarget, RenderTarget},
    image::ImageSampler,
    light::NotShadowCaster,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::{DebandDither, Tonemapping},
    },
};
use pixel_art::PixelSprite;
use plants::{Growing, Plant};

use super::InReach;
use crate::{
    GameplayScene,
    ui::{
        font,
        menus::Menu,
        palette::{BUTTON_TEXT, GAIN_TEXT, HEADER_TEXT, LOSS_TEXT},
        scale::{TEXEL, texels},
        widget,
    },
};

// meters distance the panel is above the plant
const GAP: f32 = 0.35;
const CANVAS: UVec2 = UVec2::new(128, 96);
// this is DIFFERENT from our normal one because if its not the ui is HUGE in the world
const TEXELS_PER_METER: f32 = 32.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            show_panel
                .after(super::find_in_reach)
                .run_if(in_state(GameplayScene::Outdoors)),
            fit_canvas_scale.run_if(resource_changed::<UiScale>),
        ),
    );
}

#[derive(Clone, Copy)]
enum Element {
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl Element {
    fn name(self) -> &'static str {
        match self {
            Element::Wood => "Wood",
            Element::Fire => "Fire",
            Element::Earth => "Earth",
            Element::Metal => "Metal",
            Element::Water => "Water",
        }
    }
}

// placeholder
fn name(plant: Plant) -> &'static str {
    match plant {
        Plant::Plant01 => "Plant 01",
        Plant::Plant06 => "Plant 06",
        Plant::Plant13 => "Plant 13",
    }
}

fn yields(plant: Plant) -> &'static [(Element, i32)] {
    match plant {
        Plant::Plant01 => &[(Element::Wood, 2), (Element::Metal, -1)],
        Plant::Plant06 => &[(Element::Water, 3)],
        Plant::Plant13 => &[(Element::Earth, 1), (Element::Fire, -1)],
    }
}

#[derive(Component)]
struct YieldPanel {
    plant: Entity,
}

#[derive(Component)]
#[relationship(relationship_target = PanelParts)]
struct PartOfPanel(Entity);

#[derive(Component)]
#[relationship_target(relationship = PartOfPanel, linked_spawn)]
struct PanelParts(Vec<Entity>);

#[derive(Component)]
struct CanvasCamera;

fn show_panel(
    mut commands: Commands,
    in_reach: Res<InReach>,
    menu: Res<State<Menu>>,
    panels: Query<(Entity, &YieldPanel)>,
    plants: Query<(&Growing, &GlobalTransform, &PixelSprite)>,
    mut images: ResMut<Assets<Image>>,
    ui_scale: Res<UiScale>,
) {
    // hidden while a menu or minigame is up
    let target = in_reach.0.filter(|_| *menu.get() == Menu::None);
    for (panel, shown) in &panels {
        if Some(shown.plant) != target {
            commands.entity(panel).despawn();
        }
    }
    let Some(plant) = target else {
        return;
    };
    if panels.iter().any(|(_, shown)| shown.plant == plant) {
        return;
    }
    let Ok((Growing(kind), at, sprite)) = plants.get(plant) else {
        return;
    };
    let Some(sprite_size) = images.get(&sprite.image).map(Image::size) else {
        return;
    };
    let plant_height = (1.0 - sprite.anchor.y) * sprite.rect(sprite_size).height() as f32
        / sprite.pixels_per_meter
        * at.scale().y;

    let mut canvas =
        Image::new_target_texture(CANVAS.x, CANVAS.y, TextureFormat::Bgra8UnormSrgb, None);
    canvas.sampler = ImageSampler::nearest();
    let canvas = images.add(canvas);
    let panel_entity = commands
        .spawn((
            Name::new("Yield Panel"),
            YieldPanel { plant },
            PixelSprite {
                image: canvas.clone(),
                pixels_per_meter: TEXELS_PER_METER,
                unlit: true,
                ..default()
            },
            NotShadowCaster,
            Transform::from_translation(at.translation() + Vec3::Y * (plant_height + GAP)),
            DespawnOnExit(GameplayScene::Outdoors),
        ))
        .id();
    let camera = commands
        .spawn((
            Name::new("Yield Panel Camera"),
            CanvasCamera,
            Camera2d,
            Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            Tonemapping::None,
            DebandDither::Disabled,
            RenderTarget::Image(ImageRenderTarget {
                handle: canvas,
                scale_factor: canvas_scale(&ui_scale),
            }),
            PartOfPanel(panel_entity),
        ))
        .id();
    commands
        .spawn_scene(panel(name(*kind), yields(*kind)))
        .insert((UiTargetCamera(camera), PartOfPanel(panel_entity)));
}

fn canvas_scale(ui_scale: &UiScale) -> f32 {
    1.0 / (TEXEL * ui_scale.0)
}

fn fit_canvas_scale(
    ui_scale: Res<UiScale>,
    mut targets: Query<&mut RenderTarget, With<CanvasCamera>>,
) {
    for mut target in &mut targets {
        if let RenderTarget::Image(image) = &mut *target {
            image.scale_factor = canvas_scale(&ui_scale);
        }
    }
}

fn panel(name: &'static str, yields: &[(Element, i32)]) -> impl Scene {
    let rows = yields
        .iter()
        .map(|&(element, amount)| -> Box<dyn Scene> {
            let color = if amount > 0 { GAIN_TEXT } else { LOSS_TEXT };
            Box::new(bsn! {
                Node {
                    column_gap: texels(4),
                }
                Children [
                    Text({ element.name().to_string() })
                    @font::text(1)
                    TextColor(BUTTON_TEXT)
                    --
                    Text({ format!("{amount:+}") })
                    @font::text(1)
                    TextColor(color)
                ]
            })
        })
        .collect::<Vec<_>>();

    bsn! {
        #YieldCanvas
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
        }
        Pickable::IGNORE
        Children [
            #YieldPanel
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(texels(5)),
            }
            @widget::panel()
            Pickable::IGNORE
            Children [
                Text({ name.to_string() })
                @font::text(1)
                TextColor(HEADER_TEXT)
                Node {
                    margin: UiRect::bottom(texels(2)),
                }
                --
                { rows }
            ]
        ]
    }
}
