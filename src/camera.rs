use crate::{
    model::{DrawModel, Vertex},
    Instance,
};
use cgmath::{Deg, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3};
use std::iter;
use wgpu::util::DeviceExt;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub window: winit::window::Window,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface, // The surface represents something to draw on
    pub device: wgpu::Device,   // Adapter to our graphics card
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    // Camera configuration
    camera_uniform: CameraUniform,
    camera_projection: Projection,
    camera_buffer: wgpu::Buffer,
    // Rendering
    render_pipeline: wgpu::RenderPipeline,
    depth_map: crate::texture::Texture,
    // Bind groups
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

struct CameraBindings {
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
}

impl Camera {
    pub async fn new(window: winit::window::Window) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU. BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        // The surface needs to live as long as the window that created it. State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let depth_map =
            crate::texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let camera_projection = Projection::new(config.width, config.height, Deg(45.0), 0.1, 200.0);
        let camera_uniform = CameraUniform::new();

        let texture_bind_group_layout = Camera::create_texture_bindings(&device);
        let camera_bindings = Camera::create_camera_bindings(&device, camera_uniform);
        let render_pipeline = Camera::complete_bindings(
            &device,
            &config,
            &texture_bind_group_layout,
            &camera_bindings.camera_bind_group_layout,
        );

        Self {
            window,
            size,
            surface,
            device,
            queue,
            config,
            // Camera configuration
            camera_uniform,
            camera_projection,
            camera_buffer: camera_bindings.camera_buffer,
            // Rendering
            render_pipeline,
            depth_map,
            // Bind groups
            camera_bind_group_layout: camera_bindings.camera_bind_group_layout,
            camera_bind_group: camera_bindings.camera_bind_group,
            texture_bind_group_layout,
        }
    }

    fn create_texture_bindings(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        texture_bind_group_layout
    }

    fn create_camera_bindings(
        device: &wgpu::Device,
        camera_uniform: CameraUniform,
    ) -> CameraBindings {
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        CameraBindings {
            camera_bind_group_layout,
            camera_bind_group,
            camera_buffer,
        }
    }

    fn complete_bindings(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[texture_bind_group_layout, camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            Camera::create_render_pipeline(
                &device,
                config.format,
                &render_pipeline_layout,
                Some(crate::texture::Texture::DEPTH_FORMAT),
                &[
                    crate::model::ModelVertex::desc(),
                    crate::InstanceRaw::desc(),
                ],
                shader,
            )
        };
        render_pipeline
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        layout: &wgpu::PipelineLayout,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        shader: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(shader);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{:?}", shader)),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill, // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                unclipped_depth: false,                // Requires Features::DEPTH_CLIP_CONTROL
                conservative: false, // Requires Features::CONSERVATIVE_RASTERIZATION
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        })
    }

    pub fn render(&self, world: &crate::world::World) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let instance_data = world
            .blocks()
            .values()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_map.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

            render_pass.set_pipeline(&self.render_pipeline);
            let block_instances = world.blocks().values();
            render_pass.draw_model_instanced(
                &world.obj_model,
                0..block_instances.len() as u32,
                &self.camera_bind_group,
            );
        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn update(&mut self, position: &Point3<f32>, pitch: Rad<f32>, yaw: Rad<f32>) {
        self.camera_uniform.update_view_projection(
            position.clone(), // TODO do we really have to clone this?
            pitch,
            yaw,
            &self.camera_projection,
        );
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera_projection.resize(width, height);

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth_map = crate::texture::Texture::create_depth_texture(
            &self.device,
            &self.config,
            "depth_texture",
        );
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_projection(
        &mut self,
        position: Point3<f32>,
        pitch: Rad<f32>,
        yaw: Rad<f32>,
        projection: &Projection,
    ) {
        self.view_position = position.to_homogeneous().into();

        let (sin_pitch, cos_pitch) = pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = yaw.0.sin_cos();
        let matrix = Matrix4::look_to_rh(
            position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        );

        self.view_proj = (projection.calc_matrix() * matrix).into()
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
