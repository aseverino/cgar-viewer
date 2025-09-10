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
use crate::lighting::setup::setup_camera_light;
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
        .add_plugins((
            MeshPickingPlugin, // built-in mesh picking
            WireframePlugin::default(),
        ))
        .add_systems(Startup, (setup_camera_light, setup_cgar_mesh))
        .add_systems(Update, (toggle_wireframe, camera_controller))
        .run();
}
