use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use winit::dpi::PhysicalPosition;
use winit::event::*;

const PITCH_CLAMP: Rad<f32> = Rad(FRAC_PI_2 - 0.0001);

pub struct Player {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    camera: crate::camera::Camera,
}

impl Player {
    pub fn new(
        position: Point3<f32>,
        yaw: Rad<f32>,
        pitch: Rad<f32>,
        camera: crate::camera::Camera,
    ) -> Self {
        Self {
            position,
            yaw,
            pitch,
            camera,
        }
    }

    pub fn yaw(&self) -> Rad<f32> {
        self.yaw
    }

    pub fn pitch(&self) -> Rad<f32> {
        self.pitch
    }

    pub fn camera(&self) -> &crate::camera::Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut crate::camera::Camera {
        &mut self.camera
    }

    pub fn update(
        &mut self,
        inputs: &mut crate::player::CameraController,
        dt: std::time::Duration,
    ) {
        self.camera.update(&self.position, self.pitch(), self.yaw());
        self.update_position(inputs, dt);
    }

    fn update_position(
        &mut self,
        inputs: &mut crate::player::CameraController,
        dt: std::time::Duration,
    ) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.position +=
            forward * (inputs.amount_forward - inputs.amount_backward) * inputs.speed * dt;
        self.position += right * (inputs.amount_right - inputs.amount_left) * inputs.speed * dt;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        self.position.y += (inputs.amount_up - inputs.amount_down) * inputs.speed * dt;

        // Rotate
        self.yaw += Rad(inputs.rotate_horizontal) * inputs.sensitivity * dt;
        self.pitch += Rad(-inputs.rotate_vertical) * inputs.sensitivity * dt;
        self.pitch = Rad(self.pitch.0.clamp(-PITCH_CLAMP.0, PITCH_CLAMP.0));

        inputs.rotate_horizontal = 0.0;
        inputs.rotate_vertical = 0.0;
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };
    }
}
