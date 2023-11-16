use nalgebra::Vector3;
use tempdir::TempDir;
use moldyn_core::{DataFile, ParticleDatabase};
use moldyn_solver::solver::update_force;
use crate::args::{CrystalCellType, IntegratorChoose};
use crate::commands::{initialize, solve};


#[test]
fn initialization() {
    let temp_dir = TempDir::new("test_dir").expect("Can't create temp directory");
    let mut path = temp_dir.into_path();
    path.push("test.json");
    let particle_name = String::from("Argon");
    let mass = 66.335;
    let radius = 0.071;
    let lattice_cell = 3.338339;
    let temperature = 273.15;
    initialize(&path, &CrystalCellType::U, &vec![10, 10, 10], &particle_name, &mass, &radius, &lattice_cell, &temperature, false);
    let data = DataFile::load_from_file(&path);
    assert_eq!(data.frames.len(), 1);
    assert_eq!(data.start_frame, 0);
    assert_eq!(data.frame_count, 1);
    assert_eq!(data.particles_database.len(), 1);
    ParticleDatabase::load(&data.particles_database);
    assert_ne!(ParticleDatabase::get_particle_name(0), None);
    let name = ParticleDatabase::get_particle_name(0).unwrap();
    let mass = ParticleDatabase::get_particle_mass(0).unwrap();
    let radius = ParticleDatabase::get_particle_radius(0).unwrap();
    assert_eq!(name, "Argon");
    assert_eq!(mass, 66.335);
    assert_eq!(radius, 0.071);
    let state = data.frames.get(&0).expect("Can't get state");
    let state: moldyn_core::State = state.into();
    assert_eq!(state.particles[0].len(), 1000);
    assert_eq!(state.boundary_box.x, 33.38339);
    assert_eq!(state.boundary_box.y, 33.38339);
    assert_eq!(state.boundary_box.z, 33.38339);
}

#[test]
fn solvation() {
    let temp_dir = TempDir::new("test_dir").expect("Can't create temp directory");
    let mut path = temp_dir.into_path();
    let mut path2 = path.clone();
    path.push("test.json");
    path2.push("test2.json");
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
    let data = DataFile::init_from_state(&state);
    data.save_to_file(&path, false);
    solve(&path, &path2, &IntegratorChoose::VerletMethod,
          &None, &None, 3, &0.002,
          100, &None, &None, &None,
          &None, &None, &None, false);
    let data = DataFile::load_from_file(&path2.with_extension("3.json"));
    let (_, mut state) = data.get_last_frame();
    update_force(&mut state);
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
