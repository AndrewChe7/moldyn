use crate::ParticleDatabase;
use na::Vector3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Structure that keeps all data for particle
#[derive(Serialize, Deserialize)]
pub struct Particle {
    /// position of particle in 3d space
    pub position: Vector3<f64>,
    /// velocity of particle
    pub velocity: Vector3<f64>,
    /// The sum of the forces acting on the particle
    pub force: Vector3<f64>,
    /// The total potential of the particle
    pub potential: f64,
    /// Mass of particle
    pub mass: f64,
    /// Radius of particle
    pub radius: f64,
    /// ID of particle. Defines type of particle
    pub id: u16,
}

/// Structure that keeps current state
#[derive(Serialize, Deserialize)]
pub struct State {
    pub particles: Vec<Mutex<Particle>>,
    pub boundary_box: Vector3<f64>,
}

impl Particle {
    /// Create empty particle
    pub fn new(particle_id: u16, position: Vector3<f64>, velocity: Vector3<f64>) -> Option<Self> {
        ParticleDatabase::get_particle_mass(particle_id)?;
        let mass = ParticleDatabase::get_particle_mass(particle_id).unwrap();
        let radius = ParticleDatabase::get_particle_radius(particle_id).unwrap();
        Some(Particle {
            position,
            velocity,
            force: Vector3::new(0.0, 0.0, 0.0),
            potential: 0.0,
            id: particle_id,
            mass,
            radius,
        })
    }
}

impl Default for Particle {
    fn default() -> Self {
        Particle {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            force: Vector3::new(0.0, 0.0, 0.0),
            potential: 0.0,
            id: 0,
            mass: 1.0,
            radius: 0.1,
        }
    }
}

impl State {
    pub fn get_least_r(&self, i: usize, j: usize) -> Vector3<f64> {
        let p1 = self.particles[i].lock().expect("Can't lock particle");
        let p2 = self.particles[j].lock().expect("Can't lock particle");
        let mut min_pbc = Vector3::new(-1, -1, -1);
        let mut max_pbc = Vector3::new(1, 1, 1);
        let bb = &self.boundary_box;
        if p1.position.x < bb.x / 2.0 || p2.position.x > bb.x / 2.0 {
            max_pbc.x = 0;
        }
        if p1.position.y < bb.y / 2.0 || p2.position.y > bb.y / 2.0 {
            max_pbc.y = 0;
        }
        if p1.position.z < bb.z / 2.0 || p2.position.z > bb.z / 2.0 {
            max_pbc.z = 0;
        }
        if p1.position.x > bb.x / 2.0 || p2.position.x < bb.x / 2.0 {
            min_pbc.x = 0;
        }
        if p1.position.y > bb.y / 2.0 || p2.position.y < bb.y / 2.0 {
            min_pbc.y = 0;
        }
        if p1.position.z > bb.z / 2.0 || p2.position.z < bb.z / 2.0 {
            min_pbc.z = 0;
        }
        let mut res = p2.position - p1.position;
        for x in min_pbc.x..(max_pbc.x + 1) {
            for y in min_pbc.y..(max_pbc.y + 1) {
                for z in min_pbc.z..(max_pbc.z + 1) {
                    let offset = Vector3::new(x as f64 * bb.x, y as f64 * bb.y, z as f64 * bb.z);
                    let r = p2.position - (p1.position + offset);
                    if r.magnitude_squared() < res.magnitude_squared() {
                        res = r;
                    }
                }
            }
        }
        res
    }

    pub fn apply_boundary_conditions(&mut self) {
        let bb = &self.boundary_box;
        let slice = self.particles.as_mut_slice();
        slice.into_par_iter().for_each(|particle| {
            let particle = particle.get_mut().expect("Can't lock particle");
            particle.position.x = particle.position.x.rem_euclid(bb.x);
            particle.position.y = particle.position.y.rem_euclid(bb.y);
            particle.position.z = particle.position.z.rem_euclid(bb.z);
        });
    }
}

impl Default for State {
    fn default() -> Self {
        ParticleDatabase::add(0, "test_particle", 1.0, 0.1);
        ParticleDatabase::add(1, "test_particle1", 3.0, 0.3);
        let p1 = Particle::new(0,
                               Vector3::new(0.0, 0.0, 0.0),
                               Vector3::new(0.0, 0.0, 0.0))
            .expect("Can't create particle");
        let p2 = Particle::new(1,
                               Vector3::new(0.0, 0.5, 0.0),
                               Vector3::new(0.0, 0.0, 0.0))
            .expect("Can't create particle");
        let p3 = Particle::new(1,
                               Vector3::new(0.0, 0.25, 0.0),
                               Vector3::new(0.0, 0.0, 0.0))
            .expect("Can't create particle");
        State {
            particles: vec![Mutex::new(p1), Mutex::new(p2), Mutex::new(p3)],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        }
    }
}