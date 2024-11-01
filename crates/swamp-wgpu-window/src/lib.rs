use log::info;
use std::default::Default;
use std::sync::Arc;
use wgpu::{DeviceDescriptor, Features, MemoryHints, RenderPass, RequestDeviceError, SurfaceError};
use winit::window::Window;

#[derive(Debug)]
pub struct WgpuWindow<'a> {
    //instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

impl<'a> WgpuWindow<'a> {
    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }
}

impl WgpuWindow<'_> {
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub async fn new(window: Arc<Window>) -> Result<Self, RequestDeviceError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .unwrap();

        let device_descriptor = DeviceDescriptor {
            label: None,
            required_features: Features::empty(), // Specify features as needed
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            memory_hints: MemoryHints::default(), // Use default memory hints
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .expect("Failed to request device");
        info!("got a device {:?}", device);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let window_size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let present_mode = surface_caps.present_modes[0];
        let alpha_mode = surface_caps.alpha_modes[0];
        info!(
            "found surface format {:?} {:?} {:?}",
            surface_format, present_mode, alpha_mode
        );

        Ok(Self {
            surface,
            device: device.into(),
            queue: queue.into(),
            config,
            size: window_size,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(
        &mut self,
        mut render_fn: impl FnMut(&mut RenderPass, &wgpu::Device, &wgpu::Queue),
    ) -> Result<(), SurfaceError> {
        // Gets a new texture from the swap chain
        let surface_texture = self.surface.get_current_texture()?;
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // THIS SCOPE IS ABSOLUTELY NEEDED FOR THE RENDER PASS - DO NOT REMOVE
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.3,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_fn(&mut render_pass, &self.device, &self.queue);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        surface_texture.present();

        Ok(())
    }
}
