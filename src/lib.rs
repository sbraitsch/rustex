mod data;
mod pipelines;

use data::{
    hexagon::{self, generate_hexagon_vertices},
    quad, triangle,
    vertex::Vertex,
};
use log::warn;
use pipelines::edge_pipeline;
use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlCanvasElement};
use wgpu::util::DeviceExt;
use wgpu::MemoryHints;
use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

struct State<'a> {
    surface: wgpu::Surface<'a>,
    node_data: Vec<Vertex>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    clear_color: wgpu::Color,
    size: winit::dpi::PhysicalSize<u32>,
    polygon_pipeline: wgpu::RenderPipeline,
    edge_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
    num_indices: u32,
    cursor_pos: (f32, f32),
    window: &'a Window,
    overlay: &'a Element,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window, overlay: &'a Element) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    label: None,
                    memory_hints: MemoryHints::default(),
                },
                None, // Trace path
            )
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
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let clear_color = wgpu::Color::BLACK;
        let polygon_pipeline = pipelines::polygon_pipeline::create(&device, &config);
        let edge_pipeline = pipelines::edge_pipeline::create(&device, &config);

        let indices = hexagon::INDICES;
        let vertices: Vec<Vertex> = vec![];

        let num_vertices = vertices.len() as u32;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_indices = indices.len() as u32;
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            surface,
            node_data: Vec::new(),
            device,
            queue,
            config,
            clear_color,
            size,
            polygon_pipeline,
            edge_pipeline,
            vertex_buffer,
            index_buffer,
            num_vertices,
            num_indices,
            cursor_pos: (0.0, 0.0),
            window,
            overlay,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                if state.is_pressed() {
                    // self.vertex_buffer.unmap();
                    self.node_data
                        .push(Vertex::new(self.cursor_pos.0, self.cursor_pos.1));
                    self.vertex_buffer =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Vertex Buffer"),
                                contents: bytemuck::cast_slice(self.node_data.as_slice()),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                }
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.overlay
                    .set_text_content(Some(&format!("({:.3}|{:.3})", position.x, position.y)));
                let center_x = self.size.width as f64 / 2.0;
                let center_y = self.size.height as f64 / 2.0;
                self.cursor_pos = (
                    ((position.x - center_x) / center_x) as f32,
                    -((position.y - center_y) / center_y) as f32,
                );
                self.clear_color = wgpu::Color {
                    r: position.x / self.size.width as f64,
                    g: position.y / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Polygon Node Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let num_vert = self.node_data.len() as u32;

            render_pass.set_pipeline(&self.polygon_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..num_vert * 6, 0..num_vert);

            render_pass.set_pipeline(&self.edge_pipeline);
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            render_pass.draw(0..num_vert, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

    let event_loop = EventLoop::new().unwrap();

    use winit::platform::web::WindowBuilderExtWebSys;

    let (window, overlay) = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let canvas = doc
                .get_element_by_id("rustex")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();
            let window = WindowBuilder::new()
                .with_canvas(Some(canvas))
                .with_title("Rustex")
                .build(&event_loop)
                .unwrap();
            Some((window, doc.get_element_by_id("coords").unwrap()))
        })
        .unwrap();

    let mut state = State::new(&window, &overlay).await;
    let mut surface_configured = false;

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            surface_configured = true;
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.window().request_redraw();

                            if !surface_configured {
                                return;
                            }

                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => state.window().request_redraw(),
            _ => {}
        })
        .unwrap();
}
