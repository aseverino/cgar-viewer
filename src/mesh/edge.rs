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

use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::Query;
use bevy::math::{Vec2, Vec3, Vec3A};
use bevy::pbr::wireframe::NoWireframe;
use bevy::picking::events::{Click, Pressed, Released};
use bevy::picking::pointer::PointerId;
use bevy::render::camera::Camera;
use bevy::transform::components::GlobalTransform;
use bevy::window::{PrimaryWindow, Window};
use bevy::{
    asset::Assets,
    color::Color,
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        system::{Commands, ResMut},
    },
    input::{ButtonState, mouse::MouseButtonInput},
    pbr::{MeshMaterial3d, StandardMaterial},
    picking::{events::Pointer, pointer::PointerInteraction},
    render::mesh::{Mesh, Mesh3d, PrimitiveTopology},
    transform::components::Transform,
    utils::default,
};
use bevy_inspector_egui::egui::ahash::HashMap;
use cgar::geometry::spatial_element::SpatialElement;
use cgar::geometry::{Point3, Vector3};
use cgar::mesh::basic_types::{IntersectionHit, IntersectionResult, Mesh as CgarMesh};
use cgar::numeric::cgar_f64::CgarF64;

use crate::camera::components::CgarMeshData;

#[derive(Component)]
pub struct EdgeHighlight {
    pub original_entity: Entity,
}

