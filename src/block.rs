use cgmath::{Quaternion, Vector3, Zero};

use crate::Instance;

#[derive(Clone)]
pub struct Block {
    pub position: Vector3<f32>,
}

impl Block {
    fn render(&self) {}

    pub fn to_instance(&self) -> Instance {
        Instance {
            position: self.position,
            rotation: Quaternion::zero(),
        }
    }
}
