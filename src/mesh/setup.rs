// SPDX-License-Identifier: MIT
//
// Copyright (c) 2025 Alexandre Severino
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::ops::{Add, Div, Mul, Neg, Sub};

use bevy::{
    asset::Assets,
    color::Color,
    ecs::system::{Commands, ResMut},
    pbr::{MeshMaterial3d, StandardMaterial},
    picking::Pickable,
    render::mesh::{Mesh, Mesh3d},
    transform::components::Transform,
    utils::default,
};
use cgar::{
    geometry::spatial_element::SpatialElement, io::obj::read_obj, numeric::cgar_f64::CgarF64,
};

use crate::{camera::components::CgarMeshData, mesh::conversion::cgar_to_bevy_mesh};
use cgar::mesh::basic_types::Mesh as CgarMesh;

fn create_grid_mesh(grid_size: usize) -> CgarMesh<CgarF64, 3> {
    let mut mesh = CgarMesh::<CgarF64, 3>::new();

    // make grid_size x grid_size vertices
    let id = |x: usize, y: usize| -> usize { y * grid_size + x };
    for y in 0..grid_size {
        for x in 0..grid_size {
            mesh.add_vertex(cgar::geometry::Point3::from_vals([
                CgarF64::from(x as f64),
                CgarF64::from(y as f64),
                CgarF64::from(0.0),
            ]));
        }
    }

    // triangulate (grid_size-1) x (grid_size-1) quads
    for y in 0..(grid_size - 1) {
        for x in 0..(grid_size - 1) {
            let v00 = id(x, y);
            let v10 = id(x + 1, y);
            let v01 = id(x, y + 1);
            let v11 = id(x + 1, y + 1);

            mesh.add_triangle(v00, v10, v11);
            mesh.add_triangle(v00, v11, v01);
        }
    }

    mesh.validate_connectivity();
    mesh
}

pub fn setup_cgar_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) where
    for<'a> &'a CgarF64: Add<&'a CgarF64, Output = CgarF64>
        + Sub<&'a CgarF64, Output = CgarF64>
        + Mul<&'a CgarF64, Output = CgarF64>
        + Div<&'a CgarF64, Output = CgarF64>
        + Neg<Output = CgarF64>,
{
    // For now: create a simple cube as a placeholder
    // let cgar_mesh = read_obj::<CgarF64, _>("/mnt/v/cgar_meshes/cube.obj").unwrap(); // Replace with your actual CGAR mesh
    let cgar_mesh = create_grid_mesh(16);
    let bevy_mesh = cgar_to_bevy_mesh(&cgar_mesh);

    let handle = meshes.add(bevy_mesh);
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.95), // Brighter base color
        perceptual_roughness: 0.3,               // Lower roughness = more reflective
        metallic: 0.0, // Non-metallic for better visibility with ambient light
        emissive: Color::srgb(0.5, 0.5, 0.5).into(), // Add slight emission
        ..default()
    });

    commands.spawn((
        MeshMaterial3d(material),
        Mesh3d(handle.clone()),
        Transform::default(),
        Pickable::default(),
        CgarMeshData(cgar_mesh),
    ));
}
