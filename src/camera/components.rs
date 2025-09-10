use bevy::{
    ecs::component::Component,
    math::{Vec2, Vec3},
};

#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub last_mouse_pos: Option<Vec2>,
}
