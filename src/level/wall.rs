use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct Dimensions {
    value: Vec3,
    previous: Vec3,
}

impl Dimensions {
    pub fn set(&mut self, value: Vec3) {
        self.previous = self.value;
        self.value = value;
    }

    pub fn modify(&mut self, f: impl Fn(Vec3) -> Vec3) {
        self.previous = self.value;
        self.value = f(self.value);
    }
}

fn sync_dimensions(
    mut query: Query<(&mut Transform, &mut Dimensions), (With<Wall>, Changed<Dimensions>)>,
) {
    for (mut transform, dimensions) in &mut query {
        let dimensions_change = dimensions.value - dimensions.previous;
        if dimensions_change != Vec3::ZERO {
            transform.scale += dimensions_change / dimensions.previous;
            transform.translation += dimensions_change / 2.0;
        }
    }
}

pub struct DimensionsPlugin;

impl Plugin for DimensionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(sync_dimensions);
    }
}

#[derive(Bundle)]
pub struct WallBundle {
    #[bundle]
    pbr_bundle: PbrBundle,
    rigid_body: RigidBody,
    collider: Collider,
    wall: Wall,
    dimensions: Dimensions,
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
            dimensions: Dimensions {
                value: Vec3 {
                    x: width,
                    y: height,
                    z: depth,
                },
                previous: Vec3 {
                    x: width,
                    y: height,
                    z: depth,
                },
            },
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
