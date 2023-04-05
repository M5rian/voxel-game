use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use winit::{dpi::PhysicalPosition, event::*};

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
        self.update_position(inputs, dt);
        self.camera.update(&self.position, self.pitch(), self.yaw());
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
        let mouse_average_x: f64 = inputs
            .mouse_position_history
            .iter()
            .map(|position| position.x)
            .sum::<f64>()
            / CameraController::MOUSE_HISTORY_BUFFER_SIZE as f64;
        let mouse_average_y: f64 = inputs
            .mouse_position_history
            .iter()
            .map(|position| position.y)
            .sum::<f64>()
            / CameraController::MOUSE_HISTORY_BUFFER_SIZE as f64;
        let rotation_x_delta = (mouse_average_x - inputs.last_mouse_position_average_x) as f32;
        let rotation_y_delta = (mouse_average_y - inputs.last_mouse_position_average_y) as f32;
        inputs.last_mouse_position_average_x = mouse_average_x;
        inputs.last_mouse_position_average_y = mouse_average_y;

        //println!("input: {}", rotation_x_delta);

        self.yaw += Rad(rotation_x_delta) * inputs.sensitivity * dt;
        self.pitch -= Rad(rotation_y_delta) * inputs.sensitivity * dt;
        self.pitch = Rad(self.pitch.0.clamp(-PITCH_CLAMP.0, PITCH_CLAMP.0));
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
    mouse_position_history: [PhysicalPosition<f64>; CameraController::MOUSE_HISTORY_BUFFER_SIZE],
    mouse_position_history_index: usize,
    last_mouse_position_average_x: f64,
    last_mouse_position_average_y: f64,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    const MOUSE_HISTORY_BUFFER_SIZE: usize = 10;
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            mouse_position_history: [PhysicalPosition::new(0.0, 0.0);
                CameraController::MOUSE_HISTORY_BUFFER_SIZE],
            mouse_position_history_index: 0,
            last_mouse_position_average_x: 0.0,
            last_mouse_position_average_y: 0.0,
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

    pub fn process_mouse(&mut self, position: PhysicalPosition<f64>) {
        self.mouse_position_history[self.mouse_position_history_index] = position;
        self.mouse_position_history_index =
            (self.mouse_position_history_index + 1) % CameraController::MOUSE_HISTORY_BUFFER_SIZE;
    }
}
