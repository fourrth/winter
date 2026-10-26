use std::ops::Sub;

use nalgebra::{Matrix4, Orthographic3, Point3, Vector3};

/// A first-person / fly camera.
///
/// Conventions:
/// - Right-handed, Y-up world.
/// - `yaw = 0`, `pitch = 0` looks down **-Z**.
/// - Increasing `yaw` turns *left*; increasing `pitch` looks *up*.
/// - `yaw` and `pitch` are stored in **radians**.
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,

    /// Radians of rotation per pixel of mouse movement.
    pub sensitivity: f32,
    /// Movement speed, world units per second.
    pub speed: f32,
    /// Multiplier applied while the "fast" key is held.
    pub sprint_multiplier: f32,

    pub fov_y: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
}

/// Input state for one frame
#[derive(Default, Clone, Copy)]
pub struct InputFrame {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub sprint: bool,
}

impl Camera {
    pub const WORLD_UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);

    pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.001;

    pub fn new(position: Point3<f32>, yaw: f32, pitch: f32, aspect: f32) -> Self {
        Self {
            position,
            yaw,
            pitch: pitch.clamp(-Self::PITCH_LIMIT, Self::PITCH_LIMIT),
            sensitivity: 0.0025, // ~0.14 deg per pixel
            speed: 5.0,
            sprint_multiplier: 3.0,
            fov_y: 60.0_f32.to_radians(),
            aspect,
            znear: 0.1,
            zfar: 1000.0,
        }
    }
    /// Unit vector the camera is looking along.
    pub fn forward(&self) -> Vector3<f32> {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vector3::new(-sy * cp, sp, -cy * cp)
    }

    pub fn right(&self) -> Vector3<f32> {
        let (sy, cy) = self.yaw.sin_cos();
        Vector3::new(cy, 0.0, -sy)
    }

    /// local up,
    pub fn up(&self) -> Vector3<f32> {
        self.right().cross(&self.forward()).normalize()
    }

    /// `forward()` projected onto the XZ plane
    pub fn horizontal_forward(&self) -> Vector3<f32> {
        let (sy, cy) = self.yaw.sin_cos();
        Vector3::new(-sy, 0.0, -cy)
    }
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            &self.position,
            &(self.position + self.forward()),
            &Self::WORLD_UP,
        )
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(self.aspect, self.fov_y, self.znear, self.zfar)
    }

    // ---- Input -----------------------------------------------------------

    /// `dx > 0` (mouse moved right) turns the camera right;
    /// `dy > 0` (mouse moved down) looks down — matching screen-space input, so top left corner is 0,0 in clip space
    pub fn apply_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * self.sensitivity;
        self.pitch -= dy * self.sensitivity;

        self.pitch = self.pitch.clamp(-Self::PITCH_LIMIT, Self::PITCH_LIMIT);
        // Keep yaw in [0, TAU) so it never grows unbounded.
        self.yaw = self.yaw.rem_euclid(std::f32::consts::TAU);
    }

    /// Move the camera according to the given keys, for `dt` seconds.
    /// Uses projected forward
    pub fn apply_keyboard(&mut self, input: &InputFrame, dt: f32) {
        let mut dir = Vector3::zeros();

        let fwd = self.horizontal_forward();
        let right = self.right();

        if input.forward {
            dir += fwd;
        }
        if input.backward {
            dir -= fwd;
        }
        if input.right {
            dir += right;
        }
        if input.left {
            dir -= right;
        }
        if input.up {
            dir += Self::WORLD_UP;
        }
        if input.down {
            dir -= Self::WORLD_UP;
        }

        // Only normalize if we actually moved
        if let Some(unit) = dir.try_normalize(1e-6) {
            let speed = if input.sprint {
                self.speed * self.sprint_multiplier
            } else {
                self.speed
            };
            self.position += unit * speed * dt;
        }
    }

    /// Update the aspect ratio after a framebuffer resize.
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}
