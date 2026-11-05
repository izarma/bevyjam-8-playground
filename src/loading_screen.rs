use crate::{
    scene_ready::Loading,
    ui::{
        font,
        scale::{pixels_per_texel, texels},
    },
};
use bevy::{
    asset::{AssetPath, embedded_asset, embedded_path},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    ui_render::prelude::{MaterialNode, UiMaterial, UiMaterialPlugin},
    window::PrimaryWindow,
};

const SWEEP_SECONDS: f32 = 0.6;

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "loading_screen.wesl");
    app.add_plugins(UiMaterialPlugin::<LoadingScreenMaterial>::default())
        .add_systems(Startup, spawn_loading_screen)
        .add_systems(Update, sweep);
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct LoadingScreenMaterial {
    #[uniform(0)]
    swept: f32,
    #[uniform(0)]
    pixels_per_texel: f32,
}

impl UiMaterial for LoadingScreenMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("loading_screen.wesl")).with_source("embedded"),
        )
    }
}

#[derive(Component, Default, Clone)]
struct LoadingLabel;

fn spawn_loading_screen(
    mut commands: Commands,
    mut materials: ResMut<Assets<LoadingScreenMaterial>>,
) {
    commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
            }
            GlobalZIndex(i32::MAX)
            Pickable::IGNORE
            Children [
                #LoadingLabel
                LoadingLabel
                Text("Loading...")
                @font::text(1)
                TextColor(Color::WHITE)
                Node {
                    position_type: PositionType::Absolute,
                    right: texels(4),
                    bottom: texels(4),
                }
                Visibility::Hidden
                Pickable::IGNORE
            ]
        })
        .insert(MaterialNode(materials.add(LoadingScreenMaterial {
            swept: 1.0,
            pixels_per_texel: 1.0,
        })));
}

fn sweep(
    time: Res<Time<Real>>,
    loading: Query<(), Loading>,
    screen: Single<&MaterialNode<LoadingScreenMaterial>>,
    mut label: Single<&mut Visibility, With<LoadingLabel>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut materials: ResMut<Assets<LoadingScreenMaterial>>,
) {
    let loading = !loading.is_empty();
    label.set_if_neq(if loading {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    let Some(material) = materials.get(&screen.0) else {
        return;
    };
    let swept = if loading {
        0.0
    } else {
        (material.swept + time.delta_secs() / SWEEP_SECONDS).min(1.0)
    };
    let pixels_per_texel = pixels_per_texel(&window);
    if (material.swept != swept || material.pixels_per_texel != pixels_per_texel)
        && let Some(mut material) = materials.get_mut(&screen.0)
    {
        material.swept = swept;
        material.pixels_per_texel = pixels_per_texel;
    }
}
