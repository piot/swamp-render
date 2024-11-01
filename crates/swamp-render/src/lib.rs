/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use int_math::{URect, UVec2, Vec2, Vec3};
use log::info;
use std::rc::Rc;
use std::sync::Arc;
use swamp_wgpu_sprites::{FVec4, Mx4, SpriteInfo};
use wgpu::{BindGroup, BindGroupLayout, RenderPass, RenderPipeline};

#[derive(Debug)]
pub struct Render {
    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,

    sprite_uniform_buffer: wgpu::Buffer,
    sprite_uniform_bind_group: BindGroup,

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

        let sprite_uniform_buffer =
            swamp_wgpu::create_uniform_buffer(&device, "uniform bind group");

        let sprite_uniform_bind_group_layout =
            swamp_wgpu::create_uniform_bind_group_layout(&device, "bind group");

        let sprite_uniform_bind_group = swamp_wgpu::create_uniform_bind_group(
            &device,
            &sprite_uniform_bind_group_layout,
            &sprite_uniform_buffer,
            "bind_group",
        );

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
            sprite_uniform_buffer,
            sprite_uniform_bind_group,
        }
    }

    pub fn render_sprite(
        &mut self,
        position: Vec3,
        atlas_rect: URect,
        material: &SpriteMaterialRef,
        params: SpriteParams,
    ) {
        self.sprites.push(Sprite {
            position,
            atlas_rect,
            material: Rc::clone(material),
            params,
        })
    }

    pub fn render_sprite_2d(
        &mut self,
        position: Vec2,
        atlas_rect: URect,
        material: &SpriteMaterialRef,
        params: SpriteParams,
    ) {
        self.sprites.push(Sprite {
            position: position.into(),
            atlas_rect,
            material: Rc::clone(material),
            params,
        })
    }

    pub fn render(&mut self, render_pass: &mut RenderPass) {
        sort_sprites_by_z_then_y(&mut self.sprites);

        // -------- Batches
        let mut material_batches: Vec<Vec<&Sprite>> = Vec::new();
        let mut current_batch: Vec<&Sprite> = Vec::new();
        let mut current_material: Option<&SpriteMaterialRef> = None;

        for sprite in &self.sprites {
            if Some(&sprite.material) != current_material {
                if !current_batch.is_empty() {
                    material_batches.push(current_batch.clone());
                    current_batch.clear();
                }
                current_material = Some(&sprite.material);
            }
            current_batch.push(sprite);
        }

        if !current_batch.is_empty() {
            material_batches.push(current_batch);
        }
        // ---------------

        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        let num_indices = swamp_wgpu_sprites::INDICES.len() as u32;

        for batch in material_batches {
            if let Some(first_sprite) = batch.first() {
                let sprite_texture_bind_group = &first_sprite.material.bind_group;
                render_pass.set_bind_group(1, sprite_texture_bind_group, &[]); // sets sampler and texture
            }

            for sprite in batch {
                let model_matrix =
                    Mx4::from_translation(sprite.position.x.into(), sprite.position.y.into(), 0.0)
                        * Mx4::from_scale(
                            sprite.atlas_rect.size.x.into(),
                            sprite.atlas_rect.size.y.into(),
                            1.0,
                        );

                let atlas = sprite.atlas_rect;

                let tex_coords = FVec4([
                    atlas.position.x.into(),
                    atlas.position.y.into(),
                    (atlas.position.x + atlas.size.x).into(),
                    (atlas.position.y + atlas.size.y).into(),
                ]);

                let sprite_uniform =
                    swamp_wgpu_sprites::SpriteUniform::new(model_matrix, tex_coords);

                // Data will be copied, so we can reuse the same buffer
                self.queue.write_buffer(
                    &self.sprite_uniform_buffer,
                    0,
                    bytemuck::cast_slice(&[sprite_uniform]),
                );

                render_pass.set_bind_group(0, &self.sprite_uniform_bind_group, &[]);

                render_pass.draw_indexed(0..num_indices, 0, 0..1);
            }
        }

        /*
        for sprite in &self.sprites {
            let sprite_texture_bind_group = &sprite.material.bind_group;
            render_pass.set_bind_group(0, sprite_texture_bind_group, &[]); // sets sampler and texture



            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }
        */

        self.sprites.clear();
    }

    pub fn create_material_png(&mut self, png: &[u8], label: &str) -> SpriteMaterialRef {
        let texture =
            swamp_wgpu_sprites::load_texture_from_memory(&self.device, &self.queue, png, label);
        info!("loaded texture!");

        let bind_group = swamp_wgpu::create_texture_and_sampler_bind_group(
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
    pub dest_size: Option<UVec2>,
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
    pub atlas_rect: URect,
    pub material: SpriteMaterialRef,
    pub params: SpriteParams,
}

pub type RenderPipelineRef = Rc<RenderPipeline>;

#[derive(Debug, PartialEq, Eq)]
pub struct SpriteMaterial {
    pub bind_group: BindGroup,
    pub render_pipeline: RenderPipelineRef,
}
