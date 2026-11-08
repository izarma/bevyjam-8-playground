use bevy::{
    asset::{RenderAssetUsages, embedded_asset},
    camera::visibility::NoFrustumCulling,
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

pub(crate) fn plugin(app: &mut App) {
    embedded_asset!(app, "sky.wesl");
    app.add_plugins(MaterialPlugin::<PixelSkyMaterial>::default())
        .add_systems(Update, dress_skies);
}

#[derive(Component, Reflect, Clone, PartialEq)]
#[reflect(Component, Default)]
pub struct PixelSky {
    pub zenith: Color,
    pub middle: Color,
    pub horizon: Color,
    pub ground: Color,

    pub texels_per_degree: f32,
    pub bands: u32,
    // distance gradient bands dither together
    pub dither: f32,
    // distance from the horizon dither is allowed
    pub horizon_dither: f32,

    pub horizon_reach: f32,
    pub sun_size: f32,
    pub sun_color: Color,
    pub moon_color: Color,

    pub glow_rings: u32,
    pub glow_reach: f32,
    pub glow_color: Color,
    pub glow_strength: f32,

    pub sunset: f32,
    pub sunset_color: Color,

    pub stars: f32,
}

// the sandbox's "soft pixel": a smooth gradient dithered only near the horizon.
// its "chunky smooth" is texels_per_degree 1.5, dither 0, glow_rings 0. smaller sun
impl Default for PixelSky {
    fn default() -> Self {
        Self {
            zenith: Color::srgb(0.215, 0.429, 0.642),
            middle: Color::srgb(0.442, 0.644, 0.79),
            horizon: Color::srgb(0.825, 0.831, 0.762),
            ground: Color::srgb(0.6, 0.623, 0.572),
            texels_per_degree: 3.0,
            bands: 48,
            dither: 1.0,
            horizon_dither: 6.0,
            horizon_reach: 0.4,
            sun_size: 3.5,
            sun_color: Color::srgb(1.0, 0.975, 0.893),
            moon_color: Color::srgb(0.91, 0.9, 0.83),
            glow_rings: 2,
            glow_reach: 10.0,
            glow_color: Color::srgb(0.951, 0.925, 0.813),
            glow_strength: 0.7,
            sunset: 0.9,
            sunset_color: Color::srgb(1.0, 0.91, 0.784),
            stars: 0.0,
        }
    }
}

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct PixelSkyMaterial {
    #[uniform(0)]
    settings: PixelSkySettings,
}

#[derive(ShaderType, Clone)]
struct PixelSkySettings {
    zenith: Vec4,
    middle: Vec4,
    horizon: Vec4,
    ground: Vec4,
    sun_color: Vec4,
    moon_color: Vec4,
    glow_color: Vec4,
    sunset_color: Vec4,
    texels_per_degree: f32,
    bands: f32,
    dither: f32,
    horizon_dither: f32,
    horizon_reach: f32,
    sun_size: f32,
    glow_rings: f32,
    glow_reach: f32,
    glow_strength: f32,
    sunset: f32,
    stars: f32,
}

impl PixelSky {
    fn material(&self) -> PixelSkyMaterial {
        let linear = |color: Color| Vec4::from_array(color.to_linear().to_f32_array());
        PixelSkyMaterial {
            settings: PixelSkySettings {
                zenith: linear(self.zenith),
                middle: linear(self.middle),
                horizon: linear(self.horizon),
                ground: linear(self.ground),
                sun_color: linear(self.sun_color),
                moon_color: linear(self.moon_color),
                glow_color: linear(self.glow_color),
                sunset_color: linear(self.sunset_color),
                texels_per_degree: self.texels_per_degree.max(0.1),
                bands: self.bands.max(2) as f32,
                dither: self.dither,
                horizon_dither: self.horizon_dither,
                horizon_reach: self.horizon_reach,
                sun_size: self.sun_size,
                glow_rings: self.glow_rings as f32,
                glow_reach: self.glow_reach.max(0.1),
                glow_strength: self.glow_strength,
                sunset: self.sunset,
                stars: self.stars,
            },
        }
    }
}

impl Material for PixelSkyMaterial {
    fn vertex_shader() -> ShaderRef {
        "embedded://pixel_art/sky.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://pixel_art/sky.wesl".into()
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

#[derive(Component)]
struct SkyMaterial(Handle<PixelSkyMaterial>);

// a sky entity draws as one triangle over the whole screen, at the far plane
fn dress_skies(
    mut commands: Commands,
    skies: Query<(Entity, Ref<PixelSky>, Option<&SkyMaterial>)>,
    mut screen: Local<Option<Handle<Mesh>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PixelSkyMaterial>>,
) {
    for (entity, sky, worn) in &skies {
        match worn {
            Some(SkyMaterial(handle)) => {
                if sky.is_changed() {
                    let _ = materials.insert(handle, sky.material());
                }
            }
            None => {
                let mesh = screen
                    .get_or_insert_with(|| meshes.add(screen_triangle()))
                    .clone();
                let handle = materials.add(sky.material());
                commands.entity(entity).insert((
                    SkyMaterial(handle.clone()),
                    Mesh3d(mesh),
                    MeshMaterial3d(handle),
                    NoFrustumCulling,
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
            }
        }
    }
}

fn screen_triangle() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]],
    )
}
