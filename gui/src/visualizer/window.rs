use std::mem;
use std::path::PathBuf;
use bytemuck::Pod;
use bytemuck::Zeroable;
use cgmath::Matrix4;
use egui::FontDefinitions;
use egui_gizmo::GizmoMode;
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform::{Platform, PlatformDescriptor};
use nalgebra::Vector3;
use wgpu::RenderPipelineDescriptor;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use moldyn_core::{DataFile, Particle, ParticleDatabase};
use moldyn_solver::initializer::UnitCell;
use crate::visualizer::camera::Camera;
use crate::visualizer::camera_controller::CameraController;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use crate::visualizer::imgui::main_window_ui;

const WIDTH: u32 = 1920;
const HEIGHT: u32= 1080;
const PARTICLE_COUNT: usize = 512;

const VERTICES: &[[f32; 3]] = &[[0.000000, 1.000000, 0.000000], [0.894427, 0.447214, 0.000000], [0.276393, 0.447214, 0.850651], [-0.723607, 0.447214, 0.525731], [-0.723607, 0.447214, -0.525731], [0.276393, 0.447214, -0.850651], [0.723607, -0.447214, -0.525731], [0.723607, -0.447214, 0.525731], [-0.276393, -0.447214, 0.850651], [-0.894427, -0.447214, 0.000000], [-0.276393, -0.447214, -0.850651], [0.000000, -1.000000, 0.000000], [0.111471, 0.932671, 0.343074], [0.207932, 0.739749, 0.639950], [-0.291836, 0.932671, 0.212031], [-0.544374, 0.739749, 0.395511], [-0.291836, 0.932671, -0.212031], [-0.544374, 0.739749, -0.395511], [0.111471, 0.932671, -0.343074], [0.207932, 0.739749, -0.639950], [0.672883, 0.739749, 0.000000], [0.360729, 0.932671, 0.000000], [0.568661, 0.516806, 0.639950], [0.784354, 0.516806, 0.343074], [-0.432902, 0.516806, 0.738584], [-0.083904, 0.516806, 0.851981], [-0.836210, 0.516806, -0.183479], [-0.836210, 0.516806, 0.183479], [-0.083904, 0.516806, -0.851981], [-0.432902, 0.516806, -0.738584], [0.784354, 0.516806, -0.343074], [0.568661, 0.516806, -0.639950], [0.964719, 0.156077, -0.212031], [0.905103, -0.156077, -0.395511], [0.499768, 0.156077, 0.851981], [0.655845, -0.156077, 0.738584], [-0.655845, 0.156077, 0.738584], [-0.499768, -0.156077, 0.851981], [-0.905103, 0.156077, -0.395511], [-0.964719, -0.156077, -0.212031], [0.096461, 0.156077, -0.983024], [-0.096461, -0.156077, -0.983024], [0.655845, -0.156077, -0.738584], [0.499768, 0.156077, -0.851981], [0.905103, -0.156077, 0.395511], [0.964719, 0.156077, 0.212031], [-0.096461, -0.156077, 0.983024], [0.096461, 0.156077, 0.983024], [-0.964719, -0.156077, 0.212031], [-0.905103, 0.156077, 0.395511], [-0.499768, -0.156077, -0.851981], [-0.655845, 0.156077, -0.738584], [0.432902, -0.516806, -0.738584], [0.083904, -0.516806, -0.851981], [0.836210, -0.516806, 0.183479], [0.836210, -0.516806, -0.183479], [0.083904, -0.516806, 0.851981], [0.432902, -0.516806, 0.738584], [-0.784354, -0.516806, 0.343074], [-0.568661, -0.516806, 0.639950], [-0.568661, -0.516806, -0.639950], [-0.784354, -0.516806, -0.343074], [-0.111471, -0.932671, -0.343074], [-0.207932, -0.739749, -0.639950], [0.544374, -0.739749, -0.395511], [0.291836, -0.932671, -0.212031], [0.544374, -0.739749, 0.395511], [0.291836, -0.932671, 0.212031], [-0.207932, -0.739749, 0.639950], [-0.111471, -0.932671, 0.343074], [-0.672883, -0.739749, 0.000000], [-0.360729, -0.932671, 0.000000], [0.487677, 0.789079, 0.373531], [-0.204548, 0.789079, 0.579236], [-0.614095, 0.789079, -0.015543], [-0.174983, 0.789079, -0.588843], [0.505950, 0.789079, -0.348381], [0.802301, 0.196377, -0.563693], [0.784028, 0.196377, 0.588842], [-0.317744, 0.196377, 0.927618], [-0.980405, 0.196377, -0.015543], [-0.288179, 0.196377, -0.937224], [0.317744, -0.196377, -0.927618], [0.980405, -0.196377, 0.015543], [0.288179, -0.196377, 0.937224], [-0.802301, -0.196377, 0.563693], [-0.784028, -0.196377, -0.588842], [0.204548, -0.789079, -0.579236], [0.614095, -0.789079, 0.015543], [0.174983, -0.789079, 0.588842], [-0.505950, -0.789079, 0.348381], [-0.487677, -0.789079, -0.373531], ];
const INDICES: &[u16] = &[0, 12, 21, 2, 22, 13, 1, 20, 23, 13, 72, 12, 23, 72, 22, 21, 72, 20, 12, 72, 21, 22, 72, 13, 20, 72, 23, 0, 14, 12, 3, 24, 15, 2, 13, 25, 15, 73, 14, 25, 73, 24, 12, 73, 13, 14, 73, 12, 24, 73, 15, 13, 73, 25, 0, 16, 14, 4, 26, 17, 3, 15, 27, 17, 74, 16, 27, 74, 26, 14, 74, 15, 16, 74, 14, 26, 74, 17, 15, 74, 27, 0, 18, 16, 5, 28, 19, 4, 17, 29, 19, 75, 18, 29, 75, 28, 16, 75, 17, 18, 75, 16, 28, 75, 19, 17, 75, 29, 0, 21, 18, 1, 30, 20, 5, 19, 31, 20, 76, 21, 31, 76, 30, 18, 76, 19, 21, 76, 18, 30, 76, 20, 19, 76, 31, 5, 31, 43, 1, 32, 30, 6, 42, 33, 30, 77, 31, 33, 77, 32, 43, 77, 42, 31, 77, 43, 32, 77, 30, 42, 77, 33, 1, 23, 45, 2, 34, 22, 7, 44, 35, 22, 78, 23, 35, 78, 34, 45, 78, 44, 23, 78, 45, 34, 78, 22, 44, 78, 35, 2, 25, 47, 3, 36, 24, 8, 46, 37, 24, 79, 25, 37, 79, 36, 47, 79, 46, 25, 79, 47, 36, 79, 24, 46, 79, 37, 3, 27, 49, 4, 38, 26, 9, 48, 39, 26, 80, 27, 39, 80, 38, 49, 80, 48, 27, 80, 49, 38, 80, 26, 48, 80, 39, 4, 29, 51, 5, 40, 28, 10, 50, 41, 28, 81, 29, 41, 81, 40, 51, 81, 50, 29, 81, 51, 40, 81, 28, 50, 81, 41, 5, 43, 40, 6, 52, 42, 10, 41, 53, 42, 82, 43, 53, 82, 52, 40, 82, 41, 43, 82, 40, 52, 82, 42, 41, 82, 53, 1, 45, 32, 7, 54, 44, 6, 33, 55, 44, 83, 45, 55, 83, 54, 32, 83, 33, 45, 83, 32, 54, 83, 44, 33, 83, 55, 2, 47, 34, 8, 56, 46, 7, 35, 57, 46, 84, 47, 57, 84, 56, 34, 84, 35, 47, 84, 34, 56, 84, 46, 35, 84, 57, 3, 49, 36, 9, 58, 48, 8, 37, 59, 48, 85, 49, 59, 85, 58, 36, 85, 37, 49, 85, 36, 58, 85, 48, 37, 85, 59, 4, 51, 38, 10, 60, 50, 9, 39, 61, 50, 86, 51, 61, 86, 60, 38, 86, 39, 51, 86, 38, 60, 86, 50, 39, 86, 61, 10, 53, 63, 6, 64, 52, 11, 62, 65, 52, 87, 53, 65, 87, 64, 63, 87, 62, 53, 87, 63, 64, 87, 52, 62, 87, 65, 6, 55, 64, 7, 66, 54, 11, 65, 67, 54, 88, 55, 67, 88, 66, 64, 88, 65, 55, 88, 64, 66, 88, 54, 65, 88, 67, 7, 57, 66, 8, 68, 56, 11, 67, 69, 56, 89, 57, 69, 89, 68, 66, 89, 67, 57, 89, 66, 68, 89, 56, 67, 89, 69, 8, 59, 68, 9, 70, 58, 11, 69, 71, 58, 90, 59, 71, 90, 70, 68, 90, 69, 59, 90, 68, 70, 90, 58, 69, 90, 71, 9, 61, 70, 10, 63, 60, 11, 71, 62, 60, 91, 61, 62, 91, 63, 70, 91, 71, 61, 91, 70, 63, 91, 60, 71, 91, 62, ];

