mod particle;
extern crate nalgebra as na;
extern crate serde;

pub use particle::Particle;
pub use particle::State;

#[cfg(test)]
mod tests {
    use na::Vector3;
    use crate::State;
    use crate::Particle;

    fn test_particle() -> Particle {
        Particle {
            position: Vector3::new(0.1, 0.2, 0.3),
            velocity: Vector3::new(0.1, 0.2, 0.3),
            force: Vector3::new(0.3, 2.2, 1.0),
            potential: 1.0,
            mass: 2.0,
            id: 3,
        }
    }

    fn check_particle_equality(p1: &Particle, p2: &Particle) {
        assert_eq!(p1.position, p2.position);
        assert_eq!(p1.velocity, p2.velocity);
        assert_eq!(p1.force, p2.force);
        assert_eq!(p1.potential, p2.potential);
        assert_eq!(p1.mass, p2.mass);
        assert_eq!(p1.id, p2.id);
    }

    #[test]
    fn particle_serialization () {
        let particle = test_particle();

        let serialized = ron::to_string(&particle).unwrap();
        let deserialized: Particle = ron::from_str(&serialized).unwrap();

        check_particle_equality(&particle, &deserialized);
    }

    #[test]
    fn state_serialization () {
        let particle = test_particle();

        let state = State {
            particles: vec![test_particle(), test_particle()],
        };

        let serialized = ron::to_string(&state).unwrap();
        let deserialized: State = ron::from_str(&serialized).unwrap();

        for p in &deserialized.particles {
            check_particle_equality(p, &particle);
        }
    }
}
