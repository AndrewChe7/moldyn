use std::ops::Range;
use crate::ParticleDatabase;
use na::{Rotation3, Scale3, Vector3};

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
#[derive(Debug)]
pub struct State {
    /// Particles that exists right now
    pub particles: Vec<Vec<Particle>>,
    /// Boundary conditions for current state
    pub boundary_box: Vector3<f64>,
}

/// Okey it's tautology but it is structure that keeps data for particle structures
/// like some biological protein staff or other physical structures.
#[derive(Debug)]
pub struct Structure {
    pub particles: Vec<Vec<Particle>>,
    pub origin: Vector3<f64>,
}

impl Particle {
    /// Create new particle of given type in given position with given velocity.
    ///
    /// # Arguments
    ///
    /// * `particle_id` - ID of particle from particle database.
    /// * `position` - coordinate of particle in 3D space
    /// * `velocity` - velocity of particle
    ///
    /// # Returns
    ///
    /// Particle if `particle_id` in [ParticleDatabase] else returns None
    ///
    /// # Examples
    ///
    /// ```
    /// # use nalgebra::Vector3;
    /// # use moldyn_core::{Particle, ParticleDatabase};
    /// // Try to create particle that doesn't exist in ParticleDatabase
    /// let particle = Particle::new(0, Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
    /// assert!(particle.is_none());
    /// // Add particle to database and then create it
    /// ParticleDatabase::add(0, "Argon", 66.335, 0.071);
    /// let particle = Particle::new(0, Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
    /// assert!(particle.is_some());
    /// let particle = particle.unwrap();
    /// assert_eq!(particle.mass, 66.335);
    /// assert_eq!(particle.radius, 0.071);
    /// assert_eq!(particle.position.x, 0.0);
    /// assert_eq!(particle.velocity.x, 1.0);
    /// ```
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
    /// Creates default particle for tests. Every parameter is zero, except `mass` and `radius`.
    /// `Mass` is 1.0 and `radius` is 0.1.
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
        let boundary_box = self.boundary_box;
        let mut particles: Vec<Vec<Particle>> = vec![];
        for particle_type in &self.particles {
            let mut pt = vec![];
            for particle in particle_type {
                let particle = particle.clone();
                pt.push(particle);
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
    /// Makes every particle to satisfy periodic boundary conditions.
    pub fn apply_boundary_conditions(&mut self) {
        let bb = &self.boundary_box;
        for particle_type in self.particles.iter_mut() {
            particle_type.iter_mut().for_each(|particle| {
                particle.position.x = particle.position.x.rem_euclid(bb.x);
                particle.position.y = particle.position.y.rem_euclid(bb.y);
                particle.position.z = particle.position.z.rem_euclid(bb.z);
            });
        }
    }

    /// Get minimal and maximum and maximum velocity of particles with type `particle_type_id`.
    /// > **Warning**
    /// > This function doesn't check if particle with `particle_type_id` exists!
    pub fn get_min_max_velocity(&self, particle_type_id: u16) -> (f64, f64) {
        let mut v_squared_max = 0.0;
        let mut v_squared_min = f64::MAX;
        self.particles[particle_type_id as usize].iter().for_each(|particle| {
            let velocity_squared = particle.velocity.magnitude_squared();
            if velocity_squared > v_squared_max {
                v_squared_max = velocity_squared;
            }
            if velocity_squared < v_squared_min {
                v_squared_min = velocity_squared;
            }
        });
        (v_squared_min.sqrt(), v_squared_max.sqrt())
    }

    /// Add structure to current state with all transforms
    ///
    /// > **Warning**
    /// > WIP. No transforms yet, only position
    pub fn append_structure (&mut self, structure: &Structure,
                             position: Vector3<f64>, _rotation: Rotation3<f64>, _scale: Scale3<f64>) {
        for (type_id, particles) in structure.particles.iter().enumerate() {
            for particle in particles {
                let mut particle = particle.clone();
                particle.position += position - structure.origin;
                self.particles[type_id].push(particle);
            }
        }
    }
}

impl Default for State {
    /// This default state was created just for testing, you shouldn't use it in real code.
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
            particles: vec![vec![p1, p2, p3]],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        }
    }
}

impl Structure {
    /// Creates structure from part of a state. You could choose which particles to use with select
    /// ranges.
    ///
    /// For each type there is range in `select` argument.
    pub fn from_state (state: &State, select: &[Range<usize>]) -> Self {
        let mut particles = vec![];
        let mut mass_center = Vector3::new(0.0, 0.0, 0.0);
        let mut mass = 0.0;
        for (type_id, type_range) in select.iter().enumerate() {
            particles.push(vec![]);
            let slice = &state.particles[type_id][type_range.clone()];
            for particle in slice {
                particles[type_id].push(particle.clone());
                mass_center += particle.mass * particle.position;
                mass += particle.mass;
            }
        }
        mass_center /= mass;
        Self {
            particles,
            origin: mass_center,
        }
    }
}
