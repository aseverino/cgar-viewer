use bevy::{
    core_pipeline::core_3d::Camera3d,
    ecs::{
        event::EventReader,
        query::With,
        system::{Query, Res},
    },
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion, MouseWheel},
    },
    math::{Vec2, Vec3},
    transform::components::Transform,
};

use crate::camera::components::OrbitCamera;

// Camera controller system for orbit camera
pub fn camera_controller(
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
