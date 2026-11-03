use bevy::{
    asset::{RenderAssetUsages, embedded_asset},
    core_pipeline::prepass::DepthPrepass,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    light::NotShadowCaster,
    pbr::{ExtendedMaterial, MaterialExtension, ScreenSpaceTransmission},
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat},
    shader::ShaderRef,
};

pub(crate) fn plugin(app: &mut App) {
    embedded_asset!(app, "water.wesl");
    app.add_plugins(MaterialPlugin::<PixelWaterMaterial>::default())
        .add_systems(Update, (camera_water_dependencies, recover_water).chain());
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct PixelWater {
    pub shallow: Color,
    pub deep: Color,
    pub foam: Color,
    pub depth_range: f32,
    pub bands: u32,
    pub foam_depth: f32,
    /// how much deeper the shore foam reaches at the top of a lap
    pub lap_depth: f32,
    /// laps per second, in radians
    pub lap_speed: f32,
    pub refraction: f32,

    pub foam_amount: f32,
    pub foam_spacing: f32,
    pub foam_drift: Vec2,
    pub second_foam_turn: f32,
    pub second_foam_scale: f32,
    pub second_foam_drift: Vec2,
}

impl Default for PixelWater {
    fn default() -> Self {
        Self {
            shallow: Color::srgba(0.17, 0.34, 0.38, 0.45),
            deep: Color::srgba(0.035, 0.1, 0.18, 0.92),
            foam: Color::srgb(0.45, 0.5, 0.52),
            depth_range: 1.5,
            bands: 4,
            foam_depth: 0.05,
            lap_depth: 0.0,
            lap_speed: 1.5,
            refraction: 2.0,
            foam_amount: 0.2,
            foam_spacing: 0.25,
            foam_drift: Vec2::new(0.6, 0.25),
            second_foam_turn: 120.0,
            second_foam_scale: 0.75,
            second_foam_drift: Vec2::new(-0.35, 0.45),
        }
    }
}

pub type PixelWaterMaterial = ExtendedMaterial<StandardMaterial, PixelWaterExtension>;

#[derive(Asset, AsBindGroup, Reflect, Clone)]
pub struct PixelWaterExtension {
    #[uniform(100)]
    settings: PixelWaterSettings,
    #[texture(101)]
    #[sampler(102)]
    foam_mask: Handle<Image>,
}

impl MaterialExtension for PixelWaterExtension {
    fn fragment_shader() -> ShaderRef {
        "embedded://pixel_art/water.wesl".into()
    }
}

#[derive(ShaderType, Reflect, Clone)]
struct PixelWaterSettings {
    shallow: LinearRgba,
    deep: LinearRgba,
    foam_color: LinearRgba,
    depth_range: f32,
    bands: f32,
    texels_per_meter: f32,
    foam_depth: f32,
    lap_depth: f32,
    lap_speed: f32,
    refraction: f32,
    foam: Mat2,
    second_foam: Mat2,
    foam_drift: Vec2,
    second_foam_drift: Vec2,
}

impl PixelWater {
    fn material(&self, images: &mut Assets<Image>) -> PixelWaterMaterial {
        let foam = Mat2::from_diagonal(Vec2::splat(1.0 / (self.foam_spacing * FOAM_CELLS as f32)));
        let mask_texel = 1.0 / FOAM_TILE as f32;
        ExtendedMaterial {
            base: StandardMaterial {
                specular_transmission: 1.0,
                ior: 1.0,
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                ..default()
            },
            extension: PixelWaterExtension {
                settings: PixelWaterSettings {
                    shallow: self.shallow.into(),
                    deep: self.deep.into(),
                    foam_color: self.foam.into(),
                    depth_range: self.depth_range,
                    bands: self.bands as f32,
                    texels_per_meter: crate::TEXELS_PER_METER,
                    foam_depth: self.foam_depth,
                    lap_depth: self.lap_depth,
                    lap_speed: self.lap_speed,
                    refraction: self.refraction,
                    foam,
                    second_foam: foam
                        * Mat2::from_angle(self.second_foam_turn.to_radians())
                        * self.second_foam_scale,
                    foam_drift: self.foam_drift * mask_texel,
                    second_foam_drift: self.second_foam_drift * mask_texel,
                },
                foam_mask: images.add(foam_mask(1.0 - self.foam_amount)),
            },
        }
    }
}

#[derive(Component)]
struct WaterMaterial(Handle<PixelWaterMaterial>);

type MissingWaterDependencies = (
    With<Camera3d>,
    Or<(Without<DepthPrepass>, Without<ScreenSpaceTransmission>)>,
);

fn camera_water_dependencies(
    mut commands: Commands,
    cameras: Query<Entity, MissingWaterDependencies>,
) {
    for camera in &cameras {
        commands
            .entity(camera)
            .insert((DepthPrepass, ScreenSpaceTransmission::default()));
    }
}

// whenever you change the plane in jackdaw it throws away your material
// and you have to go get it again lol
// maybe i'm doing something wrong
fn recover_water(
    mut commands: Commands,
    waters: Query<(Entity, Ref<PixelWater>, Option<&WaterMaterial>, &Children)>,
    faces: Query<(), With<MeshMaterial3d<StandardMaterial>>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<PixelWaterMaterial>>,
) {
    for (entity, water, worn, children) in &waters {
        let handle = match worn {
            Some(WaterMaterial(handle)) => {
                if water.is_changed() {
                    let _ = materials.insert(handle, water.material(&mut images));
                }
                handle.clone()
            }
            None => {
                let handle = materials.add(water.material(&mut images));
                commands
                    .entity(entity)
                    .insert(WaterMaterial(handle.clone()));
                handle
            }
        };
        for &face in children {
            if faces.contains(face) {
                commands
                    .entity(face)
                    .remove::<MeshMaterial3d<StandardMaterial>>()
                    .insert((MeshMaterial3d(handle.clone()), NotShadowCaster));
            }
        }
    }
}

const FOAM_TILE: u32 = 128;
const FOAM_CELLS: u32 = 16;

fn foam_mask(threshold: f32) -> Image {
    // generate tiling noise for foam
    let mut level: Vec<u8> = (0..FOAM_TILE * FOAM_TILE)
        .map(|i| {
            let at = Vec2::new((i % FOAM_TILE) as f32, (i / FOAM_TILE) as f32) + 0.5;
            let noise = tiling_noise(at * FOAM_CELLS as f32 / FOAM_TILE as f32);
            if noise > threshold { 255 } else { 0 }
        })
        .collect();

    // mips for the foam, the same thing that is making terrain smooth from far away
    let mut data = level.clone();
    let mut side = FOAM_TILE;
    let mut levels = 1;
    while side > 1 {
        let half = side / 2;
        level = (0..half * half)
            .map(|i| {
                let (x, y) = (i % half * 2, i / half * 2);
                let sum: u32 = [(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)]
                    .iter()
                    .map(|&(x, y)| level[(y * side + x) as usize] as u32)
                    .sum();
                (sum / 4) as u8
            })
            .collect();
        data.extend_from_slice(&level);
        side = half;
        levels += 1;
    }

    let mut image = Image::new(
        Extent3d {
            width: FOAM_TILE,
            height: FOAM_TILE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data[..(FOAM_TILE * FOAM_TILE) as usize].to_vec(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = levels;
    image.data = Some(data);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

// https://thebookofshaders.com/10/
fn random(st: Vec2) -> f32 {
    let x = st.dot(Vec2::new(12.9898, 78.233)).sin() * 43758.547;
    x - x.floor()
}

// https://thebookofshaders.com/11/
fn tiling_noise(st: Vec2) -> f32 {
    let i = st.floor();
    let f = st - i;
    let cell = |offset: Vec2| random((i + offset).rem_euclid(Vec2::splat(FOAM_CELLS as f32)));
    let (a, b, c, d) = (
        cell(Vec2::ZERO),
        cell(Vec2::X),
        cell(Vec2::Y),
        cell(Vec2::ONE),
    );
    let u = f * f * (3.0 - 2.0 * f);
    a.lerp(b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y
}
