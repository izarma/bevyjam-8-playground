use bevy::prelude::*;
use jackdaw_surface::EnvironmentOptOut;

use crate::camera::SceneCamera;

pub(crate) const RENDER_SCALES: [u32; 3] = [50, 75, 100];

#[derive(Resource, Clone, Copy, Debug)]
pub(crate) struct GraphicsSettings {
    pub render_scale: u32,
    pub depth_of_field: bool,
    pub fog: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            render_scale: 50,
            depth_of_field: true,
            fog: true,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GraphicsSettings>()
        .add_systems(Update, apply_fog);
}

fn apply_fog(
    mut commands: Commands,
    settings: Res<GraphicsSettings>,
    cameras: Query<(Entity, Option<&EnvironmentOptOut>), SceneCamera>,
) {
    for (camera, opt_out) in &cameras {
        let current = opt_out.copied().unwrap_or_default();
        if current.fog != !settings.fog || opt_out.is_none() {
            commands.entity(camera).insert(EnvironmentOptOut {
                fog: !settings.fog,
                ..current
            });
        }
    }
}
