/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use bytemuck::{Pod, Zeroable};
use std::ops::{Add, Index, Mul};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupLayout, Buffer, PipelineLayout, RenderPipeline, Sampler, ShaderModule, TextureFormat,
};

#[derive(Copy, Clone)]
pub struct Mx4([FVec4; 4]);
impl Mx4 {
    #[inline]
    pub fn from_scale(x: f32, y: f32, z: f32) -> Self {
        Self::from([
            [x, 0.0, 0.0, 0.0],
            [0.0, y, 0.0, 0.0],
            [0.0, 0.0, z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    #[inline]
    pub fn from_translation(x: f32, y: f32, z: f32) -> Self {
        Self::from([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, z, 1.0],
        ])
    }
}

impl From<[f32; 4]> for FVec4 {
    fn from(v: [f32; 4]) -> Self {
        Self(v)
    }
}

impl From<[[f32; 4]; 4]> for Mx4 {
    fn from(v: [[f32; 4]; 4]) -> Self {
        Self([v[0].into(), v[1].into(), v[2].into(), v[3].into()])
    }
}

impl Index<usize> for Mx4 {
    type Output = FVec4;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Mul<f32> for FVec4 {
    type Output = FVec4;

    fn mul(self, rhs: f32) -> Self::Output {
        Self([self[0] * rhs, self[1] * rhs, self[2] * rhs, self[3] * rhs])
    }
}

impl Add<Self> for FVec4 {
    type Output = FVec4;

    fn add(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3],
        ])
    }
}

impl Mul<Self> for Mx4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let a = self[0];
        let b = self[1];
        let c = self[2];
        let d = self[3];

        Self([
            a * rhs[0][0] + b * rhs[0][1] + c * rhs[0][2] + d * rhs[0][3],
            a * rhs[1][0] + b * rhs[1][1] + c * rhs[1][2] + d * rhs[1][3],
            a * rhs[2][0] + b * rhs[2][1] + c * rhs[2][2] + d * rhs[2][3],
            a * rhs[3][0] + b * rhs[3][1] + c * rhs[3][2] + d * rhs[3][3],
        ])
    }
}

#[derive(Copy, Clone)]
pub struct FVec4(pub [f32; 4]);

impl Index<usize> for FVec4 {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SpriteUniform {
    model: Mx4, // Transformation matrix
    tex_coords: FVec4,
}

unsafe impl Pod for SpriteUniform {}
unsafe impl Zeroable for SpriteUniform {}

impl SpriteUniform {
    pub fn new(model: Mx4, tex_coords: FVec4) -> Self {
        Self { model, tex_coords }
    }
}

/* CREATE SPRITE UNIFORM

// Create a uniform buffer for a single sprite
let sprite_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Sprite Uniform Buffer"),
    contents: bytemuck::cast_slice(&[SpriteUniform {
        model: Matrix4::identity().into(),
        tex_coords: [0.0, 0.0, 1.0, 1.0],
    }]),
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
});

// Create bind group layout for sprite uniforms
let sprite_uniform_bind_group_layout =
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Sprite Uniform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

// Create bind group for sprite uniforms
let sprite_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("Sprite Uniform Bind Group"),
    layout: &sprite_uniform_bind_group_layout,
    entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: sprite_uniform_buffer.as_entire_binding(),
    }],
});

 */

/*
struct Uniforms {
    projection: mat4x4<f32>;
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct SpriteUniform {
    model: mat4x4<f32>;
    tex_coords: vec4<f32>;
};

@group(1) @binding(0)
var<uniform> sprite_uniform: SpriteUniform;

struct VertexInput {
    @location(0) position: vec2<f32>;
    @location(1) tex_coords: vec2<f32>;
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>;
    @location(0) tex_coords: vec2<f32>;
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Apply model and projection transformations
    let pos = uniforms.projection * sprite_uniform.model * vec4<f32>(input.position, 0.0, 1.0);
    output.position = pos;
    output.tex_coords = input.tex_coords * sprite_uniform.tex_coords.xy + sprite_uniform.tex_coords.zw;

    return output;
}
 */

/*
@group(1) @binding(1)
var texture0: texture_2d<f32>;

@group(1) @binding(2)
var sampler0: sampler;

struct FragmentInput {
    @location(0) tex_coords: vec2<f32>;
};

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let color = textureSample(texture0, sampler0, input.tex_coords);
    return color;
}
 */

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],   // 2D position of the vertex
    tex_coords: [f32; 2], // Texture coordinates
}

// Implement Zeroable manually
unsafe impl Zeroable for Vertex {}

// Implement Pod manually
unsafe impl Pod for Vertex {}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// wgpu has, for very unknown reasons, put coordinate texture origo at top-left(!)
//const RIGHT: f32 = 128.0 / 512.0;
//const DOWN: f32 = 128.0 / 1024.0;

const RIGHT: f32 = 32.0 / 128.0;
const DOWN: f32 = 32.0 / 64.0;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5],
        tex_coords: [0.0, DOWN],
    }, // Bottom left
    Vertex {
        position: [0.5, -0.5],
        tex_coords: [RIGHT, DOWN],
    }, // Bottom right
    Vertex {
        position: [0.5, 0.5],
        tex_coords: [RIGHT, 0.0],
    }, // Top right
    Vertex {
        position: [-0.5, 0.5],
        tex_coords: [0.0, 0.0],
    }, // Top left
];

// u16 is the smallest index buffer supported by wgpu // IndexFormat
pub const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[derive(Debug)]
pub struct SpriteInfo {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
}

impl SpriteInfo {
    pub fn new(
        device: &wgpu::Device,
        surface_texture_format: TextureFormat,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
    ) -> Self {
        let vertex_shader =
            swamp_wgpu::create_shader_module(device, "sprite vertex", vertex_shader_source);
        let fragment_shader =
            swamp_wgpu::create_shader_module(device, "sprite fragment", fragment_shader_source);

        let bind_group_layout = create_sprite_bind_group_layout(device, "sprite bind group layout");
        let default_layout = swamp_wgpu::create_pipeline_layout(
            device,
            "sprite pipeline layout",
            &bind_group_layout,
        );

        let pipeline = create_sprite_pipeline(
            device,
            surface_texture_format,
            &default_layout,
            &vertex_shader,
            &fragment_shader,
        );

        let sampler = swamp_wgpu::create_nearest_sampler(device, "sprite nearest sampler");

        Self {
            pipeline,
            bind_group_layout,
            sampler,
        }
    }
}

pub fn load_texture_from_memory(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    octets: &[u8],
    label: &str,
) -> wgpu::Texture {
    let img = image::load_from_memory_with_format(octets, image::ImageFormat::Png)
        .expect("Failed to load image");
    let img = img.to_rgba8();
    let (width, height) = img.dimensions();

    // Create the texture and upload the data (same as before)
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[TextureFormat::Rgba8UnormSrgb],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    texture
}

pub fn create_sprite_vertex_buffer(device: &wgpu::Device, label: &str) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

pub fn create_sprite_index_buffer(device: &wgpu::Device, label: &str) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    })
}

pub fn create_sprite_bind_group_layout(device: &wgpu::Device, label: &str) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
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
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
    })
}

fn create_sprite_pipeline(
    device: &wgpu::Device,
    format: TextureFormat,
    pipeline_layout: &PipelineLayout,
    vertex_shader: &ShaderModule,
    fragment_shader: &ShaderModule,
) -> RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Sprite Alpha Blend Pipeline"),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        },

        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
