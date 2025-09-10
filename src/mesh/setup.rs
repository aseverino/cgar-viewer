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
use cgar::{io::obj::read_obj, numeric::cgar_f64::CgarF64};

use crate::mesh::conversion::cgar_to_bevy_mesh;

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
    let cgar_mesh =
        read_obj::<CgarF64, _>("/mnt/v/cgar_meshes/difference_large_boolean.obj").unwrap(); // Replace with your actual CGAR mesh
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
    ));
}
