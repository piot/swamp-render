/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use log::info;
use wgpu::{BindGroup, BindGroupLayout, PipelineLayout, Sampler, ShaderModule, Texture};

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

pub fn create_bind_group_layout(device: &wgpu::Device, label: &str) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

pub fn create_bind_group(
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
