use glam::{Mat4, Vec2, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Camera2D {
    pub center: Vec2,
    pub zoom: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl Camera2D {
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            viewport_width: viewport_width.max(1) as f32,
            viewport_height: viewport_height.max(1) as f32,
        }
    }

    pub fn with_center(mut self, center: Vec2) -> Self {
        self.center = center;
        self
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width.max(1) as f32;
        self.viewport_height = height.max(1) as f32;
    }

    pub fn pan(&mut self, delta: Vec2) {
        self.center += delta;
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.max(0.05);
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        let half_width = (self.viewport_width * 0.5) / self.zoom;
        let half_height = (self.viewport_height * 0.5) / self.zoom;
        let projection = Mat4::orthographic_rh_gl(
            -half_width,
            half_width,
            -half_height,
            half_height,
            -1.0,
            1.0,
        );
        let view = Mat4::from_translation(Vec3::new(-self.center.x, -self.center.y, 0.0));
        projection * view
    }
}
