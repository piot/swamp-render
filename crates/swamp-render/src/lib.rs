/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use int_math::{URect, Vec2, Vec3};
use log::info;
use std::rc::Rc;
use std::sync::Arc;
use swamp_wgpu_sprites::SpriteInfo;
use wgpu::{BindGroup, BindGroupLayout, RenderPass, RenderPipeline};

#[derive(Debug)]
pub struct Render {
    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,

    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>, // Queue to talk to device

    sprites: Vec<Sprite>,
    materials: Vec<SpriteMaterialRef>,
    bind_group_layout: BindGroupLayout,
    sampler: wgpu::Sampler,
    pipeline: RenderPipelineRef,
}

impl Render {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>, // Queue to talk to device
        surface_texture_format: wgpu::TextureFormat,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
    ) -> Self {
        let sprite_info = SpriteInfo::new(
            &device,
            surface_texture_format,
            vertex_shader_source,
            fragment_shader_source,
        );

        let index_buffer =
            swamp_wgpu_sprites::create_sprite_index_buffer(&device, "sprite quad index buffer");
        let vertex_buffer =
            swamp_wgpu_sprites::create_sprite_vertex_buffer(&device, "sprite quad vertex buffer");

        Self {
            device,
            queue,
            sprites: Vec::new(),
            materials: Vec::new(),
            sampler: sprite_info.sampler,
            pipeline: Rc::new(sprite_info.pipeline),
            bind_group_layout: sprite_info.bind_group_layout,
            index_buffer,
            vertex_buffer,
        }
    }

    pub fn render_sprite(
        &mut self,
        position: Vec3,
        material: &SpriteMaterialRef,
        params: SpriteParams,
    ) {
        self.sprites.push(Sprite {
            position,
            material: Rc::clone(material),
            params,
        })
    }

    pub fn render_sprite_2d(
        &mut self,
        position: Vec2,
        material: &SpriteMaterialRef,
        params: SpriteParams,
    ) {
        self.sprites.push(Sprite {
            position: position.into(),
            material: Rc::clone(material),
            params,
        })
    }

    pub fn render(&mut self, render_pass: &mut RenderPass) {
        sort_sprites_by_z_then_y(&mut self.sprites);

        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        for sprite in &self.sprites {
            let sprite_texture_bind_group = &sprite.material.bind_group;
            render_pass.set_bind_group(0, sprite_texture_bind_group, &[]); // sets sampler and texture

            let num_indices = swamp_wgpu_sprites::INDICES.len() as u32;

            render_pass.draw_indexed(0..num_indices, 0, 0..1);
            render_pass.draw(0..3, 0..1);
        }
    }

    pub fn create_material_png(&mut self, png: &[u8], label: &str) -> SpriteMaterialRef {
        let texture =
            swamp_wgpu_sprites::load_texture_from_memory(&self.device, &self.queue, png, label);
        info!("loaded texture!");

        let bind_group = swamp_wgpu::create_bind_group(
            &self.device,
            &self.bind_group_layout,
            &self.sampler,
            texture,
            label,
        );

        let material = Rc::new(SpriteMaterial {
            bind_group,
            render_pipeline: Rc::clone(&self.pipeline),
        });
        self.materials.push(Rc::clone(&material));

        material
    }
}

fn sort_sprites_by_z_then_y(sprites: &mut [Sprite]) {
    sprites.sort_by_key(|sprite| (sprite.position.z, sprite.position.y));
}

#[derive(Default, Debug)]
pub struct SpriteParams {
    pub dest_size: Option<Vec2>,
    pub source: Option<URect>,
    pub rotation: u16,
    pub flip_x: bool,
    pub flip_y: bool,
    pub pivot: Option<Vec2>,
}

pub type SpriteMaterialRef = Rc<SpriteMaterial>;

#[derive(Debug)]
pub struct Sprite {
    pub position: Vec3,
    pub material: SpriteMaterialRef,
    pub params: SpriteParams,
}

pub type RenderPipelineRef = Rc<RenderPipeline>;

#[derive(Debug)]
pub struct SpriteMaterial {
    pub bind_group: BindGroup,
    pub render_pipeline: RenderPipelineRef,
}