#[derive(Resource, Default)]
pub struct HighlightedEdges {
    pub cylinders: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct PointerPresses {
    pub pos: HashMap<PointerId, Vec2>,
    pub target: HashMap<PointerId, Entity>,
}

pub fn handle_mesh_click(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut highlighted_edges: ResMut<HighlightedEdges>,
    mut press_events: EventReader<Pointer<Pressed>>,
    mut release_events: EventReader<Pointer<Released>>,
    mut presses: ResMut<PointerPresses>,
    mesh_query: Query<(&Mesh3d, &GlobalTransform, &CgarMeshData)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    for event in press_events.read() {
        presses
            .pos
            .insert(event.pointer_id, event.pointer_location.position);
        presses.target.insert(event.pointer_id, event.target);
    }

    let click_deadzone = 3.0;
    let deadzone_sq = click_deadzone * click_deadzone;

    for event in release_events.read() {
        let Some(start_pos) = presses.pos.remove(&event.pointer_id) else {
            continue;
        };
        let _ = presses.target.remove(&event.pointer_id);

        let end_pos = event.pointer_location.position;
        let moved_sq = (end_pos - start_pos).length_squared();

        let same_target = presses
            .target
            .get(&event.pointer_id)
            .map(|t| *t == event.target)
            .unwrap_or(true);

        if moved_sq > deadzone_sq || !same_target {
            // Treat as drag; do not click
            continue;
        }

        if let Ok((_, mesh_global, cgar_data)) = mesh_query.get(event.target) {
            clear_edge_highlights(&mut commands, &mut highlighted_edges);

            if let (Ok((camera, camera_transform)), Ok(window)) =
                (camera_query.get_single(), window_query.get_single())
            {
                // Start from the pointer's position (likely logical)
                let mut pos = event.pointer_location.position;

                // Convert to physical pixels
                pos *= window.resolution.scale_factor() as f32;

                // If the camera uses a viewport, make the pos relative to it
                if let Some(vp) = camera.viewport.as_ref() {
                    pos -= vp.physical_position.as_vec2();
                }

                if let Ok(ray) = camera.viewport_to_world(camera_transform, pos) {
                    let inv_affine = mesh_global.affine().inverse();

                    // Correct: use the ray's own origin and direction
                    let local_o = inv_affine.transform_point3a(ray.origin.into());
                    let local_dir_a = inv_affine
                        .transform_vector3a(ray.direction.as_vec3().into())
                        .normalize();

                    // Optional: two-point variant using the same ray origin
                    // let local_p1 = inv_affine.transform_point3a((ray.origin + ray.direction.as_vec3()).into());
                    // let local_dir_a = (local_p1 - local_o).normalize();

                    let local_origin = Point3::<CgarF64>::from_vals([
                        local_o.x as f64,
                        local_o.y as f64,
                        local_o.z as f64,
                    ]);
                    let local_direction = Vector3::<CgarF64>::from_vals([
                        local_dir_a.x as f64,
                        local_dir_a.y as f64,
                        local_dir_a.z as f64,
                    ]);

                    println!(
                        "Local origin: {:?}, Local dir: {:?}",
                        local_origin, local_direction
                    );

                    let cgar_mesh = &cgar_data.0;
                    let tree = cgar_mesh.build_face_tree();
                    let tolerance = CgarF64::from(0.05);

                    match cgar_mesh.cast_ray(
                        &local_origin,
                        &local_direction,
                        &tree,
                        &Some(tolerance),
                    ) {
                        IntersectionResult::Hit(hit, _distance) => match hit {
                            IntersectionHit::Edge(v0, v1, _) => {
                                highlight_cgar_edge(
                                    &mut commands,
                                    &mut meshes,
                                    &mut materials,
                                    &mut highlighted_edges,
                                    cgar_mesh,
                                    (v0, v1),
                                    mesh_global,
                                    event.target,
                                );
                            }
                            IntersectionHit::Face(face_id, _) => {
                                for edge_idx in cgar_mesh.face_half_edges(face_id).iter() {
                                    if let Some(he) = cgar_mesh.half_edges.get(*edge_idx) {
                                        let v0 = he.vertex;
                                        let v1 = cgar_mesh.half_edges[he.next].vertex;
                                        highlight_cgar_edge(
                                            &mut commands,
                                            &mut meshes,
                                            &mut materials,
                                            &mut highlighted_edges,
                                            cgar_mesh,
                                            (v0, v1),
                                            mesh_global,
                                            event.target,
                                        );
                                    }
                                }
                            }
                            _ => {}
                        },
                        IntersectionResult::Miss => {
                            println!("Ray missed the mesh");
                        }
                    }
                }
            }
        }
    }
}

// Simple slab test against [0,1]^3 in mesh-local space
fn ray_hits_unit_aabb(o: Vec3A, d: Vec3A) -> bool {
    let inv = Vec3A::new(
        if d.x != 0.0 { 1.0 / d.x } else { f32::INFINITY },
        if d.y != 0.0 { 1.0 / d.y } else { f32::INFINITY },
        if d.z != 0.0 { 1.0 / d.z } else { f32::INFINITY },
    );
    let mut tmin = ((0.0 - o.x) * inv.x).min((1.0 - o.x) * inv.x);
    let mut tmax = ((0.0 - o.x) * inv.x).max((1.0 - o.x) * inv.x);

    let tymin = ((0.0 - o.y) * inv.y).min((1.0 - o.y) * inv.y);
    let tymax = ((0.0 - o.y) * inv.y).max((1.0 - o.y) * inv.y);

    if (tmin > tymax) || (tymin > tmax) {
        return false;
    }
    tmin = tmin.max(tymin);
    tmax = tmax.min(tymax);

    let tzmin = ((0.0 - o.z) * inv.z).min((1.0 - o.z) * inv.z);
    let tzmax = ((0.0 - o.z) * inv.z).max((1.0 - o.z) * inv.z);

    if (tmin > tzmax) || (tzmin > tmax) {
        return false;
    }
    tmin = tmin.max(tzmin);
    tmax = tmax.min(tzmax);

    tmax >= 0.0 && tmax >= tmin
}

// pub fn handle_mesh_click_3(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut highlighted_edges: ResMut<HighlightedEdges>,
//     mut click_events: EventReader<Pointer<Click>>,
//     // NOTE: use GlobalTransform instead of local Transform
//     mesh_query: Query<(&Mesh3d, &GlobalTransform, &CgarMeshData)>,
//     camera_query: Query<(&Camera, &GlobalTransform)>,
//     window_query: Query<&Window, With<PrimaryWindow>>,
// ) {
//     for event in click_events.read() {
//         if let Ok((_, mesh_global, cgar_data)) = mesh_query.get(event.target) {
//             clear_edge_highlights(&mut commands, &mut highlighted_edges);

//             if let (Ok((camera, camera_transform)), Ok(window)) =
//                 (camera_query.get_single(), window_query.get_single())
//             {
//                 // Start from the pointer's position
//                 let mut pos = event.pointer_location.position;

//                 // Convert to physical pixels
//                 let scale = window.resolution.scale_factor() as f32;
//                 pos *= scale;

//                 // If the camera uses a viewport, make the pos relative to it
//                 if let Some(vp) = camera.viewport.as_ref() {
//                     pos -= vp.physical_position.as_vec2();
//                 }

//                 if let Ok(ray) = camera.viewport_to_world(camera_transform, pos) {
//                     // Transform ray to mesh local space
//                     let inv_affine = mesh_global.affine().inverse();
//                     let local_origin_a =
//                         inv_affine.transform_point3a(bevy::math::Vec3A::from(ray.origin));
//                     let local_dir_a = inv_affine
//                         .transform_vector3a(ray.direction.as_vec3().into())
//                         .normalize();

//                     let local_origin = Point3::<CgarF64>::from_vals([
//                         local_origin_a.x as f64,
//                         local_origin_a.y as f64,
//                         local_origin_a.z as f64,
//                     ]);
//                     let local_direction = Vector3::<CgarF64>::from_vals([
//                         local_dir_a.x as f64,
//                         local_dir_a.y as f64,
//                         local_dir_a.z as f64,
//                     ]);

//                     // Debug
//                     println!(
//                         "Local ray origin: {:?}, direction: {:?}",
//                         local_origin, local_direction
//                     );

//                     let cgar_mesh = &cgar_data.0;
//                     let tree = cgar_mesh.build_face_tree();

//                     let tolerance = CgarF64::from(0.1);
//                     let intersection = cgar_mesh.cast_ray(
//                         &local_origin,
//                         &local_direction,
//                         &tree,
//                         &Some(tolerance),
//                     );

//                     match intersection {
//                         IntersectionResult::Hit(hit, _distance) => {
//                             println!("Hit detected: {:?}", hit);
//                             match hit {
//                                 IntersectionHit::Edge(v0, v1, _) => {
//                                     highlight_cgar_edge(
//                                         &mut commands,
//                                         &mut meshes,
//                                         &mut materials,
//                                         &mut highlighted_edges,
//                                         cgar_mesh,
//                                         (v0, v1),
//                                         mesh_global,
//                                         event.target,
//                                     );
//                                 }
//                                 IntersectionHit::Face(face_id, _hit_point) => {
//                                     println!("Face hit at face {}", face_id);
//                                     if let Some(_face) = cgar_mesh.faces.get(face_id) {
//                                         for edge_idx in cgar_mesh.face_half_edges(face_id).iter() {
//                                             if let Some(half_edge) =
//                                                 cgar_mesh.half_edges.get(*edge_idx)
//                                             {
//                                                 // Use the next half-edge to get the edge’s second vertex
//                                                 let v0 = half_edge.vertex;
//                                                 let v1 =
//                                                     cgar_mesh.half_edges[half_edge.next].vertex;
//                                                 highlight_cgar_edge(
//                                                     &mut commands,
//                                                     &mut meshes,
//                                                     &mut materials,
//                                                     &mut highlighted_edges,
//                                                     cgar_mesh,
//                                                     (v0, v1),
//                                                     mesh_global,
//                                                     event.target,
//                                                 );
//                                             }
//                                         }
//                                     }
//                                 }
//                                 _ => {
//                                     println!("Other intersection type");
//                                 }
//                             }
//                         }
//                         IntersectionResult::Miss => {
//                             println!("Ray missed the mesh");
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// pub fn handle_mesh_click_2(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut highlighted_edges: ResMut<HighlightedEdges>,
//     mut click_events: EventReader<Pointer<Click>>,
//     mesh_query: Query<(&Mesh3d, &Transform, &CgarMeshData)>,
//     camera_query: Query<(&Camera, &GlobalTransform)>,
//     window_query: Query<&Window, With<PrimaryWindow>>,
// ) {
//     for event in click_events.read() {
//         if let Ok((mesh_handle, transform, cgar_data)) = mesh_query.get(event.target) {
//             // Clear previous highlights
//             clear_edge_highlights(&mut commands, &mut highlighted_edges);

//             // Get camera and window for ray casting
//             if let (Ok((camera, camera_transform)), Ok(window)) =
//                 (camera_query.get_single(), window_query.get_single())
//             {
//                 // Use the click position from the event, not current cursor position
//                 let click_pos = event.pointer_location.position;

//                 // Convert screen coordinates to world ray
//                 if let Ok(ray) = camera.viewport_to_world(camera_transform, click_pos) {
//                     // Debug: print the ray information
//                     println!(
//                         "Ray origin: {:?}, direction: {:?}",
//                         ray.origin, ray.direction
//                     );
//                     println!("Transform matrix: {:?}", transform.compute_matrix());

//                     // Transform ray to mesh local space
//                     let inverse_transform = transform.compute_matrix().inverse();
//                     let local_origin = inverse_transform.transform_point3(ray.origin);
//                     let local_direction = inverse_transform
//                         .transform_vector3(*ray.direction)
//                         .normalize();

//                     let local_ray_origin = Point3::<CgarF64>::from_vals([
//                         local_origin.x as f64,
//                         local_origin.y as f64,
//                         local_origin.z as f64,
//                     ]);
//                     let local_ray_direction = Vector3::<CgarF64>::from_vals([
//                         local_direction.x as f64,
//                         local_direction.y as f64,
//                         local_direction.z as f64,
//                     ]);

//                     // Debug: print local ray information
//                     println!(
//                         "Local ray origin: {:?}, direction: {:?}",
//                         local_ray_origin, local_ray_direction
//                     );

//                     // Create AABB tree for the mesh
//                     let cgar_mesh = &cgar_data.0;
//                     let tree = cgar_mesh.build_face_tree();

//                     // Cast ray to find intersection
//                     let tolerance = CgarF64::from(0.1); // Increased tolerance
//                     let intersection = cgar_mesh.cast_ray(
//                         &local_ray_origin,
//                         &local_ray_direction,
//                         &tree,
//                         &Some(tolerance),
//                     );

//                     // Handle intersection result
//                     match intersection {
//                         IntersectionResult::Hit(hit, distance) => {
//                             println!("Hit detected: {:?}", hit);
//                             match hit {
//                                 IntersectionHit::Edge(v0, v1, _) => {
//                                     highlight_cgar_edge(
//                                         &mut commands,
//                                         &mut meshes,
//                                         &mut materials,
//                                         &mut highlighted_edges,
//                                         cgar_mesh,
//                                         (v0, v1),
//                                         transform,
//                                         event.target,
//                                     );
//                                 }
//                                 IntersectionHit::Face(face_id, hit_point) => {
//                                     println!("Face hit at face {}, point {:?}", face_id, hit_point);
//                                     // For now, just highlight all edges of the face
//                                     if let Some(face) = cgar_mesh.faces.get(face_id) {
//                                         for edge_idx in cgar_mesh.face_half_edges(face_id).iter() {
//                                             if let Some(half_edge) =
//                                                 cgar_mesh.half_edges.get(*edge_idx)
//                                             {
//                                                 highlight_cgar_edge(
//                                                     &mut commands,
//                                                     &mut meshes,
//                                                     &mut materials,
//                                                     &mut highlighted_edges,
//                                                     cgar_mesh,
//                                                     (half_edge.vertex, half_edge.vertex),
//                                                     transform,
//                                                     event.target,
//                                                 );
//                                             }
//                                         }
//                                     }
//                                 }
//                                 _ => {
//                                     println!("Other intersection type");
//                                 }
//                             }
//                         }
//                         IntersectionResult::Miss => {
//                             println!("Ray missed the mesh");
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

fn extract_edges_from_mesh(mesh: &Mesh) -> Vec<(Vec3, Vec3)> {
    let mut edges = Vec::new();

    if let Some(vertices) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(indices) = mesh.indices() {
            let positions: Vec<Vec3> = vertices
                .as_float3()
                .unwrap()
                .iter()
                .map(|&pos| Vec3::from(pos))
                .collect();

            match indices {
                bevy::render::mesh::Indices::U16(indices) => {
                    for triangle in indices.chunks(3) {
                        let v0 = positions[triangle[0] as usize];
                        let v1 = positions[triangle[1] as usize];
                        let v2 = positions[triangle[2] as usize];

                        edges.push((v0, v1));
                        edges.push((v1, v2));
                        edges.push((v2, v0));
                    }
                }
                bevy::render::mesh::Indices::U32(indices) => {
                    for triangle in indices.chunks(3) {
                        let v0 = positions[triangle[0] as usize];
                        let v1 = positions[triangle[1] as usize];
                        let v2 = positions[triangle[2] as usize];

                        edges.push((v0, v1));
                        edges.push((v1, v2));
                        edges.push((v2, v0));
                    }
                }
            }
        }
    }

