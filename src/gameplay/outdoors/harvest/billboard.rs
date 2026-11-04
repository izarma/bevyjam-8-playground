// ui drawn into the world: a camera renders a ui scene to an image,
// and a pixel sprite shows that image
use bevy::{
    camera::{ImageRenderTarget, RenderTarget},
    ecs::system::SystemParam,
    image::ImageSampler,
    light::NotShadowCaster,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::{DebandDither, Tonemapping},
    },
};
use pixel_art::PixelSprite;

use crate::{GameplayScene, ui::scale::TEXEL};

// meters between a sprite's top and a billboard above it
const GAP: f32 = 0.35;
// this is DIFFERENT from our normal one because if its not the ui is HUGE in the world
const TEXELS_PER_METER: f32 = 32.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, fit_canvas_scale.run_if(resource_changed::<UiScale>));
}

#[derive(Component)]
#[relationship(relationship_target = BillboardParts)]
struct PartOfBillboard(Entity);

#[derive(Component)]
#[relationship_target(relationship = PartOfBillboard, linked_spawn)]
struct BillboardParts(Vec<Entity>);

#[derive(Component)]
struct CanvasCamera;

#[derive(SystemParam)]
pub(super) struct Billboards<'w, 's> {
    commands: Commands<'w, 's>,
    images: ResMut<'w, Assets<Image>>,
    ui_scale: Res<'w, UiScale>,
}

impl Billboards<'_, '_> {
    // where a billboard over this sprite goes, once its image has loaded
    pub(super) fn above(&self, sprite: &PixelSprite, at: &GlobalTransform) -> Option<Vec3> {
        let size = self.images.get(&sprite.image)?.size();
        let height = (1.0 - sprite.anchor.y) * sprite.rect(size).height() as f32
            / sprite.pixels_per_meter
            * at.scale().y;
        Some(at.translation() + Vec3::Y * (height + GAP))
    }

    // `ui` lays out over a `canvas` texels big, bottom center at `at`
    pub(super) fn spawn(&mut self, canvas: UVec2, at: Vec3, ui: impl Scene) -> EntityCommands<'_> {
        let mut image =
            Image::new_target_texture(canvas.x, canvas.y, TextureFormat::Bgra8UnormSrgb, None);
        image.sampler = ImageSampler::nearest();
        let image = self.images.add(image);
        let billboard = self
            .commands
            .spawn((
                PixelSprite {
                    image: image.clone(),
                    pixels_per_meter: TEXELS_PER_METER,
                    unlit: true,
                    ..default()
                },
                NotShadowCaster,
                Transform::from_translation(at),
                DespawnOnExit(GameplayScene::Outdoors),
            ))
            .id();
        let camera = self
            .commands
            .spawn((
                Name::new("Billboard Camera"),
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
                    handle: image,
                    scale_factor: canvas_scale(&self.ui_scale),
                }),
                PartOfBillboard(billboard),
            ))
            .id();
        self.commands
            .spawn_scene(ui)
            .insert((UiTargetCamera(camera), PartOfBillboard(billboard)));
        self.commands.entity(billboard)
    }
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
