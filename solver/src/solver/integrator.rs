use std::sync::RwLock;
use na::Vector3;
use wgpu::{BindGroupLayoutEntry, ComputePassDescriptor, InstanceDescriptor};
use wgpu::util::DeviceExt;
use moldyn_core::State;
use crate::initializer::{Barostat, Thermostat};
use crate::solver::update_force;

#[cfg(not(target_arch = "wasm32"))]
#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Particle {
    position: [f64; 4],
    velocity: [f64; 4],
    force: [f64; 4],
    potential_mass_radius_id: [f64; 4],
    temp: [f64; 4],
}

#[cfg(not(target_arch = "wasm32"))]
#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniform {
    boundary_conditions: [f64; 4],
    dt_count: [f64; 4],
}

pub enum Integrator {
    #[cfg(not(target_arch = "wasm32"))]
    VerletMethodGPU {
        particles_count: usize,
        device: wgpu::Device,
        queue: wgpu::Queue,
        input_buffer: wgpu::Buffer,
        output_buffer: wgpu::Buffer,
        staging_buffer: wgpu::Buffer,
        global_uniform_buffer: wgpu::Buffer,
        compute_bind_group: wgpu::BindGroup,
        step1: wgpu::ComputePipeline,
        step2: wgpu::ComputePipeline,
    },
    VerletMethod,
    Custom(String),
}

impl Particle {
    fn empty() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 1.0],
            velocity: [0.0, 0.0, 0.0, 0.0],
            force: [0.0, 0.0, 0.0, 0.0],
            potential_mass_radius_id: [0.0, 0.0, 0.0, 0.0],
            temp: [0.0, 0.0, 0.0, 0.0],
        }
    }

    fn from(particle: &moldyn_core::Particle) -> Self {
        Self {
            position: [particle.position.x, particle.position.y, particle.position.z, 1.0],
            velocity: [particle.velocity.x, particle.velocity.y, particle.velocity.z, 0.0],
            force: [particle.force.x, particle.force.y, particle.force.z, 0.0],
            potential_mass_radius_id: [particle.potential, particle.mass, particle.radius, particle.id as f64],
            temp: [particle.temp, 0.0, 0.0, 0.0],
        }
    }

    fn into(&self) -> moldyn_core::Particle {
        moldyn_core::Particle {
            position: Vector3::new(self.position[0], self.position[1], self.position[2]),
            velocity: Vector3::new(self.velocity[0], self.velocity[1], self.velocity[2]),
            force: Vector3::new(self.force[0], self.force[1], self.force[2]),
            potential: self.potential_mass_radius_id[0],
            temp: self.temp[0],
            mass: self.potential_mass_radius_id[1],
            radius: self.potential_mass_radius_id[2],
            id: self.potential_mass_radius_id[3] as u16,
        }
    }
}

