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

#![recursion_limit = "512"]

use bevy::pbr::wireframe::WireframePlugin;
use bevy::picking::prelude::*;
use bevy::prelude::*;

mod camera;
mod input;
mod lighting;
mod mesh;
mod utils;

use crate::camera::systems::camera_controller;
use crate::input::systems::toggle_wireframe;
use crate::lighting::setup::{setup_camera_and_light, sync_camera_aspect};
use crate::mesh::edge::{HighlightedEdges, handle_mesh_click};
use crate::mesh::setup::setup_cgar_mesh;
// ... other imports

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CGAR Viewer".into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<HighlightedEdges>()
        .add_plugins((
            MeshPickingPlugin, // built-in mesh picking
            WireframePlugin::default(),
        ))
        .add_systems(Startup, (setup_camera_and_light, setup_cgar_mesh))
        .add_systems(Update, (toggle_wireframe, camera_controller))
        .add_systems(
            PostUpdate,
            (
                sync_camera_aspect, // updates aspect from viewport/window
                handle_mesh_click,  // computes ray using correct projection + transforms
            )
                .chain()
                .after(TransformSystem::TransformPropagate),
        )
        .run();
}
