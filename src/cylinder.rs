use bevy::{
    prelude::{Mesh, Quat, Vec3},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

/// A cylinder that points along the `Z` axis.
pub struct Cylinder {
    /// The radius of the cylinder base.
    pub radius: f32,

    /**
    The number of vertices that make up the cylinder's base.

    Must be greater than 2.
    */
    pub vertices: u32,

    /// The cylinder's length.
    pub length: f32,
}

impl Default for Cylinder {
    fn default() -> Self {
        Self {
            radius: 1.0,
            vertices: 64,
            length: 1.0,
        }
    }
}

impl From<Cylinder> for Mesh {
    fn from(cylinder: Cylinder) -> Self {
        debug_assert!(
            cylinder.vertices > 2,
            "a cylinder's base requires at least 3 vertices."
        );

        let num_vertices = cylinder.vertices + cylinder.vertices + 2;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices as usize);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices as usize);

        // bottom circle
        for i in 0..cylinder.vertices {
            let angle = ((i as f32) / (cylinder.vertices as f32)) * std::f32::consts::TAU;
            positions.push([
                cylinder.radius * angle.cos(),
                cylinder.radius * angle.sin(),
                -cylinder.length / 2.0,
            ]);
            // TODO: more intuitive normals - bisect the angle between the base and the "roof"
            normals.push((Quat::from_rotation_z(angle) * Vec3::new(1.0, 0.0, 0.0)).into());
        }

        // top circle
        for i in 0..cylinder.vertices {
            let angle = ((i as f32) / (cylinder.vertices as f32)) * std::f32::consts::TAU;
            positions.push([
                cylinder.radius * angle.cos(),
                cylinder.radius * angle.sin(),
                cylinder.length / 2.0,
            ]);
            // TODO: more intuitive normals - bisect the angle between the base and the "roof"
            normals.push((Quat::from_rotation_z(angle) * Vec3::new(1.0, 0.0, 0.0)).into());
        }

        let bottom_center_index = {
            let index = positions.len();
            positions.push([0.0, 0.0, -cylinder.length / 2.0]);
            normals.push([0.0, 0.0, -1.0]);
            index as u32
        };

        let top_center_index = {
            let index = positions.len();
            positions.push([0.0, 0.0, cylinder.length / 2.0]);
            normals.push([0.0, 0.0, 1.0]);
            index as u32
        };

        // bottom circle + top circle + sides
        let num_triangles = cylinder.vertices + cylinder.vertices + 2 * cylinder.vertices;

        let mut indices: Vec<u32> = Vec::with_capacity(3 * num_triangles as usize);

        // bottom circle
        for i in 0..cylinder.vertices {
            indices.push(bottom_center_index);
            indices.push(i);
            indices.push(if i == 0 { cylinder.vertices - 1 } else { i - 1 });
        }

        // top circle
        for i in 0..cylinder.vertices {
            indices.push(top_center_index);
            indices.push(cylinder.vertices + i - 1);
            indices.push(cylinder.vertices + i);
        }

        // sides
        for i in 0..cylinder.vertices {
            indices.push(cylinder.vertices + i);
            indices.push(cylinder.vertices + i - 1);
            indices.push(i);

            indices.push(cylinder.vertices + i - 1);
            indices.push(if i == 0 { cylinder.vertices - 1 } else { i - 1 });
            indices.push(i);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh
    }
}
