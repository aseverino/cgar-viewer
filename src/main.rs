#![recursion_limit = "512"]

use std::ops::{Add, Div, Mul, Neg, Sub};

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::picking::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use cgar::io::obj::read_obj;
use cgar::mesh::basic_types::Mesh as CgarMesh;
use cgar::numeric::cgar_f64::CgarF64;
use cgar::numeric::scalar::Scalar as CgarScalar;

#[derive(Component)]
struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub last_mouse_pos: Option<Vec2>,
}

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
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.07)))
        .add_systems(Startup, (setup_camera_light, setup_cgar_mesh))
        .add_systems(Update, (toggle_wireframe, camera_controller))
        .run();
}

fn setup_camera_light(mut commands: Commands) {
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

// ---- Example: convert a CGAR mesh (3D) to a Bevy Mesh ----
// Adapt trait bounds to your Scalar setup. We’ll cast to f32 for GPU.
fn cgar_to_bevy_mesh<T: CgarScalar>(m: &CgarMesh<T, 3>) -> Mesh
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

fn setup_cgar_mesh(
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

// Quick keyboard toggle for wireframe
fn toggle_wireframe(kb: Res<ButtonInput<KeyCode>>, mut config: ResMut<WireframeConfig>) {
    if kb.just_pressed(KeyCode::KeyW) {
        config.global = !config.global;
        info!("Wireframe: {}", config.global);
    }
}

// Camera controller system for orbit camera
fn camera_controller(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
) {
    let Ok((mut transform, mut orbit)) = camera_query.single_mut() else {
        return;
    };

    let mut rotation_move = Vec2::ZERO;
    let mut pan_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if mouse_buttons.pressed(MouseButton::Left) {
        for mouse_event in mouse_motion.read() {
            if let Some(last_pos) = orbit.last_mouse_pos {
                let actual_delta = mouse_event.delta - last_pos;
                rotation_move += actual_delta;
            }
            orbit.last_mouse_pos = Some(mouse_event.delta);
        }
    } else if mouse_buttons.pressed(MouseButton::Right) {
        // Handle panning with right mouse button
        for mouse_event in mouse_motion.read() {
            if let Some(last_pos) = orbit.last_mouse_pos {
                let actual_delta = mouse_event.delta - last_pos;
                pan_move += actual_delta;
            }
            orbit.last_mouse_pos = Some(mouse_event.delta);
        }
    } else {
        orbit.last_mouse_pos = None;
        // Still consume events
        for _mouse_event in mouse_motion.read() {}
    }

    // Consume remaining mouse motion events
    for _mouse_event in mouse_motion.read() {
        // Consume the events
    }

    for wheel_event in mouse_wheel.read() {
        scroll += wheel_event.y;
    }

    // Handle zoom with mouse wheel or keyboard
    if scroll.abs() > 0.0 {
        orbit.radius -= scroll * orbit.radius * 0.05;
        orbit.radius = orbit.radius.clamp(0.1, 20.0);
        orbit_button_changed = true;
    }

    // Keyboard zoom
    if keyboard.pressed(KeyCode::Equal) {
        orbit.radius -= 0.1;
        orbit.radius = orbit.radius.clamp(0.1, 20.0);
        orbit_button_changed = true;
    }
    if keyboard.pressed(KeyCode::Minus) {
        orbit.radius += 0.1;
        orbit.radius = orbit.radius.clamp(0.1, 20.0);
        orbit_button_changed = true;
    }

    // Handle rotation
    if rotation_move.length_squared() > 0.0 {
        let sensitivity = 0.005;
        let delta_x = rotation_move.x * sensitivity;
        let delta_y = rotation_move.y * sensitivity;

        // Convert current position to spherical coordinates
        let offset = transform.translation - orbit.focus;
        let mut theta = offset.z.atan2(offset.x); // Azimuth angle
        let mut phi = (offset.y / orbit.radius).acos(); // Polar angle

        // Update angles
        theta += delta_x; // Yaw (horizontal rotation)
        phi -= delta_y; // Pitch (vertical rotation)

        // Clamp phi to prevent flipping
        phi = phi.clamp(0.01, std::f32::consts::PI - 0.01);

        // Convert back to Cartesian coordinates
        let new_position = Vec3::new(
            orbit.radius * phi.sin() * theta.cos(),
            orbit.radius * phi.cos(),
            orbit.radius * phi.sin() * theta.sin(),
        );

        transform.translation = orbit.focus + new_position;
        transform.look_at(orbit.focus, Vec3::Y);

        orbit_button_changed = true;
    }

    // Add panning logic after the rotation handling:
    if pan_move.length_squared() > 0.0 {
        let pan_sensitivity = 0.001;

        // Get camera's right and up vectors for screen-space panning
        let camera_right = transform.local_x();
        let camera_up = transform.local_y();

        // Calculate pan offset in world space
        let pan_offset =
            (-camera_right * pan_move.x + camera_up * pan_move.y) * pan_sensitivity * orbit.radius;

        // Move the focus point
        orbit.focus += pan_offset;

        // Update camera position to maintain same relative position to new focus
        let offset = transform.translation - (orbit.focus - pan_offset);
        transform.translation = orbit.focus + offset;
        transform.look_at(orbit.focus, Vec3::Y);

        orbit_button_changed = true;
    }

    if orbit_button_changed {
        // Apply the radius
        let mut position = transform.translation - orbit.focus;
        position = position.normalize() * orbit.radius;
        transform.translation = orbit.focus + position;
        transform.look_at(orbit.focus, Vec3::Y);
    }
}
