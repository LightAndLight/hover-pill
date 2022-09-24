use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Component)]
pub struct Controlled {
    pub rotating: bool,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub hovering: bool,
}

#[derive(Component)]
pub struct Forward {
    pub value: Vec3,
}

#[derive(Component)]
pub struct Speed {
    pub value: f32,
}

pub fn move_controlled(
    time: Res<Time>,
    mut controlled_query: Query<(&Controlled, &Speed, &Forward, &mut Transform)>,
) {
    let delta_seconds = time.delta_seconds();

    for (controlled, speed, forward, mut transform) in controlled_query.iter_mut() {
        let mut movement = Vec3::ZERO;

        if controlled.forward {
            movement += delta_seconds * speed.value * forward.value;
        }

        if controlled.backward {
            movement += delta_seconds * speed.value * -forward.value;
        }

        let right = forward.value.cross(Vec3::Y).normalize();

        if controlled.left {
            movement += delta_seconds * speed.value * -right;
        }

        if controlled.right {
            movement += delta_seconds * speed.value * right;
        }

        transform.translation += movement;
    }
}

pub fn rotate_controlled(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&Controlled, &mut Forward, &mut Transform, &Children)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Controlled>)>,
) {
    let delta: Vec2 = mouse_motion_events
        .iter()
        .fold(Vec2::ZERO, |delta, mouse_motion_event| {
            delta + mouse_motion_event.delta
        });

    for (controlled, mut forward, mut transform, children) in query.iter_mut() {
        if controlled.rotating {
            let rotation = Quat::from_rotation_y(0.005 * -delta.x);
            forward.value = rotation * forward.value;
            transform.rotate(rotation);

            for child in children.iter() {
                if let Ok(mut camera_transform) = camera_query.get_mut(*child) {
                    let camera_rotation = Quat::from_axis_angle(Vec3::Z, 0.005 * -delta.y);

                    camera_transform.rotate_around(Vec3::ZERO, camera_rotation);
                }
            }
        }
    }
}
