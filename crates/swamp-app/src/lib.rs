use async_trait::async_trait;
use log::info;
use std::fmt::Debug;
use std::sync::Arc;
use swamp_render::Render;
use swamp_wgpu_window::WgpuWindow;
use swamp_window::AppHandler;
use winit::dpi;
use winit::window::Window;

pub trait Application: Debug {
    fn init(&mut self, render: &mut Render);
    fn tick(&mut self);
    fn render(&mut self, render: &mut Render);
}

#[derive(Debug)]
pub struct App<'a> {
    main_render: Option<Render>,
    wgpu_window: Option<WgpuWindow<'a>>,
    app: &'a mut dyn Application,
    #[allow(unused)]
    title: String,
}

#[async_trait(?Send)]
impl<'a> AppHandler for App<'a> {
    fn create_window(&mut self, window: Arc<Window>) {
        info!("create window!");
        let wgpu_window = pollster::block_on(WgpuWindow::new(window)).expect("REASON");

        let vertex_shader_source = "
// Define a structure to hold the vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>, // Clip space position
    @location(1) tex_coords: vec2<f32>,      // Texture coordinates
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,  // Input vertex position
    @location(1) tex_coords: vec2<f32> // Input texture coordinates
) -> VertexOutput {
    // Create the output structure
    var output: VertexOutput;

    // Set the clip space position
    output.position = vec4<f32>(position, 0.0, 1.0); // Convert 2D position to 4D
    // Pass through the texture coordinates
    output.tex_coords = tex_coords;

    return output; // Return the output structure
}
        ";

        let fragment_shader_source = "
@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

@fragment
fn fs_main(@location(1) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    // Sample the texture with nearest filtering for hard pixel edges
    return textureSample(texture, texture_sampler, tex_coords);
}

";
        self.main_render = Some(Render::new(
            Arc::clone(wgpu_window.device()),
            Arc::clone(wgpu_window.queue()),
            wgpu_window.surface_config().format,
            vertex_shader_source,
            fragment_shader_source,
        ));

        self.wgpu_window = Some(wgpu_window);

        self.app.init(self.main_render.as_mut().unwrap())
    }

    fn resized(&mut self, physical_size: dpi::PhysicalSize<u32>) {
        info!("resized!");
        self.wgpu_window.as_mut().unwrap().resize(physical_size);
    }
    fn redraw(&mut self) {
        let main_render = self.main_render.as_mut().expect("REASON");
        self.app.render(main_render);
        self.wgpu_window
            .as_mut()
            .unwrap()
            .render(|render_pass, _, _| main_render.render(render_pass))
            .expect("TODO: panic message");
    }
}

impl<'a> App<'a> {
    pub fn new(title: &str, app: &'a mut impl Application) -> Self {
        Self {
            main_render: None,
            title: title.into(),
            app,
            wgpu_window: None,
        }
    }

    pub fn run(&mut self, title: &str) {
        let _ = swamp_window::WindowRunner::run_app(self, title);
    }
}
