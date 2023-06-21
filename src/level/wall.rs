use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

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
pub enum WallType {
    Avoid,
    Goal,
}
