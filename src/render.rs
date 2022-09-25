use std::{collections::HashMap, hash::Hash, mem::size_of, cell::Cell};

use image::EncodableLayout;
use wgpu::util::DeviceExt;

use crate::state::{State, self};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BoxVertex {
    position: glam::Vec2,
    uv: glam::Vec2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    camera: glam::Mat4,
}

pub struct BoxRenderer {
    layout: wgpu::BindGroupLayout,
    resources: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl BoxRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        camera_size: glam::Vec2,
        texture_atlas: &TextureAtlas,
    ) -> Result<Self, anyhow::Error> {
        let uniforms = Uniforms {
            camera: glam::Mat4::orthographic_lh(0.0, camera_size.x, 0.0, camera_size.y, 0.0, 1.0),
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let resources = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_atlas.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture_atlas.texture.sampler),
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(wgpu::include_wgsl!("shaders/box.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<BoxVertex>() as _,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            multiview: None,
        });

        Ok(Self {
            layout,
            resources,
            pipeline,
        })
    }

    pub fn mesh_from_state(&self, device: &wgpu::Device, state: &State, texture_atlas: &TextureAtlas) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut index = 0;

        let min = state.player.body.pos;
        let max = min + state.player.body.size;
        let sprite = texture_atlas.get_sprite("player").unwrap();
        let scale = glam::vec2(1.0 / texture_atlas.width() as f32, 1.0 / texture_atlas.height() as f32);
        let uv_min = sprite.min * scale;
        let uv_max = (sprite.min + sprite.size) * scale;

        vertices.push(BoxVertex { position: min, uv: uv_min });
        vertices.push(BoxVertex { position: glam::vec2(max.x, min.y), uv: glam::vec2(uv_max.x, uv_min.y) });
        vertices.push(BoxVertex { position: max, uv: uv_max });
        vertices.push(BoxVertex { position: glam::vec2(min.x, max.y), uv: glam::vec2(uv_min.x, uv_max.y) });

        indices.push(index);
        indices.push(index + 1);
        indices.push(index + 2);
        indices.push(index);
        indices.push(index + 2);
        indices.push(index + 3);
        index += 4;

        let min = state.ball.body.pos;
        let max = min + state.ball.body.size;
        let sprite = texture_atlas.get_sprite("ball").unwrap();
        let uv_min = sprite.min * scale;
        let uv_max = (sprite.min + sprite.size) * scale;

        vertices.push(BoxVertex { position: min, uv: uv_min });
        vertices.push(BoxVertex { position: glam::vec2(max.x, min.y), uv: glam::vec2(uv_max.x, uv_min.y) });
        vertices.push(BoxVertex { position: max, uv: uv_max });
        vertices.push(BoxVertex { position: glam::vec2(min.x, max.y), uv: glam::vec2(uv_min.x, uv_max.y) });

        indices.push(index);
        indices.push(index + 1);
        indices.push(index + 2);
        indices.push(index);
        indices.push(index + 2);
        indices.push(index + 3);
        index += 4;


        for brick in &state.bricks {
            let body = &brick.body; 
            let min = body.pos;
            let max = min + body.size;
            let sprite_id = format!("brick{}", brick.status);
            if let Some(sprite) = texture_atlas.get_sprite(&sprite_id) {
                let uv_min = sprite.min * scale;
                let uv_max = (sprite.min + sprite.size) * scale;
                
                vertices.push(BoxVertex { position: min, uv: uv_min });
                vertices.push(BoxVertex { position: glam::vec2(max.x, min.y), uv: glam::vec2(uv_max.x, uv_min.y) });
                vertices.push(BoxVertex { position: max, uv: uv_max });
                vertices.push(BoxVertex { position: glam::vec2(min.x, max.y), uv: glam::vec2(uv_min.x, uv_max.y) });
    
                indices.push(index);
                indices.push(index + 1);
                indices.push(index + 2);
                indices.push(index);
                indices.push(index + 2);
                indices.push(index + 3);
                index += 4;
            }
        }

        Mesh::from_verts_and_indices(device, vertices, indices)
    }

    pub fn draw_mesh<'a, 'b>(&'a self, pass: &'b mut wgpu::RenderPass<'a>, mesh: &'a Mesh) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.resources, &[]);
        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
    }
}

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Mesh {
    fn from_verts_and_indices(device: &wgpu::Device, vertices: Vec<BoxVertex>, indices: Vec<u32>) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

pub struct TextureAtlas {
    texture: Texture,
    atlas: Atlas,
}

impl TextureAtlas {
    pub fn with_json(device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> Result<Self, anyhow::Error> {
        let data = std::fs::read_to_string(path)?;
        let atlas: Atlas = serde_json::from_str(&data)?;
        let texture = Texture::new(device, queue, &atlas.texture)?;

        Ok(Self {
            texture,
            atlas,
        })
    }

    pub fn get_sprite(&self, id: &str) -> Option<&'_ Sprite> {
        self.atlas.sprites.get(id)
    }

    pub fn width(&self) -> u32 {
        self.texture.width
    }

    pub fn height(&self) -> u32 {
        self.texture.height
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Atlas {
    texture: String,
    sprites: HashMap<String, Sprite>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Sprite {
    pub min: glam::Vec2,
    pub size: glam::Vec2,
}

pub struct Texture {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    view: wgpu::TextureView,
}

impl Texture {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> anyhow::Result<Self> {
        let img = image::open(path)?;
        let img = img.to_rgba8();

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: img.width(),
                    height: img.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            },
            img.as_bytes(),
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let view = texture.create_view(&Default::default());

        Ok(Self { width: img.width(), height: img.height(), texture, sampler, view })
    }
}
