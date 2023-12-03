use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nalgebra::Vector3;
use moldyn_core::ParticleDatabase;
use moldyn_solver::initializer::{initialize_particles, randomize_positions};
use moldyn_solver::solver::{Potential, update_force};

static UNIT_CELL: f64 = 3.338339;

pub fn lennard_jones_bench(c: &mut Criterion) {
    let lennard_jones = Potential::new_lennard_jones(0.3418, 1.712);
    c.bench_function("lennard jones", |b| b.iter(|| lennard_jones.get_potential_and_force(black_box(0.3))));
}

pub fn update_force_10_10_10_bench(c: &mut Criterion) {
    ParticleDatabase::add(0, "Argon", 66.335, 0.071);
    let size = (10, 10, 10);
    let size_v = Vector3::new(size.0 as f64, size.1 as f64, size.2 as f64);
    let mut state = initialize_particles(&[size.0 * size.1 * size.2],
                                         &(size_v * UNIT_CELL)).unwrap();
    randomize_positions(&mut state, 0, size, UNIT_CELL);
    c.bench_function("update force 1000", |b| b.iter(|| {
        let mut new_state = state.clone();
        update_force(black_box(&mut new_state))
    })
    );
}

pub fn update_force_20_20_20_bench(c: &mut Criterion) {
    ParticleDatabase::add(0, "Argon", 66.335, 0.071);
    let size = (20, 20, 20);
    let size_v = Vector3::new(size.0 as f64, size.1 as f64, size.2 as f64);
    let mut state = initialize_particles(&[size.0 * size.1 * size.2],
                                         &(size_v * UNIT_CELL)).unwrap();
    randomize_positions(&mut state, 0, size, UNIT_CELL);
    c.bench_function("update force 8000", |b| b.iter(|| {
        let mut new_state = state.clone();
        update_force(black_box(&mut new_state))
    })
    );
}

pub fn update_force_30_30_30_bench(c: &mut Criterion) {
    ParticleDatabase::add(0, "Argon", 66.335, 0.071);
    let size = (30, 30, 30);
    let size_v = Vector3::new(size.0 as f64, size.1 as f64, size.2 as f64);
    let mut state = initialize_particles(&[size.0 * size.1 * size.2],
                                         &(size_v * UNIT_CELL)).unwrap();
    randomize_positions(&mut state, 0, size, UNIT_CELL);
    c.bench_function("update force 27000", |b| b.iter(|| {
        let mut new_state = state.clone();
        update_force(black_box(&mut new_state))
    })
    );
}


criterion_group!(solver_benches, lennard_jones_bench,
    update_force_10_10_10_bench, update_force_20_20_20_bench, update_force_30_30_30_bench);
criterion_main!(solver_benches);