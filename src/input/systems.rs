use bevy::{
    ecs::system::{Res, ResMut},
    input::{ButtonInput, keyboard::KeyCode},
    log::info,
    pbr::wireframe::WireframeConfig,
};

// Quick keyboard toggle for wireframe
pub fn toggle_wireframe(kb: Res<ButtonInput<KeyCode>>, mut config: ResMut<WireframeConfig>) {
    if kb.just_pressed(KeyCode::KeyW) {
        config.global = !config.global;
        info!("Wireframe: {}", config.global);
    }
}
