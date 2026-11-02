// an idea shamelessly stolen from https://www.youtube.com/watch?v=d6tp43wZqps
// prevents distortion of pixel art on moving sprites

use bevy::{
    asset::{RenderAssetUsages, embedded_asset, load_embedded_asset},
    camera::primitives::Aabb,
    camera::visibility::NoAutoAabb,
    light::NotShadowReceiver,
    mesh::{Indices, PrimitiveTopology},
    pbr::{ExtendedMaterial, MaterialExtension, check_entities_needing_specialization},
    platform::collections::HashMap,
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::{Shader, ShaderRef},
};

pub struct PixelSpritePlugin;

impl Plugin for PixelSpritePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "pixel_sprite.wesl");
        embedded_asset!(app, "pixel_sprite_prepass.wesl");
        embedded_asset!(app, "billboard.wesl");
        let billboard = load_embedded_asset!(app, "billboard.wesl");
        app.insert_resource(ShaderLibrary(billboard))
            .add_plugins(MaterialPlugin::<PixelSpriteMaterial>::default())
            .init_resource::<PixelSpriteCache>()
            .add_systems(
                PostUpdate,
                (mark_stale_sprites, sync_pixel_sprites)
                    .chain()
                    .before(check_entities_needing_specialization::<PixelSpriteMaterial>),
            );
    }
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default)]
#[require(NotShadowReceiver)]
pub struct PixelSprite {
    pub image: Handle<Image>,
    // which part of the image (like a spritesheet) to draw
    pub rect: URect,
    pub pixels_per_meter: f32,
    pub flip_x: bool,
    pub anchor: Vec2,
}

impl Default for PixelSprite {
    fn default() -> Self {
        Self {
            image: Handle::default(),
            rect: URect::default(),
            pixels_per_meter: 32.0,
            flip_x: false,
            anchor: Vec2::new(0.5, 0.0),
        }
    }
}

pub type PixelSpriteMaterial = ExtendedMaterial<StandardMaterial, PixelSpriteExtension>;

#[derive(Asset, AsBindGroup, Reflect, Clone)]
pub struct PixelSpriteExtension {
    #[texture(100)]
    image: Handle<Image>,
}

impl MaterialExtension for PixelSpriteExtension {
    fn vertex_shader() -> ShaderRef {
        "embedded://pixel_sprite/pixel_sprite.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://pixel_sprite/pixel_sprite.wesl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "embedded://pixel_sprite/pixel_sprite_prepass.wesl".into()
    }
}

#[derive(Resource)]
struct ShaderLibrary(
    #[expect(dead_code, reason = "this keeps the shader loaded")] Handle<Shader>,
);

#[derive(Resource, Default)]
struct PixelSpriteCache {
    materials: HashMap<AssetId<Image>, Handle<PixelSpriteMaterial>>,
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

// unstale the sprites
fn sync_pixel_sprites(
    mut commands: Commands,
    sprites: Query<
        (Entity, &PixelSprite),
        Or<(Changed<PixelSprite>, With<Stale>, Without<Mesh3d>)>,
    >,
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
            .entry(image)
            .or_insert_with(|| {
                materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        base_color_texture: Some(sprite.image.clone()),
                        alpha_mode: AlphaMode::Mask(0.5),
                        ..default()
                    },
                    extension: PixelSpriteExtension {
                        image: sprite.image.clone(),
                    },
                })
            })
            .clone();
        let rect = if sprite.rect.is_empty() {
            URect::from_corners(UVec2::ZERO, size)
        } else {
            sprite.rect
        };
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
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, corners.to_vec())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs.to_vec())
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 3, 2]))
}
