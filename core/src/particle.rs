use crate::ParticleDatabase;
use na::Vector3;
use std::sync::RwLock;

/// Structure that keeps all data for particle
#[derive(Clone, Debug)]
pub struct Particle {
    /// position of particle in 3d space
    pub position: Vector3<f64>,
    /// velocity of particle
    pub velocity: Vector3<f64>,
    /// The sum of the forces acting on the particle
    pub force: Vector3<f64>,
    /// The total potential of the particle
    pub potential: f64,
    /// Sum of F(i,j) * r(i, j) for every other particle
    pub temp: f64,
    /// Mass of particle
    pub mass: f64,
    /// Radius of particle
    pub radius: f64,
    /// ID of particle. Defines type of particle
    pub id: u16,
}

/// Structure that keeps current state
pub struct State {
    pub particles: Vec<Vec<RwLock<Particle>>>,
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
            temp: 0.0,
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
            temp: 0.0,
            id: 0,
            mass: 1.0,
            radius: 0.1,
        }
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        let boundary_box = self.boundary_box.clone();
        let mut particles: Vec<Vec<RwLock<Particle>>> = vec![];
        for particle_type in &self.particles {
            let mut pt = vec![];
            for particle in particle_type {
                let particle = particle.read().expect("Can't lock particle");
                let particle = particle.clone();
                pt.push(RwLock::new(particle));
            }
            particles.push(pt);
        }
        Self {
            particles,
            boundary_box,
        }
    }
}

impl State {
    pub fn apply_boundary_conditions(&mut self) {
        let bb = &self.boundary_box;
        for particle_type in self.particles.iter_mut() {
            let slice = particle_type.as_mut_slice();
            slice.into_iter().for_each(|particle| {
                let particle = particle.get_mut().expect("Can't lock particle");
                particle.position.x = particle.position.x.rem_euclid(bb.x);
                particle.position.y = particle.position.y.rem_euclid(bb.y);
                particle.position.z = particle.position.z.rem_euclid(bb.z);
            });
        }
    }

    pub fn get_min_max_velocity(&self, particle_type_id: u16) -> (f64, f64) {
        let mut v_squared_max = 0.0;
        let mut v_squared_min = f64::MAX;
        self.particles[particle_type_id as usize].iter().for_each(|particle| {
            let particle = particle.read().expect("Can't lock particle");
            if particle.velocity.magnitude_squared() > v_squared_max {
                v_squared_max = particle.velocity.magnitude_squared();
            }
            if particle.velocity.magnitude_squared() < v_squared_min {
                v_squared_min = particle.velocity.magnitude_squared();
            }
        });
        (v_squared_min.sqrt(), v_squared_max.sqrt())
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
            particles: vec![vec![RwLock::new(p1), RwLock::new(p2), RwLock::new(p3)]],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        }
    }
}