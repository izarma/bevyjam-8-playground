// an idea shamelessly stolen from https://www.youtube.com/watch?v=d6tp43wZqps
// prevents distortion of pixel art on moving sprites

use bevy::{
    asset::{RenderAssetUsages, embedded_asset},
    camera::primitives::Aabb,
    camera::visibility::NoAutoAabb,
    mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology},
    pbr::{
        ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
        check_entities_needing_specialization,
    },
    platform::collections::HashMap,
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

pub(crate) fn plugin(app: &mut App) {
    embedded_asset!(app, "sprite.wesl");
    embedded_asset!(app, "sprite_prepass.wesl");
    app.add_plugins(MaterialPlugin::<PixelSpriteMaterial>::default())
        .init_resource::<PixelSpriteCache>()
        .add_systems(
            PostUpdate,
            (mark_stale_sprites, sync_pixel_sprites)
                .chain()
                .before(check_entities_needing_specialization::<PixelSpriteMaterial>),
        );
}

#[derive(Component, Reflect, Clone, Debug, FromTemplate)]
#[reflect(Component, Default)]
pub struct PixelSprite {
    pub image: Handle<Image>,
    // the size in pixels of each sprite in the sheet
    pub size: Option<UVec2>,
    // which linear position in the sheet to draw (like if you collapsed the 2d array)
    pub cell: u32,
    pub pixels_per_meter: f32,
    pub flip_x: bool,
    pub anchor: Vec2,

    pub unlit: bool,
    
    pub billboard: bool,
}

impl Default for PixelSprite {
    fn default() -> Self {
        Self {
            image: Handle::default(),
            size: None,
            cell: 0,
            pixels_per_meter: crate::TEXELS_PER_METER,
            flip_x: false,
            anchor: Vec2::new(0.5, 0.0),
            unlit: false,
            billboard: true,
        }
    }
}

impl PixelSprite {
    pub fn rect(&self, image_size: UVec2) -> URect {
        let size = self.size.unwrap_or(image_size).max(UVec2::ONE);
        let columns = (image_size.x / size.x).max(1);
        let min = UVec2::new(self.cell % columns, self.cell / columns) * size;
        URect::from_corners(min, min + size)
    }
}

pub type PixelSpriteMaterial = ExtendedMaterial<StandardMaterial, PixelSpriteExtension>;

#[derive(Asset, AsBindGroup, Reflect, Clone)]
#[bind_group_data(PixelSpriteKey)]
pub struct PixelSpriteExtension {
    #[texture(100)]
    image: Handle<Image>,
    billboard: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixelSpriteKey {
    billboard: bool,
}

impl From<&PixelSpriteExtension> for PixelSpriteKey {
    fn from(extension: &PixelSpriteExtension) -> Self {
        Self {
            billboard: extension.billboard,
        }
    }
}

impl MaterialExtension for PixelSpriteExtension {
    fn vertex_shader() -> ShaderRef {
        "embedded://pixel_art/sprite.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://pixel_art/sprite.wesl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "embedded://pixel_art/sprite_prepass.wesl".into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.bind_group_data.billboard {
            descriptor.vertex.shader_defs.push("BILLBOARD".into());
            if let Some(fragment) = &mut descriptor.fragment {
                fragment.shader_defs.push("BILLBOARD".into());
            }
        }
        Ok(())
    }
}