    edges
}

fn clear_edge_highlights(
    commands: &mut Commands,
    highlighted_edges: &mut ResMut<HighlightedEdges>,
) {
    for entity in highlighted_edges.cylinders.drain(..) {
        commands.entity(entity).despawn();
    }
}

fn highlight_cgar_edge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    highlighted_edges: &mut ResMut<HighlightedEdges>,
    cgar_mesh: &CgarMesh<CgarF64, 3>,
    edge_vertices: (usize, usize),
    mesh_transform: &GlobalTransform,
    original_entity: Entity,
) {
    // Get the specific edge from CGAR mesh
    if let Some(edge) = cgar_mesh.edge_half_edges(edge_vertices.0, edge_vertices.1) {
        // Get edge vertices
        let start_vertex = &cgar_mesh.vertices[edge_vertices.0];
        let end_vertex = &cgar_mesh.vertices[edge_vertices.1];

        let start = bevy::math::Vec3::new(
            start_vertex.position[0].0 as f32,
            start_vertex.position[1].0 as f32,
            start_vertex.position[2].0 as f32,
        );
        let end = bevy::math::Vec3::new(
            end_vertex.position[0].0 as f32,
            end_vertex.position[1].0 as f32,
            end_vertex.position[2].0 as f32,
        );

        // Create cylinder to highlight this specific edge
        let cylinder = create_edge_cylinder(
            commands,
            meshes,
            materials,
            start,
            end,
            mesh_transform,
            edge_vertices,
            original_entity,
        );
        highlighted_edges.cylinders.push(cylinder);

        println!("Highlighted edge {:?}", edge_vertices);
    }
}

