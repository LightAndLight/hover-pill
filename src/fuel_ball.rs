use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::fuel::{add_fuel, Fuel, FuelChanged};

#[derive(Component)]
pub struct FuelBall {
    pub amount: f32,
}

#[derive(Bundle)]
pub struct FuelBallBundle {
    #[bundle]
    pbr_bundle: PbrBundle,
    collider: Collider,
    active_events: ActiveEvents,
    sensor: Sensor,
    fuel_ball: FuelBall,
}

impl FuelBallBundle {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
    ) -> Self {
        Self {
            pbr_bundle: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: 0.25,
                    sectors: 4,
                    stacks: 3,
                })),
                material: materials.add(Color::rgb(0.4, 0.4, 1.).into()),
                transform: Transform::from_translation(position)
                    .with_rotation(Quat::from_rotation_x(PI / 2.)),
                ..default()
            },
            collider: Collider::ball(0.25),
            active_events: ActiveEvents::COLLISION_EVENTS,
            sensor: Sensor,
            fuel_ball: FuelBall { amount: 0.20 },
        }
    }
}

pub fn refuel(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut fuel_query: Query<&mut Fuel, Without<FuelBall>>,
    ball_query: Query<&FuelBall>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            let (fuel_entity, ball_entity) = if fuel_query.contains(*entity1) {
                (*entity1, *entity2)
            } else {
                (*entity2, *entity1)
            };

            if let (Ok(mut fuel), Ok(refuel_ball)) =
                (fuel_query.get_mut(fuel_entity), ball_query.get(ball_entity))
            {
                add_fuel(&mut fuel, refuel_ball.amount, &mut fuel_changed);
                commands.entity(ball_entity).despawn();
            }
        }
    }
}

pub fn rotate(time: Res<Time>, mut query: Query<&mut Transform, With<FuelBall>>) {
    let radians_per_second = 0.6;
    let delta_seconds = time.delta_seconds();
    for mut transform in &mut query {
        transform.rotate_y(radians_per_second * delta_seconds);
    }
}

pub struct FuelBallPlugin;

impl Plugin for FuelBallPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(refuel).add_system(rotate);
    }
}
