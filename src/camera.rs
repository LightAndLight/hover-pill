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

pub fn setup(parent: &mut ChildBuilder) {
    let camera_looking_at = Vec3::new(0.0, 1.0, 0.0);
    let transform = Transform::from_xyz(-5.0, 4.0, 0.0).looking_at(camera_looking_at, Vec3::Y);
    parent
        .spawn_bundle(Camera3dBundle {
            projection: PerspectiveProjection {
                fov: (60.0 / 360.0) * 2.0 * PI,
                ..default()
            }
            .into(),
            transform,
            ..default()
        })
        .insert(Camera)
        .insert(AtmosphereCamera(None))
        .insert(Zoom);
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
