extern crate moldyn_core;
extern crate nalgebra as na;
extern crate rand_distr;
pub mod initializer;
pub mod macro_parameters;
pub mod solver;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initializer::{Barostat, InitError, initialize_particles, initialize_particles_position, initialize_velocities_maxwell_boltzmann, Thermostat, UnitCell};
    use crate::macro_parameters::{get_center_of_mass_velocity, get_kinetic_energy, get_potential_energy, get_pressure, get_temperature, get_thermal_energy};
    use moldyn_core::{Particle, ParticleDatabase, State};
    use crate::solver::*;
    use na::Vector3;

    #[test]
    fn initialize_uniform_grid() {
        let res = initialize_particles(&[8],
                                       &Vector3::new(4.0, 4.0, 4.0));
        assert_eq!(res.unwrap_err(), InitError::ParticleIdDidNotFound);
        ParticleDatabase::add(0, "test_particle", 1.0, 0.1);
        let mut state = initialize_particles(&[8],
                                         &Vector3::new(4.0, 4.0, 4.0)).unwrap();
        let res = initialize_particles_position(
            UnitCell::U,
            &mut state,
            0,
            (0.0, 0.0, 0.0),
            (3, 3, 3),
            1.0,
        );
        assert_eq!(res, Err(InitError::TooBig));
        let res = initialize_particles_position(
            UnitCell::U,
            &mut state,
            0,
            (0.0, 0.0, 0.0),
            (2, 2, 2),
            2.0,
        );
        assert_eq!(res, Ok(()));
        let particle = &state.particles[0][2];
        assert_eq!(particle.position.x, 0.0);
        assert_eq!(particle.position.y, 2.0);
        assert_eq!(particle.position.z, 0.0);
    }

    fn check_momentum(state: &State) {
        let mut p = Vector3::new(0.0, 0.0, 0.0);
        for particle in &state.particles[0] {
            p += particle.velocity;
        }
        assert!(p.x.abs() < 1e-12);
        assert!(p.y.abs() < 1e-12);
        assert!(p.z.abs() < 1e-12);
    }

    #[test]
    fn momentum () {
        let bounding_box = Vector3::new(2.0, 2.0, 2.0) * 3.338339;
        let verlet_method = Integrator::VerletMethod;
        ParticleDatabase::add(0, "Argon", 66.335, 0.071);
        let mut state = initialize_particles(&[8], &bounding_box).unwrap();
        initialize_particles_position(UnitCell::U, &mut state, 0, (0.0, 0.0, 0.0), (2, 2, 2), 3.338339)
            .expect("Can't initialize particles");
        initialize_velocities_maxwell_boltzmann(&mut state, 273.15, 0);
        update_force(&mut state);
        check_momentum(&state);
        for _ in 0..100000 {
            verlet_method.calculate(&mut state, 0.002, &mut None, &mut None);
            check_momentum(&state);
        }
    }

    #[test]
    fn lennard_jones() {
        let lennard_jones_potential = Potential::new_lennard_jones(0.3418, 1.712);
        let (potential, force) = lennard_jones_potential.get_potential_and_force(0.5);
        assert_eq!(
            format!("{:.8}", potential),
            "-0.59958655"
        );
        assert_eq!(
            format!("{:.8}", force),
            "6.67445797"
        );
    }

    #[test]
    fn update_force_lennard_jones() {
        let p1 = Particle::default();
        let mut p2 = Particle::default();
        p2.position.x = 0.5;
        let mut state = State {
            particles: vec![vec![p1, p2]],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        update_force(&mut state);
        let force_p1 = &state.particles[0][0]
            .force;
        assert_eq!(format!("{:.8}", force_p1.x), "6.67445797");
        assert_eq!(format!("{:.8}", force_p1.y), "0.00000000");
        assert_eq!(format!("{:.8}", force_p1.z), "0.00000000");
    }

    #[test]
    fn verlet_with_lennard_jones() {
        let mut p1 = Particle::default();
        let mut p2 = Particle::default();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![vec![p1, p2]],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        let verlet = Integrator::VerletMethod;
        update_force(&mut state); // Initialize forces
        {
            let p1 = &state.particles[0][0];
            let p2 = &state.particles[0][1];
            let pos1 = p1.position;
            let pos2 = p2.position;
            let v1 = p1.velocity;
            let v2 = p2.velocity;
            let f1 = p1.force;
            let f2 = p2.force;

            assert_eq!(format!("{:.8}", pos1.x), "0.75000000");
            assert_eq!(format!("{:.8}", pos1.y), "0.75000000");
            assert_eq!(format!("{:.8}", pos1.z), "0.50000000");

            assert_eq!(format!("{:.8}", pos2.x), "1.25000000");
            assert_eq!(format!("{:.8}", pos2.y), "0.75000000");
            assert_eq!(format!("{:.8}", pos2.z), "0.50000000");

            assert_eq!(format!("{:.8}", v1.x), "1.00000000");
            assert_eq!(format!("{:.8}", v1.y), "1.00000000");
            assert_eq!(format!("{:.8}", v1.z), "0.00000000");

            assert_eq!(format!("{:.8}", v2.x), "-1.00000000");
            assert_eq!(format!("{:.8}", v2.y), "1.00000000");
            assert_eq!(format!("{:.8}", v2.z), "0.00000000");

            assert_eq!(format!("{:.8}", f1.x), "6.67445797");
            assert_eq!(format!("{:.8}", f1.y), "0.00000000");
            assert_eq!(format!("{:.8}", f1.z), "0.00000000");

            assert_eq!(format!("{:.8}", f2.x), "-6.67445797");
            assert_eq!(format!("{:.8}", f2.y), "0.00000000");
            assert_eq!(format!("{:.8}", f2.z), "0.00000000");
        }
        verlet.calculate(&mut state, 0.002, &mut None, &mut None);
        {
            let p1 = &state.particles[0][0];
            let p2 = &state.particles[0][1];
            let pos1 = p1.position;
            let pos2 = p2.position;
            let v1 = p1.velocity;
            let v2 = p2.velocity;
            let f1 = p1.force;
            let f2 = p2.force;
            assert_eq!(format!("{:.8}", pos1.x), "0.75200020");
            assert_eq!(format!("{:.8}", pos1.y), "0.75200000");
            assert_eq!(format!("{:.8}", pos1.z), "0.50000000");

            assert_eq!(format!("{:.8}", pos2.x), "1.24799980");
            assert_eq!(format!("{:.8}", pos2.y), "0.75200000");
            assert_eq!(format!("{:.8}", pos2.z), "0.50000000");

            assert_eq!(format!("{:.8}", f1.x), "6.97111732");
            assert_eq!(format!("{:.8}", f1.y), "0.00000000");
            assert_eq!(format!("{:.8}", f1.z), "0.00000000");

            assert_eq!(format!("{:.8}", f2.x), "-6.97111732");
            assert_eq!(format!("{:.8}", f2.y), "0.00000000");
            assert_eq!(format!("{:.8}", f2.z), "0.00000000");

            assert_eq!(format!("{:.8}", v1.x), "1.00020571");
            assert_eq!(format!("{:.8}", v1.y), "1.00000000");
            assert_eq!(format!("{:.8}", v1.z), "0.00000000");

            assert_eq!(format!("{:.8}", v2.x), "-1.00020571");
            assert_eq!(format!("{:.8}", v2.y), "1.00000000");
            assert_eq!(format!("{:.8}", v2.z), "0.00000000");
        }
        verlet.calculate(&mut state, 0.002, &mut None, &mut None);
        {
            let p1 = &state.particles[0][0];
            let p2 = &state.particles[0][1];
            let pos1 = p1.position;
            let pos2 = p2.position;
            let v1 = p1.velocity;
            let v2 = p2.velocity;
            let f1 = p1.force;
            let f2 = p2.force;
            assert_eq!(format!("{:.8}", pos1.x), "0.75400082");
            assert_eq!(format!("{:.8}", pos1.y), "0.75400000");
            assert_eq!(format!("{:.8}", pos1.z), "0.50000000");

            assert_eq!(format!("{:.8}", pos2.x), "1.24599918");
            assert_eq!(format!("{:.8}", pos2.y), "0.75400000");
            assert_eq!(format!("{:.8}", pos2.z), "0.50000000");

            assert_eq!(format!("{:.8}", f1.x), "7.27764996");
            assert_eq!(format!("{:.8}", f1.y), "0.00000000");
            assert_eq!(format!("{:.8}", f1.z), "0.00000000");

            assert_eq!(format!("{:.8}", f2.x), "-7.27764996");
            assert_eq!(format!("{:.8}", f2.y), "0.00000000");
            assert_eq!(format!("{:.8}", f2.z), "0.00000000");

            assert_eq!(format!("{:.8}", v1.x), "1.00042051");
            assert_eq!(format!("{:.8}", v1.y), "1.00000000");
            assert_eq!(format!("{:.8}", v1.z), "0.00000000");

            assert_eq!(format!("{:.8}", v2.x), "-1.00042051");
            assert_eq!(format!("{:.8}", v2.y), "1.00000000");
            assert_eq!(format!("{:.8}", v2.z), "0.00000000");
        }
        verlet.calculate(&mut state, 0.002, &mut None, &mut None);
        {
            let p1 = &state.particles[0][0];
            let p2 = &state.particles[0][1];
            let pos1 = p1.position;
            let pos2 = p2.position;
            let v1 = p1.velocity;
            let v2 = p2.velocity;
            let f1 = p1.force;
            let f2 = p2.force;
            assert_eq!(format!("{:.8}", pos1.x), "0.75600188");
            assert_eq!(format!("{:.8}", pos1.y), "0.75600000");
            assert_eq!(format!("{:.8}", pos1.z), "0.50000000");

            assert_eq!(format!("{:.8}", pos2.x), "1.24399812");
            assert_eq!(format!("{:.8}", pos2.y), "0.75600000");
            assert_eq!(format!("{:.8}", pos2.z), "0.50000000");

            assert_eq!(format!("{:.8}", f1.x), "7.59359964");
            assert_eq!(format!("{:.8}", f1.y), "0.00000000");
            assert_eq!(format!("{:.8}", f1.z), "0.00000000");

            assert_eq!(format!("{:.8}", f2.x), "-7.59359964");
            assert_eq!(format!("{:.8}", f2.y), "0.00000000");
            assert_eq!(format!("{:.8}", f2.z), "0.00000000");

            assert_eq!(format!("{:.8}", v1.x), "1.00064469");
            assert_eq!(format!("{:.8}", v1.y), "1.00000000");
            assert_eq!(format!("{:.8}", v1.z), "0.00000000");

            assert_eq!(format!("{:.8}", v2.x), "-1.00064469");
            assert_eq!(format!("{:.8}", v2.y), "1.00000000");
            assert_eq!(format!("{:.8}", v2.z), "0.00000000");
        }
    }

    #[test]
    fn verlet_lj_1000_iterations () {
        let mut p1 = Particle::default();
        let mut p2 = Particle::default();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![vec![p1, p2]],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        let verlet = Integrator::VerletMethod;
        update_force(&mut state); // Initialize forces
        for _ in 0..999 {
            verlet.calculate(&mut state, 0.002, &mut None, &mut None);
        }
        {
            let p1 = &state.particles[0][0];
            let p2 = &state.particles[0][1];
            let pos1 = p1.position;
            let pos2 = p2.position;
            let v1 = p1.velocity;
            let v2 = p2.velocity;
            let f1 = p1.force;
            let f2 = p2.force;
            assert_eq!(format!("{:.8}", pos1.x), "0.50617554");
            assert_eq!(format!("{:.8}", pos1.y), "0.74800000");
            assert_eq!(format!("{:.8}", pos1.z), "0.50000000");

            assert_eq!(format!("{:.8}", pos2.x), "1.49382446");
            assert_eq!(format!("{:.8}", pos2.y), "0.74800000");
            assert_eq!(format!("{:.8}", pos2.z), "0.50000000");

            assert_eq!(format!("{:.8}", f1.x), "0.00000000");
            assert_eq!(format!("{:.8}", f1.y), "0.00000000");
            assert_eq!(format!("{:.8}", f1.z), "0.00000000");

            assert_eq!(format!("{:.8}", f2.x), "0.00000000");
            assert_eq!(format!("{:.8}", f2.y), "0.00000000");
            assert_eq!(format!("{:.8}", f2.z), "0.00000000");

            assert_eq!(format!("{:.8}", v1.x), "-0.99547744");
            assert_eq!(format!("{:.8}", v1.y), "1.00000000");
            assert_eq!(format!("{:.8}", v1.z), "0.00000000");

            assert_eq!(format!("{:.8}", v2.x), "0.99547744");
            assert_eq!(format!("{:.8}", v2.y), "1.00000000");
            assert_eq!(format!("{:.8}", v2.z), "0.00000000");
        }
        let mv = get_center_of_mass_velocity(&state, 0);
        let kinetic = get_kinetic_energy(&state, 0);
        let thermal = get_thermal_energy(&state, 0, &mv);
        let potential = get_potential_energy(&state, 0);
        let internal = thermal + potential;
        let full = kinetic + potential;
        let temperature = get_temperature(thermal, 2);
        let pressure = get_pressure(&state, 0, &mv);
        assert_eq!(format!("{:.8}", kinetic), "132.07134835");
        assert_eq!(format!("{:.8}", thermal), "65.73634835");
        assert_eq!(format!("{:.8}", potential), "0.00000000");
        assert_eq!(format!("{:.8}", internal), "65.73634835");
        assert_eq!(format!("{:.8}", full), "132.07134835");
        assert_eq!(format!("{:.8}", temperature / 100.0), "15.87088652");
        assert_eq!(format!("{:.8}", pressure), "5.47802903");
    }

    #[test]
    fn energies() {
        let mut p1 = Particle::default();
        let mut p2 = Particle::default();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![vec![p1, p2]],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        update_force(&mut state);
        let mv = get_center_of_mass_velocity(&state, 0);
        assert_eq!(mv, Vector3::new(0.0, 1.0, 0.0));
        let e_kinetic = get_kinetic_energy(&state, 0);
        let e_thermal = get_thermal_energy(&state, 0, &mv);
        let e_potential = get_potential_energy(&state, 0);
        let e = e_kinetic + e_potential;
        let e_internal = e_thermal + e_potential;
        assert_eq!(
            format!("{:.8}", e_kinetic),
            "132.67000000"
        );
        assert_eq!(
            format!("{:.8}", e_thermal),
            "66.33500000"
        );
        assert_eq!(
            format!("{:.8}", e_potential),
            "-0.59958655"
        );
        assert_eq!(
            format!("{:.8}", e_internal),
            "65.73541345"
        );
        assert_eq!(
            format!("{:.8}", e),
            "132.07041345"
        );
    }

    #[test]
    fn temperature() {
        let mut p1 = Particle::default();
        let mut p2 = Particle::default();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![vec![p1, p2]],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        update_force(&mut state);
        let mv = get_center_of_mass_velocity(&state, 0);
        let e_thermal = get_thermal_energy(&state, 0, &mv);
        let temperature = get_temperature(e_thermal, 2);
        assert_eq!(
            format!("{:.8}", temperature),
            "1601.54204479"
        );
    }

    #[ignore]
    #[test]
    fn pressure() {
        let mut p1 = Particle::default();
        let mut p2 = Particle::default();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![vec![p1, p2]],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        update_force(&mut state);
        let mv = get_center_of_mass_velocity(&state, 0);
        let pressure = get_pressure(&state, 0, &mv);
        assert_eq!(
            format!("{:.8}", pressure),
            "5.38886546"
        );
    }

    #[ignore]
    #[test]
    fn berendsen_thermostat () {
        let bb = Vector3::new(2.0, 2.0, 2.0) * 3.338339;
        ParticleDatabase::add(0, "Argon", 66.335, 0.071);
        let mut state = initialize_particles(&[8], &bb).unwrap();
        initialize_particles_position(UnitCell::U, &mut state, 0,
                                      (0.0, 0.0, 0.0), (2, 2, 2), 3.338339)
            .expect("Can't init particles");
        initialize_velocities_maxwell_boltzmann(&mut state, 273.15, 0);
        update_force(&mut state);
        let mut berendsen = Thermostat::Berendsen {
            tau: 0.5,
            lambda: 0.0,
        };
        let verlet = Integrator::VerletMethod;
        for _ in 0..100000 {
            verlet.calculate(&mut state, 0.002, &mut None, &mut Some((&mut berendsen, 273.15)));
        }
        let mv = get_center_of_mass_velocity(&state, 0);
        let thermal_energy = get_thermal_energy(&state, 0, &mv);
        let temperature = get_temperature(thermal_energy, 8);
        println!("{}", temperature);
        assert!((temperature - 273.15).abs() < 1e-5);
    }

    #[ignore]
    #[test]
    fn berendsen_barostat () {
        let bb = Vector3::new(2.0, 2.0, 2.0) * 3.338339;
        ParticleDatabase::add(0, "Argon", 66.335, 0.071);
        let mut state = initialize_particles(&[8], &bb).unwrap();
        initialize_particles_position(UnitCell::U, &mut state, 0,
                                      (0.0, 0.0, 0.0), (2, 2, 2), 3.338339)
            .expect("Can't init particles");
        initialize_velocities_maxwell_boltzmann(&mut state, 273.15, 0);
        update_force(&mut state);
        let mut berendsen = Barostat::Berendsen {
            beta: 1.0,
            tau: 0.1,
            myu: 0.0,
        };
        let verlet = Integrator::VerletMethod;
        for _ in 0..100000 {
            verlet.calculate(&mut state, 0.002, &mut Some((&mut berendsen, 0.101325)), &mut None);
        }
        update_force(&mut state);
        let mv = get_center_of_mass_velocity(&state, 0);
        let pressure = get_pressure(&state, 0, &mv);
        println!("{:.15} ", pressure);
        assert!((pressure - 0.101325).abs() < 1e-5);
    }
}
