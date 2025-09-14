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
    ecs::{hierarchy::ChildOf, query::With, system::Commands},
    math::{EulerRot, Quat, Vec2, Vec3},
    pbr::{AmbientLight, DirectionalLight},
    picking::mesh_picking::MeshPickingCamera,
    render::camera::{OrthographicProjection, Projection, ScalingMode},
    transform::components::Transform,
    utils::default,
    window::{PrimaryWindow, Window},
};

use crate::camera::components::OrbitCamera;

pub fn setup_camera_and_light(mut commands: Commands) {
    // Camera with sensible transform
    let camera_entity = commands
        .spawn((
            Camera3d::default(),
            Projection::Orthographic(OrthographicProjection {
                near: 0.01,
                far: 1000.0,
                scale: 2.0, // Increase scale to see the unit cube better
                viewport_origin: Vec2::new(0.5, 0.5),
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 2.0,
                },
                // Remove manual area setting - let it be computed automatically
                ..OrthographicProjection::default_3d()
            }),
            // Move camera further back to avoid near plane issues
            Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            MeshPickingCamera,
            OrbitCamera {
                focus: Vec3::ZERO,
                radius: 10.0,
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

pub fn sync_camera_aspect(
    windows: bevy::ecs::system::Query<&Window, With<PrimaryWindow>>,
    mut q: bevy::ecs::system::Query<
        (
            &bevy::render::camera::Camera,
            &mut bevy::render::camera::Projection,
        ),
        With<Camera3d>,
    >,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    for (camera, mut proj) in &mut q {
        let (w, h) = if let Some(vp) = camera.viewport.as_ref() {
            (vp.physical_size.x as f32, vp.physical_size.y as f32)
        } else {
            (
                window.physical_width() as f32,
                window.physical_height() as f32,
            )
        };
        if h > 0.0 {
            if let bevy::render::camera::Projection::Perspective(p) = proj.as_mut() {
                p.aspect_ratio = w / h;
            }
        }
    }
}
