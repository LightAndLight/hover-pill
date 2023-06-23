use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum WallType {
    Neutral,
    Avoid,
    Goal,
}

#[derive(Component)]
pub struct Wall {
    pub wall_type: WallType,
}

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
        wall_type: WallType,
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
            wall: Wall { wall_type },
        }
    }

    pub fn goal(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        rotation: Quat,
        size: Vec2,
    ) -> Self {
        WallBundle::new(
            meshes,
            materials,
            WallType::Goal,
            Transform::IDENTITY
                .with_translation(position)
                .with_rotation(rotation),
            size,
            Color::GREEN,
        )
    }

    pub fn avoid(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        rotation: Quat,
        size: Vec2,
    ) -> Self {
        WallBundle::new(
            meshes,
            materials,
            WallType::Avoid,
            Transform::IDENTITY
                .with_translation(position)
                .with_rotation(rotation),
            size,
            Color::RED,
        )
    }

    pub fn neutral(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        rotation: Quat,
        size: Vec2,
    ) -> Self {
        WallBundle::new(
            meshes,
            materials,
            WallType::Neutral,
            Transform::IDENTITY
                .with_translation(position)
                .with_rotation(rotation),
            size,
            Color::WHITE,
        )
    }
}
