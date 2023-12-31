use nalgebra::Vector3;
use tempdir::TempDir;
use moldyn_core::{ParticleDatabase, StateToSave};
use moldyn_solver::solver::{PotentialsDatabase, update_force};
use crate::args::{CrystalCellType, IntegratorChoose};
use crate::commands::{initialize, solve};


#[test]
fn initialization() {
    let temp_dir = TempDir::new("test_dir").expect("Can't create temp directory");
    let path = temp_dir.into_path();
    let particle_name = String::from("Argon");
    let mass = 66.335;
    let radius = 0.071;
    let lattice_cell = 3.338339;
    let temperature = 273.15;
    initialize(&path, &CrystalCellType::U, &vec![10, 10, 10], &particle_name, &mass, &radius, &lattice_cell, &temperature);
    let data = StateToSave::load_from_file(&path, 0);
    ParticleDatabase::load_particles_data(&path).unwrap();
    assert_ne!(ParticleDatabase::get_particle_name(0), None);
    let name = ParticleDatabase::get_particle_name(0).unwrap();
    let mass = ParticleDatabase::get_particle_mass(0).unwrap();
    let radius = ParticleDatabase::get_particle_radius(0).unwrap();
    assert_eq!(name, "Argon");
    assert_eq!(mass, 66.335);
    assert_eq!(radius, 0.071);
    let state: moldyn_core::State = data.into();
    assert_eq!(state.particles[0].len(), 1000);
    assert_eq!(state.boundary_box.x, 33.38339);
    assert_eq!(state.boundary_box.y, 33.38339);
    assert_eq!(state.boundary_box.z, 33.38339);
}

#[test]
fn solvation() {
    let temp_dir = TempDir::new("test_dir").expect("Can't create temp directory");
    let path = temp_dir.into_path();
    let particle_name = String::from("Argon");
    let mass = 66.335;
    let radius = 0.071;
    ParticleDatabase::add(0, &particle_name, mass, radius);
    let p1 = moldyn_core::Particle::new(0,
                                        Vector3::new(0.75, 0.75, 0.5),
                                        Vector3::new(1.0, 1.0, 0.0))
        .expect("Can't create particle");
    let p2 = moldyn_core::Particle::new(0,
                                        Vector3::new(1.25, 0.75, 0.5),
                                        Vector3::new(-1.0, 1.0, 0.0))
        .expect("Can't create particle");
    let state = moldyn_core::State {
        particles: vec![vec![p1, p2]],
        boundary_box: Vector3::new(2.0, 2.0, 2.0),
    };
    let potentials_db = PotentialsDatabase::new();
    let data = StateToSave::from(&state);
    data.save_to_file(&path, 0);
    ParticleDatabase::save_particles_data(&path).expect("");
    solve(&path, 0, &IntegratorChoose::VerletMethod,
          &None, &false, 3, &0.002,
          &None, &None, &None,
          &None, &None, &None);
    let data = StateToSave::load_from_file(&path, 3);
    let mut state = data.into();
    update_force(&potentials_db, &mut state);
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
