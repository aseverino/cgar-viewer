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

use bevy::asset::RenderAssetUsages;
use bevy::math::Vec3;
use bevy::render::mesh::{Indices, Mesh};

use cgar::mesh::basic_types::Mesh as CgarMesh;
use cgar::numeric::scalar::Scalar as CgarScalar;

// ---- Example: convert a CGAR mesh (3D) to a Bevy Mesh ----
// Adapt trait bounds to your Scalar setup. We’ll cast to f32 for GPU.
pub fn cgar_to_bevy_mesh<T: CgarScalar>(m: &CgarMesh<T, 3>) -> Mesh
where
    for<'a> &'a T: Add<&'a T, Output = T>
        + Sub<&'a T, Output = T>
        + Mul<&'a T, Output = T>
        + Div<&'a T, Output = T>
        + Neg<Output = T>,
{
    // 1) Positions
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(m.vertices.len());
    for v in &m.vertices {
        // Adjust to your actual struct accessors:
        let p: [f32; 3] = [
            (v.position.coords[0].clone().into().0) as f32,
            (v.position.coords[1].clone().into().0) as f32,
            (v.position.coords[2].clone().into().0) as f32,
        ];
        positions.push(p);
    }

    // 2) Indices
    // Replace with your face loop; assume triangles:
    let mut indices: Vec<u32> = Vec::with_capacity(m.faces.len() * 3);
    for (fi, f) in m.faces.iter().enumerate() {
        if f.removed {
            continue;
        }
        // If you store half-edges, fetch the three vertex ids:
        let [i0, i1, i2] = tri_vertices_of_face(m, fi); // implement below
        indices.extend_from_slice(&[i0 as u32, i1 as u32, i2 as u32]);
    }

    // 3) Normals (vertex-averaged)
    let mut normals = vec![[0.0f32; 3]];
    normals.resize(positions.len(), [0.0; 3]);

    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = Vec3::from(positions[a]);
        let pb = Vec3::from(positions[b]);
        let pc = Vec3::from(positions[c]);
        let n = (pb - pa).cross(pc - pa);
        let n_arr = [n.x, n.y, n.z];
        for &i in &[a, b, c] {
            normals[i][0] += n_arr[0];
            normals[i][1] += n_arr[1];
            normals[i][2] += n_arr[2];
        }
    }
    for n in &mut normals {
        let v = Vec3::from(*n);
        let vn = v.length();
        if vn > 1e-20 {
            let u = v / vn;
            *n = [u.x, u.y, u.z];
        } else {
            *n = [0.0, 1.0, 0.0];
        }
    }
    // 4) Build bevy::Mesh
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::all(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// Stub: fetch triangle’s vertex indices from your half-edge structure
fn tri_vertices_of_face<T: CgarScalar>(m: &CgarMesh<T, 3>, face_idx: usize) -> [usize; 3]
where
    for<'a> &'a T: Add<&'a T, Output = T>
        + Sub<&'a T, Output = T>
        + Mul<&'a T, Output = T>
        + Div<&'a T, Output = T>
        + Neg<Output = T>,
{
    let hes = m.face_half_edges(face_idx);
    let v0 = m.half_edges[hes[0]].vertex;
    let v1 = m.half_edges[hes[1]].vertex;
    let v2 = m.half_edges[hes[2]].vertex;
    [v0, v1, v2]
}