impl Integrator {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn create_gpu_verlet(particles_count: usize) -> Self {
        todo!();
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            dx12_shader_compiler: Default::default(),
        });
        let adapter = instance.request_adapter(&Default::default()).await.unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: adapter.features(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();
        println!("{:?}", adapter.get_info());
        let shader = device.create_shader_module(wgpu::include_wgsl!("verlet_method.wgsl"));
        let particles: Vec<Particle> = (0..particles_count).map(|_| {
            Particle::empty()
        }).collect();
        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input particles buffer"),
            contents: bytemuck::cast_slice(particles.as_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output particles buffer"),
            size: input_buffer.size(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging buffer"),
            size: input_buffer.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let global_uniform = GlobalUniform {
            boundary_conditions: [0.0, 0.0, 0.0, 0.0],
            dt_count: [0.0, 0.0, 0.0, 0.0],
        };
        let global_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Boundary conditions buffer"),
            contents: bytemuck::cast_slice(&[global_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });
        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform_buffer.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: input_buffer.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buffer.as_entire_binding(),
            }],
        });
        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });
        let step1 = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Step 1 pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: "step1",
        });
        let step2 = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Step 2 pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: "step2",
        });

        Integrator::VerletMethodGPU {
            particles_count,
            device,
            queue,
            input_buffer,
            output_buffer,
            staging_buffer,
            global_uniform_buffer,
            compute_bind_group,
            step1,
            step2
        }
    }

    pub fn calculate(&self, state: &mut State, delta_time: f64,
                     mut barostat: Option<(&mut Barostat, f64)>, mut thermostat: Option<(&mut Thermostat, f64)>) {
        match self {
            Integrator::VerletMethod => {
                if let Some((barostat, target_pressure)) = barostat.as_mut() {
                    for particle_type in 0..state.particles.len() {
                        barostat.calculate_myu(&state, delta_time, particle_type as u16, *target_pressure);
                    }
                }
                if let Some((thermostat, target_temperature)) = thermostat.as_mut() {
                    for particle_type in 0..state.particles.len() {
                        thermostat.calculate_lambda(&state, delta_time, particle_type as u16, *target_temperature);
                    }
                }
                state.particles.iter_mut().for_each(|particle_type| {
                    particle_type.iter_mut().for_each(|particle| {
                        let mut particle = particle.write().expect("Can't lock particle");
                        let acceleration = particle.force / particle.mass;
                        let velocity = particle.velocity;
                        particle.position += velocity * delta_time + acceleration * delta_time * delta_time / 2.0;
                        particle.velocity += acceleration * delta_time / 2.0;
                    });
                });
                state.apply_boundary_conditions();
                if let Some((barostat, target_pressure)) = barostat.as_ref() {
                    for particle_type in 0..state.particles.len() {
                        barostat.update(state, delta_time, particle_type as u16, *target_pressure);
                    }
                }
                if let Some((thermostat, target_temperature)) = thermostat.as_ref() {
                    for particle_type in 0..state.particles.len() {
                        thermostat.update(state, delta_time, particle_type as u16, *target_temperature);
                    }
                }
                update_force(state);
                state.particles.iter_mut().for_each(|particle_type| {
                    particle_type.iter_mut().for_each(|particle| {
                        let mut particle = particle.write().expect("Can't lock particle");
                        let acceleration = particle.force / particle.mass;
                        particle.velocity += acceleration * delta_time / 2.0;
                    });
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            Integrator::VerletMethodGPU {
                particles_count,
                device,
                queue,
                step1,
                step2,
                compute_bind_group,
                input_buffer,
                output_buffer,
                staging_buffer,
                global_uniform_buffer,
            } => {
                let future = async {
                    if let Some((barostat, target_pressure)) = barostat.as_mut() {
                        for particle_type in 0..state.particles.len() {
                            barostat.calculate_myu(&state, delta_time, particle_type as u16, *target_pressure);
                        }
                    }
                    if let Some((thermostat, target_temperature)) = thermostat.as_mut() {
                        for particle_type in 0..state.particles.len() {
                            thermostat.calculate_lambda(&state, delta_time, particle_type as u16, *target_temperature);
                        }
                    }
                    state.particles.iter_mut().for_each(|particle_type| {
                        particle_type.iter_mut().for_each(|particle| {
                            let mut particle = particle.write().expect("Can't lock particle");
                            let acceleration = particle.force / particle.mass;
                            let velocity = particle.velocity;
                            particle.position += velocity * delta_time + acceleration * delta_time * delta_time / 2.0;
                            particle.velocity += acceleration * delta_time / 2.0;
                        });
                    });
                    state.apply_boundary_conditions();
                    if let Some((barostat, target_pressure)) = barostat.as_ref() {
                        for particle_type in 0..state.particles.len() {
                            barostat.update(state, delta_time, particle_type as u16, *target_pressure);
                        }
                    }
                    if let Some((thermostat, target_temperature)) = thermostat.as_ref() {
                        for particle_type in 0..state.particles.len() {
                            thermostat.update(state, delta_time, particle_type as u16, *target_temperature);
                        }
                    }

                    let mut particles = vec![];
                    let global_uniform = GlobalUniform {
                        boundary_conditions: [state.boundary_box.x, state.boundary_box.y, state.boundary_box.z, delta_time],
                        dt_count: [delta_time, particles_count.clone() as f64, 0.0, 0.0],
                    };
                    for particle_type in &state.particles {
                        for particle in particle_type {
                            let particle = particle.read().expect("Can't lock particle");
                            particles.push(Particle::from(&particle));
                        }
                    }
                    let p_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Buffer of particles to init "),
                        contents: bytemuck::cast_slice(particles.as_slice()),
                        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
                    });
                    let gu_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Buffer of bb to init "),
                        contents: bytemuck::cast_slice(&[global_uniform]),
                        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::UNIFORM,
                    });

                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Main command encoder"),
                    });
                    encoder.copy_buffer_to_buffer(&p_buffer, 0,
                                                          &input_buffer, 0,
                                                          p_buffer.size());
                    encoder.copy_buffer_to_buffer(&gu_buffer,0,
                                                          &global_uniform_buffer, 0,
                                                  gu_buffer.size());
                    // { // Step1
                    //     let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    //         label: Some("Step 1 compute pass"),
                    //     });
                    //     compute_pass.set_pipeline(&step1);
                    //     compute_pass.set_bind_group(0, &compute_bind_group, &[]);
                    //     compute_pass.dispatch_workgroups(particles_count.clone() as u32 / 8, 1, 1);
                    //
                    // }
                    { // Step2
                        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                            label: Some("Step 2 compute pass"),
                        });
                        compute_pass.set_pipeline(&step2);
                        compute_pass.set_bind_group(0, &compute_bind_group, &[]);
                        compute_pass.dispatch_workgroups(particles_count.clone() as u32 / 8, 1, 1);
                    }
                    encoder.copy_buffer_to_buffer(&output_buffer, 0,
                                                  &staging_buffer, 0,
                                                  output_buffer.size());
                    queue.submit(Some(encoder.finish()));
                    let buf_slice = staging_buffer.slice(..);
                    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
                    buf_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
                    device.poll(wgpu::Maintain::Wait);
                    if let Some(Ok(())) = receiver.receive().await {
                        let data_raw = &*buf_slice.get_mapped_range();
                        let data: &[Particle] = bytemuck::cast_slice(data_raw);
                        for particle_type in &mut state.particles {
                            particle_type.clear();
                        }
                        for particle in data {
                            let particle: moldyn_core::Particle = particle.into();
                            let particle_type = particle.id as usize;
                            state.particles[particle_type].push(RwLock::new(particle));
                        }
                    }
                    staging_buffer.unmap();

                    // update_force(state); // Step2
                    state.particles.iter_mut().for_each(|particle_type| {
                        particle_type.iter_mut().for_each(|particle| {
                            let mut particle = particle.write().expect("Can't lock particle");
                            let acceleration = particle.force / particle.mass;
                            particle.velocity += acceleration * delta_time / 2.0;
                        });
                    }); // Step3
                };
                pollster::block_on(future);
            }
            Integrator::Custom(_) => {
                todo!()
            }
        }
    }
}