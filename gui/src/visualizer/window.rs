#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::window::Window;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const MAX_PARTICLES: usize = 1000;

use bytemuck::Pod;
use bytemuck::Zeroable;
use egui::FontDefinitions;
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use moldyn_core::Particle;
use crate::visualizer::camera::Camera;
use crate::visualizer::camera_controller::CameraController;

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct ParticleData {
    position: [f64; 4],
    velocity: [f64; 4],
    force: [f64; 4],
    potential_mass_id: [f64; 4],
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraData {
    eye: [f32; 4],
    forward: [f32; 4],
    right: [f32; 4],
    up: [f32; 4],
    fovx: f32,
    width: u32,
    height: u32,
    _padding: u32,
}

fn particle_data_from_particle(particle: &Particle) -> ParticleData {
    ParticleData {
        position: [particle.position.x, particle.position.y, particle.position.z, 1.0],
        velocity: [particle.velocity.x, particle.velocity.y, particle.velocity.z, 0.0],
        force: [particle.force.x, particle.force.y, particle.force.z, 0.0],
        potential_mass_id: [particle.potential, particle.mass, particle.id as f64, 0.0],
    }
}

fn camera_data_from_camera(camera: &Camera) -> CameraData {
    CameraData {
        eye: [camera.eye.x, camera.eye.y, camera.eye.z, 1.0],
        forward: [camera.forward.x, camera.forward.y, camera.forward.z, 0.0],
        right: [camera.right.x, camera.right.y, camera.right.z, 0.0],
        up: [camera.up.x, camera.up.y, camera.up.z, 0.0],
        fovx: camera.fovx,
        width: camera.width,
        height: camera.height,
        _padding: 0,
    }
}

fn particle_data_vector_from_state(state: &moldyn_core::State) -> Vec<ParticleData> {
    let mut res = vec![];
    for particle in &state.particles {
        let particle = particle.lock().expect("Cann't lock particle");
        let particle_data = particle_data_from_particle(&particle);
        res.push(particle_data);
    }
    res
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Window,
    screen_texture: wgpu::Texture,
    particles_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera: Camera,
    camera_controller: CameraController,
    clear_screen_pipeline: wgpu::ComputePipeline,
    compute_pipeline: wgpu::ComputePipeline,
    render_to_screen_pipeline: wgpu::RenderPipeline,
    screen_bind_group: wgpu::BindGroup,
    compute_bind_group: wgpu::BindGroup,
    particle_count: u32,
    platform: Platform,
    egui_rpass: egui_wgpu_backend::RenderPass,
    egui_demo: egui_demo_lib::DemoWindows,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn visualizer_window() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("visualizer")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        state.imgui_event(&event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::DeviceEvent {
                ref event,
                ..
            } => {
                state.camera_controller.process_events(&event);
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}

fn create_compute_pipelines (device: &wgpu::Device,
                            clear_shader: &wgpu::ShaderModule,
                            shader: &wgpu::ShaderModule,
                            compute_bind_group_layout: &wgpu::BindGroupLayout) -> (wgpu::ComputePipeline, wgpu::ComputePipeline) {
    let compute_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute renderer Pipeline Layout"),
            bind_group_layouts: &[compute_bind_group_layout],
            push_constant_ranges: &[],
        });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("MainRenderer"),
        layout: Some(&compute_pipeline_layout),
        module: &shader,
        entry_point: "main",
    });
    let clear_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("ClearTexture"),
        layout: Some(&compute_pipeline_layout),
        module: &clear_shader,
        entry_point: "main",
    });
    (clear_pipeline, pipeline)
}

fn create_render_pipeline (device: &wgpu::Device,
                           config: &wgpu::SurfaceConfiguration,
                           shader: &wgpu::ShaderModule,
                           texture_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline {
    let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[texture_bind_group_layout],
            push_constant_ranges: &[],
        });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });
    render_pipeline
}

fn create_pipelines(device: &wgpu::Device,
                    config: &wgpu::SurfaceConfiguration,
                    texture_bind_group_layout: &wgpu::BindGroupLayout,
                    compute_bind_group_layout: &wgpu::BindGroupLayout) -> (wgpu::ComputePipeline, wgpu::ComputePipeline, wgpu::RenderPipeline) {
    let render_shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
    let clear_shader = device.create_shader_module(wgpu::include_wgsl!("clear_texture.wgsl"));
    let compute_shader = device.create_shader_module(wgpu::include_wgsl!("render_particles.wgsl"));
    let (clear_pipeline, compute_pipeline) = create_compute_pipelines(device, &clear_shader, &compute_shader, compute_bind_group_layout);
    let render_pipeline = create_render_pipeline(device, config, &render_shader, texture_bind_group_layout);
    (clear_pipeline, compute_pipeline, render_pipeline)
}

