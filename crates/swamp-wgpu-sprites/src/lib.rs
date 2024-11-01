use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupLayout, Buffer, PipelineLayout, RenderPipeline, Sampler, ShaderModule, TextureFormat,
};

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

        let bind_group_layout =
            swamp_wgpu::create_bind_group_layout(device, "sprite bind group layout");
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
