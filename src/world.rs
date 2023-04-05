use crate::Instance;
use cgmath::{Quaternion, Vector3, Zero};
use std::collections::HashMap;

pub struct World {
    blocks: HashMap<Vector3<i32>, Instance>,
    pub obj_model: crate::model::Model,
}

impl World {
    pub async fn new(camera: &crate::camera::Camera) -> Self {
        let debug_material = {
            let diffuse_bytes = include_bytes!("../res/cobble-diffuse.png");
            let diffuse_texture = crate::texture::Texture::from_bytes(
                &camera.device,
                &camera.queue,
                diffuse_bytes,
                "res/alt-diffuse.png",
                false,
            )
            .unwrap();
            crate::model::Material::new(
                &camera.device,
                "alt-material",
                diffuse_texture,
                &camera.texture_bind_group_layout,
            )
        };

        let obj_model = crate::resources::load_model(
            "cube.obj",
            &camera.device,
            &camera.queue,
            &camera.texture_bind_group_layout,
        )
        .await
        .unwrap();

        let mut blocks: HashMap<Vector3<i32>, Instance> = HashMap::new();
        for x in 0..100 {
            for z in 0..100 {
                let position = Vector3 {
                    x: x * 2,
                    y: 0,
                    z: z * 2,
                };
                let position_exact = Vector3 {
                    x: x as f32 * 2.0,
                    y: 0.0,
                    z: z as f32 * 2.0,
                };
                let instance = Instance {
                    position: position_exact,
                    rotation: Quaternion::zero(),
                };
                blocks.insert(position, instance);
            }
        }

        Self { blocks, obj_model }
    }

    pub fn blocks(&self) -> &HashMap<Vector3<i32>, Instance> {
        &self.blocks
    }

    pub fn destroy(&mut self, coords: &Vector3<i32>) {
        self.blocks.remove(coords);
    }

    pub fn place(&mut self, coords: Vector3<i32>) {
        let instance = Instance {
            position: coords.map(|v| v as f32),
            rotation: Quaternion::zero(),
        };
        self.blocks.insert(coords, instance);
    }
}
