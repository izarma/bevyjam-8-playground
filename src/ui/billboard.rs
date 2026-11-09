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
use pixel_art::{PixelSprite, PixelSpriteMaterial};

use crate::{
    GameplayScene,
    camera::SceneCamera,
    scene_ready::Loading,
    ui::{font, scale::TEXEL, widget},
};

// meters between a sprite's top and a billboard above it
const GAP: f32 = 0.35;
const FADE_SECONDS: f32 = 0.2;
// this is DIFFERENT from our normal one because if its not the ui is HUGE in the world
const TEXELS_PER_METER: f32 = 32.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, fit_canvas_scale.run_if(resource_changed::<UiScale>))
        .add_systems(Update, (prime, fade).chain());
}

#[derive(Component)]
#[relationship(relationship_target = BillboardParts)]
struct PartOfBillboard(Entity);

#[derive(Component)]
#[relationship_target(relationship = PartOfBillboard, linked_spawn)]
struct BillboardParts(Vec<Entity>);

#[derive(Component)]
struct CanvasCamera;

#[derive(Component, Default)]
pub(crate) struct Fade {
    shown: f32,
    leaving: bool,
}

impl Fade {
    pub(crate) fn leave(&mut self) {
        self.leaving = true;
    }

    pub(crate) fn stay(&mut self) {
        self.leaving = false;
    }
}

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
                // shown once the fade has made it see-through
                Visibility::Hidden,
                Fade::default(),
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

#[derive(Component)]
pub(crate) struct Primer;

// shader preloader for billboards
fn prime(
    mut commands: Commands,
    mut billboards: Billboards,
    loading: Query<(), Loading>,
    primers: Query<Entity, With<Primer>>,
    cameras: Query<Entity, SceneCamera>,
    scene: Option<Res<State<GameplayScene>>>,
) {
    if loading.is_empty() {
        for primer in &primers {
            commands.entity(primer).despawn();
        }
        return;
    }
    let (Some(scene), Some(camera)) = (scene, cameras.iter().next()) else {
        return;
    };
    if !primers.is_empty() {
        return;
    }
    billboards
        .spawn(UVec2::splat(16), Vec3::NEG_Z * 2.0, primer(), *scene.get())
        .insert((Name::new("Billboard Primer"), Primer, ChildOf(camera)));
}

fn primer() -> impl Scene {
    bsn! {
        Node
        @widget::panel()
        Children [
            Node {
                width: px(2),
                height: px(2),
            }
            BackgroundColor(Color::WHITE)
            --
            Text("0")
            @font::text(1)
        ]
    }
}

fn fade(
    mut commands: Commands,
    time: Res<Time>,
    mut billboards: Query<(
        Entity,
        &mut Fade,
        &mut Visibility,
        Option<&MeshMaterial3d<PixelSpriteMaterial>>,
    )>,
    mut materials: ResMut<Assets<PixelSpriteMaterial>>,
) {
    let step = time.delta_secs() / FADE_SECONDS;
    for (billboard, mut fade, mut visibility, material) in &mut billboards {
        let Some(material) = material else {
            continue;
        };
        let shown = if fade.leaving {
            fade.shown - step
        } else {
            fade.shown + step
        }
        .clamp(0.0, 1.0);
        if fade.leaving && shown == 0.0 {
            commands.entity(billboard).despawn();
            continue;
        }
        fade.shown = shown;
        // each billboard draws its own canvas, so its material is its own. only
        // written when it changes: writing marks it for re-upload
        let stale = materials
            .get(&material.0)
            .is_some_and(|material| material.base.base_color.alpha() != shown);
        if stale && let Some(mut material) = materials.get_mut(&material.0) {
            material.base.base_color.set_alpha(shown);
        }
        visibility.set_if_neq(Visibility::Inherited);
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
