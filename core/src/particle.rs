use na::Vector3;
use serde::{Deserialize, Serialize};

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
    /// ID of particle. Defines type of particle
    pub id: i16,
}

/// Structure that keeps current state
#[derive(Serialize, Deserialize)]
pub struct State {
    pub particles: Vec<Particle>,
}