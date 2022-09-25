use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{controls::Controlled, game, level::load_level};

#[derive(Bundle)]
pub struct WallBundle {
    #[bundle]
    pbr_bundle: PbrBundle,
    collider: Collider,
    wall: Wall,
}

impl WallBundle {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        transform: Transform,
        size: Vec2,
        color: Color,
    ) -> Self {
        let width = size.x;
        let height = 0.1;
        let depth = size.y;

        Self {
            pbr_bundle: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(width, height, depth))),
                material: materials.add(color.into()),
                transform,
                ..default()
            },
            collider: Collider::cuboid(width / 2.0, height / 2.0, depth / 2.0),
            wall: Wall,
        }
    }
}

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Avoid;

#[derive(Component)]
pub struct Goal;

pub enum PlayerHit {
    Avoid,
    Goal,
}

fn handle_player_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    player_query: Query<&Controlled>,
    avoid_query: Query<&Avoid>,
    goal_query: Query<&Goal>,
    mut player_hit: EventWriter<PlayerHit>,
) {
    for event in collision_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            let target = if player_query.contains(*entity1) {
                Some(*entity2)
            } else if player_query.contains(*entity2) {
                Some(*entity1)
            } else {
                None
            };

            if let Some(target) = target {
                let hit = if avoid_query.contains(target) {
                    Some(PlayerHit::Avoid)
                } else if goal_query.contains(target) {
                    Some(PlayerHit::Goal)
                } else {
                    None
                };

                if let Some(event) = hit {
                    player_hit.send(event);
                }
            }
        }
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    load_level(
        &mut commands,
        &mut meshes,
        &mut materials,
        //&game::levels::level_1(),
        &game::levels::tutorial_1(),
    );

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.1,
                sectors: 20,
                stacks: 20,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: Color::rgba_linear(100.0, 100.0, 100.0, 0.0),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, game::levels::WORLD_BOX_SIZE / 2.0, 0.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(PointLightBundle {
                point_light: PointLight {
                    intensity: 2000.0,
                    radius: 0.1,
                    shadows_enabled: true,
                    ..default()
                },
                ..default()
            });
        });

    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 3.5)),
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            shadow_projection: OrthographicProjection {
                left: -10.0,
                right: 10.0,
                bottom: -10.0,
                top: 10.0,
                near: -10.0,
                far: 10.0,
                ..default()
            },
            ..default()
        },
        ..default()
    });
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_event::<PlayerHit>()
            .add_system(handle_player_collisions);
    }
}
