/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use log::info;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, Sampler, ShaderModule, Texture};

use bytemuck::{Pod, Zeroable};

pub fn create_shader_module(
    device: &wgpu::Device,
    name: &str,
    shader_source: &str,
) -> ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    })
}

pub fn create_pipeline_layout(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    info!("creating pipeline layout");
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub fn create_nearest_sampler(device: &wgpu::Device, label: &str) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: None,
        anisotropy_clamp: 1,
        lod_min_clamp: 0.0,
        lod_max_clamp: 32.0,
        border_color: None,
    })
}

pub fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> Texture {
    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("My Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm], // Specify the view format(s),
    })
}

pub fn create_texture_and_sampler_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &BindGroupLayout,
    sampler: &Sampler,
    texture: Texture,
    label: &str,
) -> BindGroup {
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
        label: Some(label),
    })
}

/// A struct that holds uniform data to be passed to shaders.
/// In this case, it contains the combined view-projection matrix.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
}

unsafe impl Pod for Uniforms {}
unsafe impl Zeroable for Uniforms {}

pub fn create_uniform_buffer(device: &wgpu::Device, label: &str) -> Buffer {
    let uniforms = Uniforms {
        view_proj: [[0.0; 4]; 4],
    };

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_uniform_bind_group_layout(device: &wgpu::Device, label: &str) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[
            // View-Projection Matrix
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
        ],
    })
}

// Create Uniform Bind Group
pub fn create_uniform_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &BindGroupLayout,
    uniform_buffer: &Buffer,
    label: &str,
) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    })
}
