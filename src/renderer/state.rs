use std::sync::Arc;
use wgpu::{include_wgsl, util::DeviceExt};
use winit::window::Window;

use crate::{
    renderer::{load_binary, pipeline::create_render_pipeline, texture},
    wad::{Patch, Wad},
};

pub struct RenderState {
    window: Arc<Window>,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    queue: wgpu::Queue,
    primary_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::DepthTexture,
    palette: texture::Palette,
    color_map: texture::ColorMap,
    textures: Vec<texture::PalettizedTexture>,
    index_buffer: wgpu::Buffer,
}

impl RenderState {
    pub async fn new(window: Arc<Window>, wad: &Wad) -> RenderState {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::wgt::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::IMMEDIATES,
                required_limits: wgpu::Limits {
                    max_immediate_size: 4,
                    ..Default::default()
                },
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let primary_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[
                    &texture::PALETTE_BIND_GROUP_LAYOUT.get_or_init(|| {
                        device.create_bind_group_layout(&texture::PALETTE_BIND_GROUP_LAYOUT_DESC)
                    }),
                    &texture::PALETTE_INDEX_BIND_GROUP_LAYOUT.get_or_init(|| {
                        device.create_bind_group_layout(
                            &texture::PALETTE_INDEX_BIND_GROUP_LAYOUT_DESC,
                        )
                    }),
                    &texture::COLORMAP_BIND_GROUP_LAYOUT.get_or_init(|| {
                        device.create_bind_group_layout(
                            &texture::COLORMAP_BIND_GROUP_LAYOUT_DESC,
                        )
                    }),
                ],
                immediate_size: 4,
            });

        let primary_pipeline = create_render_pipeline(
            &device,
            &primary_pipeline_layout,
            config.format,
            &[],
            include_wgsl!("shaders/shader.wgsl"),
            wgpu::CompareFunction::Less,
            "Primary Pipeline",
        );

        let depth_texture = texture::DepthTexture::new(&device, size.width, size.height);

        let palette = texture::Palette::new(wad.get_palette(), &device, &queue);

        let color_map = texture::ColorMap::new(wad.get_colormap(), &device, &queue);

        let texture = texture::PalettizedTexture::new(
            "Title",
            Patch::new(&load_binary("TITLEPIC.lmp")),
            &device,
            &queue,
        );

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&[0 as u32, 1, 2, 3, 4, 5]),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            window,
            instance,
            adapter,
            device,
            surface,
            queue,
            primary_pipeline,
            depth_texture,
            palette,
            color_map,
            textures: vec![texture],
            index_buffer,
        }
    }

    pub fn render(&mut self) {
        self.window.request_redraw();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Base Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.primary_pipeline);
        render_pass.set_bind_group(0, &self.palette.bind_group, &[]);
        render_pass.set_bind_group(1, &self.textures[0].bind_group, &[]);
        render_pass.set_bind_group(2, &self.color_map.bind_group, &[]);
        render_pass.set_immediates(0, &[0u8; 4]);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..6, 0, 0..1);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let mut config = self
            .surface
            .get_configuration()
            .expect("Failed to get current surface config");
        config.width = width;
        config.height = height;
        self.surface.configure(&self.device, &config);

        self.depth_texture = texture::DepthTexture::new(&self.device, width, height);
    }
}