#[derive(Resource, Default)]
struct PixelSpriteCache {
    // keyed by image, unlit and billboard
    materials: HashMap<(AssetId<Image>, bool, bool), Handle<PixelSpriteMaterial>>,
    quads: HashMap<(AssetId<Image>, QuadShape), Handle<Mesh>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct QuadShape {
    rect: URect,
    pixels_per_meter: u32,
    flip_x: bool,
    anchor: [u32; 2],
}

impl QuadShape {
    fn of(sprite: &PixelSprite, rect: URect) -> Self {
        Self {
            rect,
            pixels_per_meter: sprite.pixels_per_meter.to_bits(),
            flip_x: sprite.flip_x,
            anchor: [sprite.anchor.x.to_bits(), sprite.anchor.y.to_bits()],
        }
    }
}

#[derive(Component)]
struct Stale;

// if the properties of a sprite asset changed we want to reload them
// but might have to do something async like get an image asset
// so it could be stale for sseveral frames
fn mark_stale_sprites(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<Image>>,
    mut cache: ResMut<PixelSpriteCache>,
    sprites: Query<(Entity, &PixelSprite)>,
) {
    for event in events.read() {
        let AssetEvent::Modified { id } = event else {
            continue;
        };
        cache.quads.retain(|(image, ..), _| image != id);
        for (entity, sprite) in &sprites {
            if sprite.image.id() == *id {
                commands.entity(entity).insert(Stale);
            }
        }
    }
}

// sprites whose mesh and material have to be (re)built
type NeedsSync = Or<(Changed<PixelSprite>, With<Stale>, Without<Mesh3d>)>;

// unstale the sprites
fn sync_pixel_sprites(
    mut commands: Commands,
    sprites: Query<(Entity, &PixelSprite), NeedsSync>,
    mut cache: ResMut<PixelSpriteCache>,
    images: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PixelSpriteMaterial>>,
) {
    for (entity, sprite) in &sprites {
        let Some(size) = images.get(&sprite.image).map(Image::size) else {
            commands.entity(entity).insert(Stale);
            continue;
        };
        let image = sprite.image.id();
        let material = cache
            .materials
            .entry((image, sprite.unlit, sprite.billboard))
            .or_insert_with(|| {
                materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        base_color_texture: Some(sprite.image.clone()),
                        alpha_mode: AlphaMode::Mask(0.5),
                        unlit: sprite.unlit,
                        ..default()
                    },
                    extension: PixelSpriteExtension {
                        image: sprite.image.clone(),
                        billboard: sprite.billboard,
                    },
                })
            })
            .clone();
        let rect = sprite.rect(size);
        let mesh = cache
            .quads
            .entry((image, QuadShape::of(sprite, rect)))
            .or_insert_with(|| meshes.add(quad(sprite, rect, size)))
            .clone();
        let (low, high) = corners(sprite, rect);
        let reach = Vec3::splat(low.abs().max(high.abs()).length());
        let bounds = Aabb::from_min_max(-reach, reach);
        commands
            .entity(entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material), bounds, NoAutoAabb))
            .remove::<Stale>();
    }
}

// returns the quad's bottom-left and top-right corners in meters
fn corners(sprite: &PixelSprite, rect: URect) -> (Vec2, Vec2) {
    let extent = rect.size().as_vec2() / sprite.pixels_per_meter;
    let low = -sprite.anchor * extent;
    (low, low + extent)
}

// returns a quad sized to `sprite`
fn quad(sprite: &PixelSprite, rect: URect, size: UVec2) -> Mesh {
    let (low, high) = corners(sprite, rect);
    let uv_min = rect.min.as_vec2() / size.as_vec2();
    let uv_max = rect.max.as_vec2() / size.as_vec2();
    let (left, right) = if sprite.flip_x {
        (uv_max.x, uv_min.x)
    } else {
        (uv_min.x, uv_max.x)
    };
    let corners = [
        [low.x, low.y, 0.0],
        [high.x, low.y, 0.0],
        [low.x, high.y, 0.0],
        [high.x, high.y, 0.0],
    ];
    let uvs = [
        [left, uv_max.y],
        [right, uv_max.y],
        [left, uv_min.y],
        [right, uv_min.y],
    ];
    let opposite_uvs = [uvs[3], uvs[2], uvs[1], uvs[0]];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, corners.to_vec())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs.to_vec())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, opposite_uvs.to_vec())
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 3, 2]))
}