async fn create_base_objects (window: &Window)
            -> (wgpu::Surface, wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });

    // # Safety
    //
    // The surface needs to live as long as the window that created it.
    // State owns the window so this should be safe.
    let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
                features: wgpu::Features::SHADER_F64,
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        )
        .await
        .unwrap();
    (surface, adapter, device, queue)
}

fn create_screen_texture (device: &wgpu::Device,
                          config: &wgpu::SurfaceConfiguration,
                          particles_buffer: &wgpu::Buffer,
                          camera_buffer: &wgpu::Buffer)
    -> (wgpu::Texture, wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroupLayout, wgpu::BindGroupLayout) {
    let screen_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("screen_texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    let screen_texture_view = screen_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let screen_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let screen_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
    let compute_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("compute_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
    let screen_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &screen_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&screen_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&screen_sampler),
                }
            ],
            label: Some("screen_bind_group"),
        }
    );
    let compute_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particles_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&screen_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("compute_bind_group"),
        }
    );
    (screen_texture, screen_bind_group, compute_bind_group, screen_bind_group_layout, compute_bind_group_layout)
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let (surface, adapter, device, queue) = create_base_objects(&window).await;
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let particles_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("particles_buffer"),
                size: (std::mem::size_of::<ParticleData>() * MAX_PARTICLES) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        let camera_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("camera_buffer"),
                size: std::mem::size_of::<CameraData>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });
        let (screen_texture,
            screen_bind_group, compute_bind_group,
            screen_bind_group_layout, compute_bind_group_layout) =
            create_screen_texture(&device, &config, &particles_buffer, &camera_buffer);
        // We use the egui_wgpu_backend crate as the render backend.
        let egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);
        let egui_demo = egui_demo_lib::DemoWindows::default();
        let (clear_screen_pipeline, compute_pipeline, render_to_screen_pipeline) =
            create_pipelines(&device, &config, &screen_bind_group_layout, &compute_bind_group_layout);
        let camera = Camera::new((-1.0, 0.0, 0.0), 90.0, (size.width, size.height));
        let camera_controller = CameraController::new(0.1);
        let mut res = Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            screen_texture,
            particles_buffer,
            camera_buffer,
            camera,
            camera_controller,
            clear_screen_pipeline,
            compute_pipeline,
            render_to_screen_pipeline,
            screen_bind_group,
            compute_bind_group,
            particle_count: 0,
            platform,
            egui_rpass,
            egui_demo,
        };
        let particles_state = moldyn_core::State::default();
        res.load_state_to_buffer(&particles_state);
        res.load_camera_to_buffer();
        res
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn imgui_event(&mut self, event: &Event<()>) {
        self.platform.handle_event(&event);
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.camera.update((self.camera.eye.x, self.camera.eye.y, self.camera.eye.z),
                               (self.camera.forward.x, self.camera.forward.y, self.camera.forward.z),
                               self.config.width,
                               self.config.height);
            self.load_camera_to_buffer();
            self.surface.configure(&self.device, &self.config);
            // Recreate texture with new sizes
            let (screen_texture,
                screen_bind_group, compute_bind_group,
                screen_bind_group_layout, compute_bind_group_layout) =
                create_screen_texture(&self.device, &self.config, &self.particles_buffer, &self.camera_buffer);
            self.screen_texture = screen_texture;
            self.screen_bind_group = screen_bind_group;
            self.compute_bind_group = compute_bind_group;
            let (clear_screen_pipeline, compute_pipeline, render_to_screen_pipeline) =
                create_pipelines(&self.device, &self.config, &screen_bind_group_layout, &compute_bind_group_layout);
            self.clear_screen_pipeline = clear_screen_pipeline;
            self.compute_pipeline = compute_pipeline;
            self.render_to_screen_pipeline = render_to_screen_pipeline;
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn load_state_to_buffer (&mut self, state: &moldyn_core::State) {
        let load_buffer = particle_data_vector_from_state(state);
        self.particle_count = state.particles.len() as u32;
        let source_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("State data load buffer"),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
            contents: bytemuck::cast_slice(&load_buffer),
        });
        let mut command_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
            label: Some("Loader")
        });
        command_encoder.copy_buffer_to_buffer(&source_buffer, 0,
                                              &self.particles_buffer, 0,
                                              source_buffer.size());
        self.queue.submit(Some(command_encoder.finish()));
        source_buffer.destroy();
    }

    fn load_camera_to_buffer(&mut self) {
        let load_buffer = camera_data_from_camera(&self.camera);
        let source_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera data load buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
            contents: bytemuck::cast_slice(&[load_buffer]),
        });
        let mut command_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Loader")
            });
        command_encoder.copy_buffer_to_buffer(&source_buffer, 0,
                                              &self.camera_buffer, 0,
                                              std::mem::size_of::<CameraData>() as wgpu::BufferAddress);
        self.queue.submit(Some(command_encoder.finish()));
        source_buffer.destroy();
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera, self.config.width, self.config.height);
        self.load_camera_to_buffer();
    }

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
        { // Clear texture state
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Clear texture Pass"),
            });
            compute_pass.set_pipeline(&self.clear_screen_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            let x = self.config.width;
            let y = self.config.height;
            compute_pass.dispatch_workgroups(x, y, 1);
        }
        { // Rendering particles to screen quad
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("MainRenderer Pass"),
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            let x = self.config.width;
            let y = self.config.height;
            compute_pass.dispatch_workgroups(x, y, self.particle_count);
        }
        { // Rendering screen quad
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_to_screen_pipeline);
            render_pass.set_bind_group(0, &self.screen_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        {
            self.platform.begin_frame();
            /////
            self.egui_demo.ui(&self.platform.context());
            ////
            // End the UI frame. We could now handle the output and draw the UI with the backend.
            let full_output = self.platform.end_frame(Some(&self.window));
            let paint_jobs = self.platform.context().tessellate(full_output.shapes);

            // Upload all resources for the GPU.
            let screen_descriptor = ScreenDescriptor {
                physical_width: self.config.width,
                physical_height: self.config.height,
                scale_factor: self.window.scale_factor() as f32,
            };
            let tdelta: egui::TexturesDelta = full_output.textures_delta;
            self.egui_rpass
                .add_textures(&self.device, &self.queue, &tdelta)
                .expect("Something went wrong");
            self.egui_rpass.update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

            // Record all render passes.
            self.egui_rpass
                .execute(
                    &mut encoder,
                    &view,
                    &paint_jobs,
                    &screen_descriptor,
                    None,
                )
                .unwrap();

        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector3;
    use moldyn_core::{Particle, ParticleDatabase, State};
    use crate::visualizer::window::{particle_data_from_particle, particle_data_vector_from_state};

    #[test]
    fn particle_data_converter () {
        ParticleDatabase::add(0, "test_particle", 1.0);
        let particle = Particle::new(0,
                                     Vector3::new(1.0, 2.0, 3.0),
                                     Vector3::new(4.0, 5.0, 6.0)).expect("Can't create particle");
        let pd = particle_data_from_particle(&particle);
        assert_eq!(pd.potential_mass_id[1], 1.0);
        assert_eq!(pd.potential_mass_id[2], 0.0);
        assert_eq!(pd.position[0], 1.0);
        assert_eq!(pd.position[1], 2.0);
        assert_eq!(pd.position[2], 3.0);
        assert_eq!(pd.velocity[0], 4.0);
        assert_eq!(pd.velocity[1], 5.0);
        assert_eq!(pd.velocity[2], 6.0);
    }

    #[test]
    fn particle_data_converter_default () {
        let particle = Particle::default();
        let pd = particle_data_from_particle(&particle);
        assert_eq!(pd.potential_mass_id[1], 1.0);
        assert_eq!(pd.potential_mass_id[2], 0.0);
        assert_eq!(pd.position[0], 0.0);
        assert_eq!(pd.position[1], 0.0);
        assert_eq!(pd.position[2], 0.0);
        assert_eq!(pd.velocity[0], 0.0);
        assert_eq!(pd.velocity[1], 0.0);
        assert_eq!(pd.velocity[2], 0.0);
    }

    #[test]
    fn particle_state_converter () {
        let state = State::default();
        let state_data = particle_data_vector_from_state(&state);
        assert_eq!(state_data.len(), 2);
        let pd = &state_data[0];
        assert_eq!(pd.potential_mass_id[1], 3.0);
        assert_eq!(pd.potential_mass_id[2], 1.0);
        assert_eq!(pd.position[0], 0.0);
        assert_eq!(pd.position[1], 0.5);
        assert_eq!(pd.position[2], 0.0);
        assert_eq!(pd.velocity[0], 0.0);
        assert_eq!(pd.velocity[1], 0.0);
        assert_eq!(pd.velocity[2], 0.0);
        let pd = &state_data[1];
        assert_eq!(pd.potential_mass_id[1], 1.0);
        assert_eq!(pd.potential_mass_id[2], 0.0);
        assert_eq!(pd.position[0], 0.0);
        assert_eq!(pd.position[1], 0.0);
        assert_eq!(pd.position[2], 0.0);
        assert_eq!(pd.velocity[0], 0.0);
        assert_eq!(pd.velocity[1], 0.0);
        assert_eq!(pd.velocity[2], 0.0);
    }
}