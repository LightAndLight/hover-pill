use bevy::{
    prelude::{Mesh, Quat, Vec3},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

/// A cone with its base in the `XY` plane and tip pointing in the `+Z` direction.
pub struct Cone {
    /// The radius of the cone's base.
    pub radius: f32,

    /**
    The number of vertices that make up the cone's base.

    Must be greater than 2.
    */
    pub vertices: u32,

    /// The cone's height.
    pub height: f32,
}

impl Default for Cone {
    fn default() -> Self {
        Self {
            radius: 1.0,
            vertices: 64,
            height: 1.0,
        }
    }
}

impl From<Cone> for Mesh {
    fn from(cone: Cone) -> Self {
        debug_assert!(
            cone.vertices > 2,
            "a cone's base requires at least 3 vertices."
        );

        let num_vertices = cone.vertices + 2;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices as usize);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices as usize);
        for i in 0..cone.vertices {
            let angle = ((i as f32) / (cone.vertices as f32)) * std::f32::consts::TAU;
            positions.push([cone.radius * angle.cos(), cone.radius * angle.sin(), 0.0]);
            // TODO: more intuitive normals - bisect the angle between the base and the "roof"
            normals.push((Quat::from_rotation_z(angle) * Vec3::new(1.0, 0.0, 0.0)).into());
        }

        let origin_index = {
            let index = positions.len();
            positions.push([0.0, 0.0, 0.0]);
            normals.push([0.0, 0.0, -1.0]);
            index as u32
        };

        let tip_index = {
            let index = positions.len();
            positions.push([0.0, 0.0, cone.height]);
            normals.push([0.0, 0.0, 1.0]);
            index as u32
        };

        let num_triangles = cone.vertices + cone.vertices;
        let mut indices: Vec<u32> = Vec::with_capacity(3 * num_triangles as usize);
        for i in 0..cone.vertices {
            // the cone is pointing in +Z, so the base triangles
            // are pointing away from the camera (back). Assuming CCW front-facing,
            // we need to push the indices in CW order to make them face
            // backward.
            indices.push(origin_index);
            indices.push(i);
            indices.push(if i == 0 { cone.vertices - 1 } else { i - 1 });
        }
        for i in 0..cone.vertices {
            // the triangles on the "roof" of the cone are front-facing.
            // push indices in CCW order.
            indices.push(tip_index);
            indices.push(if i == 0 { cone.vertices - 1 } else { i - 1 });
            indices.push(i);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh
    }
}
