use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::controls::Controlled;

#[derive(Bundle)]
pub struct WallBundle {
    #[bundle]
    pbr_bundle: PbrBundle,
    rigid_body: RigidBody,
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
            rigid_body: RigidBody::Fixed,
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
            let entities = if player_query.contains(*entity1) {
                Some((*entity1, *entity2))
            } else if player_query.contains(*entity2) {
                Some((*entity2, *entity1))
            } else {
                None
            };

            if let Some((player, target)) = entities {
                let hit = if avoid_query.contains(target) {
                    debug!("player {:?} hit avoid {:?}", player, target);
                    Some(PlayerHit::Avoid)
                } else if goal_query.contains(target) {
                    debug!("player {:?} hit goal {:?}", player, target);
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

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerHit>()
            .add_system(handle_player_collisions);
    }
}
