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

use bevy::{
    color::Color,
    core_pipeline::core_3d::Camera3d,
    ecs::{hierarchy::ChildOf, system::Commands},
    math::{EulerRot, Quat, Vec3},
    pbr::{AmbientLight, DirectionalLight},
    picking::mesh_picking::MeshPickingCamera,
    render::camera::{PerspectiveProjection, Projection},
    transform::components::Transform,
    utils::default,
};

use crate::camera::components::OrbitCamera;

pub fn setup_camera_light(mut commands: Commands) {
    // Camera with sensible transform
    let camera_entity = commands
        .spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: std::f32::consts::PI / 6.0, // 30 degrees (narrower FOV for closer inspection)
                near: 0.01,                      // Very close near plane (default is usually 0.1)
                far: 1000.0,                     // Keep far plane reasonable
                aspect_ratio: 1.0,               // Will be adjusted automatically
            }),
            Transform::from_xyz(2.5, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            MeshPickingCamera,
            OrbitCamera {
                focus: Vec3::ZERO,
                radius: 5.0,
                upside_down: false,
                last_mouse_pos: None,
            },
        ))
        .id();

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
        affects_lightmapped_meshes: true,
    });

    commands
        .spawn((
            DirectionalLight {
                color: Color::WHITE,
                illuminance: 3000.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.25, -0.25, 0.0)),
        ))
        .insert(ChildOf(camera_entity));
}
