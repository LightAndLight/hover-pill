use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};
use bevy_atmosphere::prelude::AtmosphereCamera;

#[derive(Component)]
pub struct Camera;

#[derive(Component)]
pub struct Zoom;

#[derive(Bundle)]
pub struct CameraBundle {
    #[bundle]
    camera3d_bundle: Camera3dBundle,
    atmosphere_camera: AtmosphereCamera,
    zoom: Zoom,
    camera: Camera,
}

impl CameraBundle {
    pub fn new(transform: Transform) -> Self {
        Self {
            camera3d_bundle: Camera3dBundle {
                projection: PerspectiveProjection {
                    fov: (60.0 / 360.0) * 2.0 * PI,
                    ..default()
                }
                .into(),
                transform: transform.looking_at(Vec3::Y, Vec3::Y),
                ..default()
            },
            atmosphere_camera: AtmosphereCamera(None),
            zoom: Zoom,
            camera: Camera,
        }
    }
}

pub fn scroll_zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Zoom>>,
) {
    let scroll_amount: f32 = scroll_events
        .iter()
        .map(|scroll_event| match scroll_event.unit {
            MouseScrollUnit::Line => scroll_event.y,
            unit => {
                warn!("unsupported scroll unit: {:?}", unit);
                0.0
            }
        })
        .sum();

    for mut transform in query.iter_mut() {
        let translation = transform.translation;
        transform.translation += 0.08 * scroll_amount * -translation;
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(scroll_zoom);
    }
}
