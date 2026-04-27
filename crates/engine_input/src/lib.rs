use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy, Default)]
pub struct InputSnapshot {
    pub escape_pressed: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub interact_pressed: bool,
}

pub fn handle_keyboard_event(snapshot: &mut InputSnapshot, event: &winit::event::KeyEvent) {
    let is_pressed = event.state == ElementState::Pressed;

    match event.physical_key {
        PhysicalKey::Code(KeyCode::Escape) => snapshot.escape_pressed = is_pressed,
        PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
            snapshot.move_left = is_pressed
        }
        PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
            snapshot.move_right = is_pressed
        }
        PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
            snapshot.move_up = is_pressed
        }
        PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
            snapshot.move_down = is_pressed
        }
        PhysicalKey::Code(KeyCode::KeyE) | PhysicalKey::Code(KeyCode::Space) => {
            snapshot.interact_pressed = is_pressed
        }
        _ => {}
    }
}
