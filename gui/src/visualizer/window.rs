#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::window::Window;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
// use crate::visualizer::camera::Camera;

use bytemuck::Pod;
use bytemuck::Zeroable;
use wgpu::util::DeviceExt;
use moldyn_core::Particle;

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct ParticleData {
    position: [f64; 3],
    velocity: [f64; 3],
    force: [f64; 3],
    potential: f64,
    mass: f64,
    id: u32,
    _padding: u32,
}

fn particle_data_from_particle(particle: &Particle) -> ParticleData {
    ParticleData {
        position: particle.position.as_slice().try_into().expect("Something went wrong"),
        velocity: particle.velocity.as_slice().try_into().expect("Something went wrong"),
        force: particle.force.as_slice().try_into().expect("Something went wrong"),
        potential: particle.potential,
        mass: particle.mass,
        id: particle.id as u32,
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
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    screen_texture: wgpu::Texture,
    particles_buffer: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
    render_to_screen_pipeline: wgpu::RenderPipeline,
    screen_bind_group: wgpu::BindGroup,
    compute_bind_group: wgpu::BindGroup,
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
    let window = WindowBuilder::new().build(&event_loop).unwrap();

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

fn create_compute_pipeline (device: &wgpu::Device,
                            _config: &wgpu::SurfaceConfiguration,
                            shader: &wgpu::ShaderModule,
                            compute_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline {
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
    pipeline
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
                    compute_bind_group_layout: &wgpu::BindGroupLayout) -> (wgpu::ComputePipeline, wgpu::RenderPipeline) {
    let render_shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
    let compute_shader = device.create_shader_module(wgpu::include_wgsl!("render_particles.wgsl"));
    let compute_pipeline = create_compute_pipeline(device, config, &compute_shader, compute_bind_group_layout);
    let render_pipeline = create_render_pipeline(device, config, &render_shader, texture_bind_group_layout);
    (compute_pipeline, render_pipeline)
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
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
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
                    }
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
        let particles_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("particles_buffer"),
                size: (std::mem::size_of::<ParticleData>() * 1000000) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
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
                    }
                ],
                label: Some("compute_bind_group"),
            }
        );
        let (compute_pipeline, render_to_screen_pipeline) = create_pipelines(&device, &config, &screen_bind_group_layout, &compute_bind_group_layout);
        let mut res = Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            screen_texture,
            particles_buffer,
            compute_pipeline,
            render_to_screen_pipeline,
            screen_bind_group,
            compute_bind_group,
        };
        let particles_state = moldyn_core::State::default();
        res.load_state_to_buffer(&particles_state);
        res
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

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn load_state_to_buffer (&mut self, state: &moldyn_core::State) {
        let load_buffer = particle_data_vector_from_state(state);
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
        { // Rendering particles to screen quad
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("MainRenderer Pass"),
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            let x = self.config.width;
            let y = self.config.height;
            compute_pass.dispatch_workgroups(x, y, 1);
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

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