const BOX_VERTICES: &[[f32; 3]] = &[
    [0.0, 1.0, 1.0],
    [0.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [1.0, 0.0, 1.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0],
    [1.0, 1.0, 0.0],
    [1.0, 0.0, 0.0],
];
const BOX_INDICES: &[u16] = &[
    0, 2, 3, 0, 3, 1,
    2, 6, 7, 2, 7, 3,
    6, 4, 5, 6, 5, 7,
    4, 0, 1, 4, 1, 5,
    0, 4, 6, 0, 6, 2,
    1, 5, 7, 1, 7, 3,
];

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct ParticleDataLite {
    position: [f32; 4],
    velocity: [f32; 4],
    mass_radius_id: [f32; 4],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_pos: [f32; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BoundingBox {
    size: [f32; 4],
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VisualizationParameterType {
    Type = 0,
    Velocity = 1,
    Pressure = 2,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct VisualizationParameter {
    gradient_color_min: [f32; 4],
    gradient_color_mid: [f32; 4],
    gradient_color_max: [f32; 4],
    gradient_min_max: [f32; 4],
    visualization_parameter_type: [u32; 4],
}

#[derive(Debug, PartialEq)]
pub enum ChoosePlot {
    Energy,
    UnitEnergy,
    Temperature,
    Pressure,
}

pub struct MacroPlots {
    pub kinetic_energy: Vec<[f64; 2]>,
    pub potential_energy: Vec<[f64; 2]>,
    pub thermal_energy: Vec<[f64; 2]>,
    pub full_energy: Vec<[f64; 2]>,
    pub internal_energy: Vec<[f64; 2]>,
    pub unit_kinetic_energy: Vec<[f64; 2]>,
    pub unit_potential_energy: Vec<[f64; 2]>,
    pub unit_thermal_energy: Vec<[f64; 2]>,
    pub unit_full_energy: Vec<[f64; 2]>,
    pub unit_internal_energy: Vec<[f64; 2]>,
    pub pressure: Vec<[f64; 2]>,
    pub temperature: Vec<[f64; 2]>,
}

pub struct UiData {
    pub color_0: [u8; 3],
    pub color_05: [u8; 3],
    pub color_1: [u8; 3],
    pub gradient_min: f64,
    pub gradient_max: f64,
    pub frame_index: usize,
    pub play: bool,
    pub play_speed: usize,
    pub file_path: Option<PathBuf>,
    pub loaded_frames: [usize; 2],
    pub last_frame_from_all: usize,
    pub files: Vec<usize>,
    pub visualization_parameter_type: VisualizationParameterType,
    pub macro_plots: Option<MacroPlots>,
    pub show_plot: bool,
    pub plot_to_draw: ChoosePlot,
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Window,
    camera: Camera,
    camera_controller: CameraController,
    ui_data: UiData,
    visualization_parameter_buffer: wgpu::Buffer,
    visualization_parameter_bind_group: wgpu::BindGroup,
    particles_data: Vec<ParticleDataLite>,
    particles_center: (f32, f32, f32),
    particles_bounding_box: BoundingBox,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
    render_pipeline: wgpu::RenderPipeline,
    bounding_box_buffer: wgpu::Buffer,
    bounding_box_index_buffer: wgpu::Buffer,
    bounding_box_uniform_buffer: wgpu::Buffer,
    bounding_box_bind_group: wgpu::BindGroup,
    bounding_box_render_pipeline: wgpu::RenderPipeline,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    depth_texture_sampler: wgpu::Sampler,
    platform: Platform,
    egui_rpass: egui_wgpu_backend::RenderPass,
    _egui_demo: egui_demo_lib::DemoWindows,
    data_file: Option<DataFile>,
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
        .with_title("MolDyn")
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::platform::web::WindowExtWebSys;
        use winit::dpi::{PhysicalSize, LogicalSize};
        let mut width = WIDTH;
        let mut height = HEIGHT;
        web_sys::window().and_then(|win| {
            width = win.inner_width()
                .unwrap().as_f64().unwrap() as u32;
            height = win.inner_height()
                .unwrap().as_f64().unwrap() as u32;
            let factor = window.scale_factor();
            let logical = LogicalSize { width, height };
            let PhysicalSize { width, height }: PhysicalSize<u32> = logical.to_physical(factor);
            let width = width.min(2048);
            window.set_inner_size(PhysicalSize::new(width, height));
            Some(())
        });

        web_sys::window().and_then(|win| { win.document() })
            .and_then(|doc| {
                let dst = doc.get_element_by_id("visualizer")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    ParticleDatabase::add(0, "Argon", 66.335, 0.071);
    let mut state = State::new(window).await;
    let mut particles_state =
        moldyn_solver::initializer::initialize_particles(&[125],
 &(Vector3::new(5.0, 5.0, 5.0) * 3.338339))
            .unwrap();
    moldyn_solver::initializer::initialize_particles_position(UnitCell::U,
        &mut particles_state, 0, (0.0, 0.0, 0.0),
(5, 5, 5), 3.338339).expect("Can't init positions");
    moldyn_solver::initializer::initialize_velocities_maxwell_boltzmann(&mut particles_state, 273.0, 0);
    state.update_particle_state(&particles_state);
    event_loop.run(move |event, _, control_flow| {
        state.imgui_event(&event);
        let ctx = &state.platform.context();
        match event {
            Event::DeviceEvent {
                ref event,
                ..
            } => {
                state.camera_controller.process_events(&event, ctx.is_using_pointer());
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                #[cfg(target_arch = "wasm32")]
                state.camera_controller.process_window_events(event);
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
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                let (min_v, max_v) = particles_state.get_min_max_velocity(0);
                state.ui_data.gradient_min = min_v;
                state.ui_data.gradient_max = max_v;
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

impl MacroPlots {
    pub fn new() -> Self {
        Self {
            kinetic_energy: vec![],
            potential_energy: vec![],
            thermal_energy: vec![],
            full_energy: vec![],
            internal_energy: vec![],
            unit_kinetic_energy: vec![],
            unit_potential_energy: vec![],
            unit_thermal_energy: vec![],
            unit_full_energy: vec![],
            unit_internal_energy: vec![],
            pressure: vec![],
            temperature: vec![],
        }
    }
}

impl BoundingBox {
    pub fn from(state: &moldyn_core::State) -> Self {
        Self {
            size: [state.boundary_box.x as f32, state.boundary_box.y as f32, state.boundary_box.z as f32, 0.0],
        }
    }
}

impl ParticleDataLite {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ParticleDataLite>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                },
            ],
        }
    }

    pub fn from(particle: &Particle) -> ParticleDataLite {
        let position = [particle.position.x as f32, particle.position.y as f32, particle.position.z as f32, 1.0];
        let velocity = [particle.velocity.x as f32, particle.velocity.y as f32, particle.velocity.z as f32, 1.0];
        let mass_radius_id = [particle.mass as f32, particle.radius as f32, particle.id as f32, 0.0];
        ParticleDataLite {
            position,
            velocity,
            mass_radius_id,
        }
    }
}

fn vertex_desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    }
}

impl CameraUniform {
    pub fn from(camera: &Camera) -> CameraUniform {
        let view = Matrix4::look_to_rh(camera.eye, camera.forward, camera.up);
        let aspect = camera.width as f32 / camera.height as f32;
        let proj = cgmath::perspective(cgmath::Deg(camera.fovy), aspect, 0.01, 10000.0);
        CameraUniform {
            view_pos: [camera.eye.x, camera.eye.y, camera.eye.z, 1.0],
            view: view.into(),
            proj: proj.into(),
        }
    }
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

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

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
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
        ).await.unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
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
        let camera = Camera::new((-1.0, 0.0, 0.0), 70.0, (config.width, config.height));
        let camera_controller = CameraController::new(0.2);
        let camera_uniform = CameraUniform::from(&camera);
        let bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let ui_data = UiData {
            color_0: [0, 0, 0],
            color_05: [100, 100, 100],
            color_1: [255, 255, 255],
            gradient_min: 0.0,
            gradient_max: 1.0,
            frame_index: 0,
            play: false,
            play_speed: 1,
            file_path: None,
            loaded_frames: [0, 0],
            last_frame_from_all: 0,
            files: vec![],
            visualization_parameter_type: VisualizationParameterType::Velocity,
            macro_plots: None,
            show_plot: false,
            plot_to_draw: ChoosePlot::Energy,
        };
        let visualization_parameter = VisualizationParameter {
            gradient_color_min: [0.0, 0.0, 0.0, 1.0],
            gradient_color_mid: [0.5, 0.5, 0.5, 1.0],
            gradient_color_max: [1.0, 1.0, 1.0, 1.0],
            gradient_min_max: [0.0, 1.0, 0.0, 0.0],
            visualization_parameter_type: [1, 0, 0, 0],
        };
        let visualization_parameter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Visualization parameter buffer"),
            contents: bytemuck::cast_slice(&[visualization_parameter]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let visualization_parameter_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Visualization parameter bind group layout"),
            entries: &[
                bind_group_layout_entry.clone(),
            ],
        });
        let visualization_parameter_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Visualization parameter bind group"),
            layout: &visualization_parameter_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: visualization_parameter_buffer.as_entire_binding(),
            }],
        });
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    bind_group_layout_entry.clone(),
                ],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &visualization_parameter_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(
            &RenderPipelineDescriptor {
                label: Some("Render pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[vertex_desc(), ParticleDataLite::desc()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
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
                multiview: None,
            }
        );

        let particles_state =
            moldyn_solver::initializer::initialize_particles(
        &[PARTICLE_COUNT], &Vector3::zeros()).unwrap();
        let mut instances = vec![];
        for particle_type in &particles_state.particles {
            for particle in particle_type {
                instances.push(ParticleDataLite::from(particle));
            }
        }
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let bb_uniform_data = BoundingBox::from(&particles_state);
        let bounding_box_uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("BB Uniform Buffer"),
                contents: bytemuck::cast_slice(&[bb_uniform_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let bounding_box_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Bounding Box Uniform Bind Group Layout"),
                entries: &[
                    bind_group_layout_entry.clone(),
                ],
            }
        );
        let bounding_box_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("BB Uniform Bind Group"),
                layout: &bounding_box_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: bounding_box_uniform_buffer.as_entire_binding(),
                    }
                ],
            }
        );
        let bounding_box_render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("BB Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &bounding_box_bind_group_layout],
                push_constant_ranges: &[],
            }
        );
        let bb_shader = device.create_shader_module(wgpu::include_wgsl!("bounding_box.wgsl"));
        let bounding_box_render_pipeline = device.create_render_pipeline(
            &RenderPipelineDescriptor {
                label: Some("BB Render pipeline"),
                layout: Some(&bounding_box_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &bb_shader,
                    entry_point: "vs_main",
                    buffers: &[vertex_desc()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &bb_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            }
        );
        let bounding_box_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("BB Vertex Buffer"),
                contents: bytemuck::cast_slice(BOX_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let bounding_box_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("BB Index Buffer"),
                contents: bytemuck::cast_slice(BOX_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let (depth_texture, depth_texture_view, depth_texture_sampler) =
            Self::create_depth_texture(&device, &config, "depth texture");
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });
        let egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);
        let egui_demo = egui_demo_lib::DemoWindows::default();
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            camera,
            camera_controller,
            particles_data: vec![],
            particles_center: (
                bb_uniform_data.size[0] / 2.0,
                bb_uniform_data.size[1] / 2.0,
                bb_uniform_data.size[2] / 2.0),
            particles_bounding_box: bb_uniform_data,
            ui_data,
            visualization_parameter_buffer,
            visualization_parameter_bind_group,
            camera_buffer,
            camera_bind_group,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instance_count: 0,
            render_pipeline,
            bounding_box_buffer,
            bounding_box_index_buffer,
            bounding_box_uniform_buffer,
            bounding_box_bind_group,
            bounding_box_render_pipeline,
            depth_texture,
            depth_texture_view,
            depth_texture_sampler,
            platform,
            egui_rpass,
            _egui_demo: egui_demo,
            data_file: None,
        }
    }

    fn imgui_event(&mut self, event: &Event<()>) {
        self.platform.handle_event(&event);
    }

    fn update_particle_state (&mut self, state: &moldyn_core::State) {
        let mut data = vec![];
        let bb = state.boundary_box;
        let center = (bb.x as f32 / 2.0, bb.y as f32 / 2.0, bb.z as f32 / 2.0);
        for particle_type in &state.particles {
            for particle in particle_type {
                data.push(ParticleDataLite::from(particle));
            }
        }
        self.particles_data = data;
        self.particles_center = center;
        self.particles_bounding_box = BoundingBox::from(&state);
        self.update_instance_buffer();
        self.update_bounding_box_uniform_buffer();
    }

    fn update_visualization_parameter (&mut self) {
        let ui = &self.ui_data;
        let data = VisualizationParameter {
            gradient_color_min: [
                ui.color_0[0] as f32 / 255.0,
                ui.color_0[1] as f32 / 255.0,
                ui.color_0[2] as f32 / 255.0,
                1.0],
            gradient_color_mid: [
                ui.color_05[0] as f32 / 255.0,
                ui.color_05[1] as f32 / 255.0,
                ui.color_05[2] as f32 / 255.0,
                1.0],
            gradient_color_max: [
                ui.color_1[0] as f32 / 255.0,
                ui.color_1[1] as f32 / 255.0,
                ui.color_1[2] as f32 / 255.0,
                1.0],
            gradient_min_max: [
                ui.gradient_min as f32,
                ui.gradient_max as f32,
                0.0, 0.0
            ],
            visualization_parameter_type: [ui.visualization_parameter_type as u32, 0, 0, 0],
        };
        let source_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Visualization parameter buffer source"),
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
            }
        );
        let mut command_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Loader")
            });
        command_encoder.copy_buffer_to_buffer(&source_buffer, 0,
                                              &self.visualization_parameter_buffer, 0,
                                              mem::size_of::<VisualizationParameter>() as wgpu::BufferAddress);
        self.queue.submit(Some(command_encoder.finish()));
    }

    fn update_bounding_box_uniform_buffer (&mut self) {
        let source_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("BB Uniform Buffer"),
                contents: bytemuck::cast_slice(&[self.particles_bounding_box]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
            }
        );
        let mut command_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Loader")
            });
        command_encoder.copy_buffer_to_buffer(&source_buffer, 0,
                                              &self.bounding_box_uniform_buffer, 0,
                                              mem::size_of::<BoundingBox>() as wgpu::BufferAddress);
        self.queue.submit(Some(command_encoder.finish()));
    }

    fn update_instance_buffer (&mut self) {
        let instance_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&self.particles_data.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.instance_buffer = instance_buffer;
        self.instance_count = self.particles_data.len() as u32;
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            let (depth_texture, depth_texture_view, depth_texture_sampler) =
                Self::create_depth_texture(&self.device, &self.config, "depth texture");
            self.depth_texture = depth_texture;
            self.depth_texture_view = depth_texture_view;
            self.depth_texture_sampler = depth_texture_sampler;
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        let center = (self.particles_center.0, self.particles_center.1, self.particles_center.2);
        self.camera_controller.update_camera(&mut self.camera, center, self.config.width, self.config.height);
        self.load_camera_to_buffer();
        let index = self.ui_data.frame_index;
        if let Some(path) = &self.ui_data.file_path {
            if self.data_file.is_none() ||
                index < self.ui_data.loaded_frames[0] ||
                index > self.ui_data.loaded_frames[1]  {
                let file = self.ui_data.files.iter()
                    .filter( |x| **x >= index ).min().unwrap();
                let load_file = path.with_extension("")
                    .with_extension(format!("{}.json", file));
                let _ = self.data_file.insert(DataFile::load_from_file(&load_file));
                let df = self.data_file.as_ref().unwrap();
                let start = df.start_frame;
                let end = df.start_frame + df.frame_count - 1;
                self.ui_data.loaded_frames[0] = start;
                self.ui_data.loaded_frames[1] = end;
            }
        }
        if let Some(df) = &self.data_file {
            if index >= self.ui_data.loaded_frames[0] &&
                index <= self.ui_data.loaded_frames[1] {
                let state = &df.frames.get(&self.ui_data.frame_index).unwrap().into();
                let frames_count = self.ui_data.last_frame_from_all + 1;
                self.update_particle_state(state);
                self.update_instance_buffer();
                if self.ui_data.play {
                    self.ui_data.frame_index += self.ui_data.play_speed;
                    if self.ui_data.frame_index >= frames_count {
                        self.ui_data.frame_index = frames_count - 1;
                        self.ui_data.play = false;
                    }
                }
            }
        }
        self.update_visualization_parameter();
    }

    fn load_camera_to_buffer(&mut self) {
        let load_buffer = CameraUniform::from(&self.camera);
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
                                              mem::size_of::<CameraUniform>() as wgpu::BufferAddress);
        self.queue.submit(Some(command_encoder.finish()));
        source_buffer.destroy();
    }

    fn create_depth_texture (device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str)
            -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );
        (texture, view, sampler)
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
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
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }
                ),
            });
            let buffer_size = self.particles_data.len() as u32;
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.visualization_parameter_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..buffer_size);
            // Bounding box
            render_pass.set_pipeline(&self.bounding_box_render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.bounding_box_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.bounding_box_buffer.slice(..));
            render_pass.set_index_buffer(self.bounding_box_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..BOX_INDICES.len() as u32, 0, 0..1);
        }
        {
            self.platform.begin_frame();
            let ctx = &self.platform.context();
            /////
            main_window_ui(&mut self.ui_data, ctx);
            use egui_gizmo::Gizmo;
            egui::Area::new("Gizmo Area").show(ctx, |ui| {
                let camera = &self.camera;
                let camera_uniform = CameraUniform::from(camera);
                let view = camera_uniform.view;
                let proj = camera_uniform.proj;
                let s = self.particles_bounding_box.size.clone();
                let model = [
                    [s[0], 0.0, 0.0, 0.0,],
                    [0.0, s[1], 0.0, 0.0,],
                    [0.0, 0.0, s[2], 0.0,],
                    [0.0, 0.0, 0.0, 1.0,],
                ];
                let gizmo = Gizmo::new("My gizmo")
                    .view_matrix(view)
                    .projection_matrix(proj)
                    .model_matrix(model)
                    .mode(GizmoMode::Translate);
                gizmo.interact(ui);
            });
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