fn create_edge_cylinder(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start: bevy::math::Vec3,
    end: bevy::math::Vec3,
    mesh_transform: &GlobalTransform,
    edge_vertices: (usize, usize),
    original_entity: Entity,
) -> Entity {
    let world_start = mesh_transform.transform_point(start);
    let world_end = mesh_transform.transform_point(end);

    let direction = world_end - world_start;
    let length = direction.length();
    let center = (world_start + world_end) / 2.0;

    // Create cylinder mesh
    let cylinder_mesh = Mesh::from(bevy::math::primitives::Cylinder {
        radius: 0.005, // Slightly larger for better visibility
        half_height: length / 2.0,
    });

    let mesh_handle = meshes.add(cylinder_mesh);
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),      // Red highlight
        emissive: Color::srgb(0.8, 0.0, 0.0).into(), // Brighter emission
        ..default()
    });

    // Calculate rotation to align cylinder with edge
    let up = bevy::math::Vec3::Y;
    let rotation = if direction.length() > 0.001 {
        bevy::math::Quat::from_rotation_arc(up, direction.normalize())
    } else {
        bevy::math::Quat::IDENTITY
    };

    commands
        .spawn((
            MeshMaterial3d(material_handle),
            Mesh3d(mesh_handle),
            Transform {
                translation: center,
                rotation,
                ..default()
            },
            NoWireframe,
            EdgeHighlight { original_entity },
        ))
        .id()
}
