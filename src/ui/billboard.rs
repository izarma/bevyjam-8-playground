// ui drawn into the world: a camera renders a ui scene to an image,
// and a pixel sprite shows that image
use bevy::{
    camera::{ImageRenderTarget, RenderTarget, visibility::RenderLayers},
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

use crate::{GameState, GameplayScene, ui::scale::TEXEL};

// meters between a sprite's top and a billboard above it
const GAP: f32 = 0.35;
// this is DIFFERENT from our normal one because if its not the ui is HUGE in the world
const TEXELS_PER_METER: f32 = 32.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, fit_canvas_scale.run_if(resource_changed::<UiScale>))
        .add_systems(OnEnter(GameState::Loading), warm_up);
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
pub(crate) struct Billboards<'w, 's> {
    commands: Commands<'w, 's>,
    images: ResMut<'w, Assets<Image>>,
    ui_scale: Res<'w, UiScale>,
}

impl Billboards<'_, '_> {
    // where a billboard over this sprite goes, once its image has loaded
    pub(crate) fn above(&self, sprite: &PixelSprite, at: &GlobalTransform) -> Option<Vec3> {
        let size = self.images.get(&sprite.image)?.size();
        let height = (1.0 - sprite.anchor.y) * sprite.rect(size).height() as f32
            / sprite.pixels_per_meter
            * at.scale().y;
        Some(at.translation() + Vec3::Y * (height + GAP))
    }

    // `ui` lays out over a `canvas` texels big, bottom center at `at`
    pub(crate) fn spawn(
        &mut self,
        canvas: UVec2,
        at: Vec3,
        ui: impl Scene,
        scene: GameplayScene,
    ) -> EntityCommands<'_> {
        let (image, camera) = self.canvas(canvas);
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
                DespawnOnExit(scene),
            ))
            .id();
        let camera = self
            .commands
            .spawn((
                Name::new("Billboard Camera"),
                camera,
                PartOfBillboard(billboard),
            ))
            .id();
        self.commands
            .spawn_scene(ui)
            .insert((UiTargetCamera(camera), PartOfBillboard(billboard)));
        self.commands.entity(billboard)
    }
}

impl Billboards<'_, '_> {
    fn canvas(&mut self, size: UVec2) -> (Handle<Image>, impl Bundle) {
        let mut image =
            Image::new_target_texture(size.x, size.y, TextureFormat::Bgra8UnormSrgb, None);
        image.sampler = ImageSampler::nearest();
        let image = self.images.add(image);
        let camera = (
            CanvasCamera,
            Camera2d,
            Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            Tonemapping::None,
            DebandDither::Disabled,
            RenderLayers::none(),
            RenderTarget::Image(ImageRenderTarget {
                handle: image.clone(),
                scale_factor: canvas_scale(&self.ui_scale),
            }),
        );
        (image, camera)
    }
}

fn warm_up(mut billboards: Billboards) {
    let (_, camera) = billboards.canvas(UVec2::splat(8));
    let camera = billboards
        .commands
        .spawn((
            Name::new("Billboard Warm-up"),
            camera,
            DespawnOnExit(GameState::Title),
        ))
        .id();
    billboards.commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::WHITE),
        UiTargetCamera(camera),
        DespawnOnExit(GameState::Title),
    ));
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
