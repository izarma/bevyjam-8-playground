// render at a lower resolution with a second camera
use bevy::{
    camera::{CameraUpdateSystems, ImageRenderTarget, RenderTarget},
    image::ImageSampler,
    prelude::*,
    render::render_resource::TextureFormat,
    ui::IsDefaultUiCamera,
    window::PrimaryWindow,
};

use crate::graphics::GraphicsSettings;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, apply_render_scale.before(CameraUpdateSystems));
}

// looks at the scene but draws to an image
#[derive(Component)]
struct ScaledView(Handle<Image>);

/// draws the image itself, draws the ui on top
#[derive(Component)]
struct PresentCamera;

/// the full-window ui node that shows the scaled image.
#[derive(Component)]
struct PresentImage;

fn apply_render_scale(
    mut commands: Commands,
    settings: Res<GraphicsSettings>,
    window: Single<&Window, With<PrimaryWindow>>,
    // every camera drawing the 3d world, title screen included
    mut cameras: Query<
        (
            Entity,
            &Camera,
            Option<&ScaledView>,
            Has<IsDefaultUiCamera>,
            &mut Projection,
        ),
        With<Camera3d>,
    >,
    presents: Query<Entity, Or<(With<PresentCamera>, With<PresentImage>)>>,
    mut present_images: Query<&mut ImageNode, With<PresentImage>>,
    mut images: ResMut<Assets<Image>>,
) {
    let scale = settings.render_scale as f32 / 100.0;
    let drawing = cameras.iter().any(|(_, camera, ..)| camera.is_active);
    if scale >= 1.0 || !drawing {
        for (camera, _, scaled, _, mut projection) in &mut cameras {
            if scaled.is_some() {
                commands
                    .entity(camera)
                    .insert((RenderTarget::default(), IsDefaultUiCamera))
                    .remove::<ScaledView>();
                // bevy recomputes a camera's size on projection changes but
                // not on render target changes
                projection.set_changed();
            }
        }
        for entity in &presents {
            commands.entity(entity).despawn();
        }
        return;
    }

    let size = (window.physical_size().as_vec2() * scale)
        .ceil()
        .max(Vec2::ONE)
        .as_uvec2();
    for (camera, active, scaled, ui, mut projection) in &mut cameras {
        // an inactive camera never draws into its image
        if !active.is_active {
            continue;
        }
        if ui {
            commands.entity(camera).remove::<IsDefaultUiCamera>();
        }
        let current = scaled.map(|ScaledView(handle)| handle).filter(|handle| {
            images
                .get(*handle)
                .is_some_and(|image| image.size() == size)
        });
        let handle = match current {
            Some(handle) => handle.clone(),
            None => {
                let mut image =
                    Image::new_target_texture(size.x, size.y, TextureFormat::Bgra8UnormSrgb, None);
                image.sampler = ImageSampler::nearest();
                let handle = images.add(image);
                commands.entity(camera).insert((
                    RenderTarget::Image(ImageRenderTarget {
                        handle: handle.clone(),
                        scale_factor: 1.0,
                    }),
                    ScaledView(handle.clone()),
                ));
                projection.set_changed();
                handle
            }
        };

        if presents.is_empty() {
            commands.spawn((
                PresentCamera,
                Camera2d,
                Camera {
                    order: 1,
                    ..default()
                },
                IsDefaultUiCamera,
            ));
            commands.spawn((
                PresentImage,
                ImageNode::new(handle),
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                GlobalZIndex(-1),
                Pickable::IGNORE,
            ));
        } else {
            for mut node in &mut present_images {
                if node.image != handle {
                    node.image = handle.clone();
                }
            }
        }
    }
}
