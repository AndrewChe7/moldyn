use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nalgebra::Vector3;
use moldyn_core::ParticleDatabase;
use moldyn_solver::initializer::{initialize_particles, initialize_particles_position, UnitCell};

static UNIT_CELL: f64 = 3.338339;

pub fn uniform_positions_10_10_10_bench(c: &mut Criterion) {
    ParticleDatabase::add(0, "Argon", 66.335, 0.071);
    let size = (10, 10, 10);
    let size_v = Vector3::new(size.0 as f64, size.1 as f64, size.2 as f64);
    let mut state = initialize_particles(&[size.0 * size.1 * size.2],
                                         &(size_v * UNIT_CELL)).unwrap();
    c.bench_function("uniform positions 1000 particles", |b|
        b.iter(||
            initialize_particles_position(UnitCell::U,
                                          &mut state,
                                          black_box(0),
                                          black_box((0.0, 0.0, 0.0)),
                                          black_box(size),
                                          black_box(UNIT_CELL))));
}

pub fn uniform_positions_30_30_30_bench(c: &mut Criterion) {
    ParticleDatabase::add(0, "Argon", 66.335, 0.071);
    let size = (30, 30, 30);
    let size_v = Vector3::new(size.0 as f64, size.1 as f64, size.2 as f64);
    let mut state = initialize_particles(&[size.0 * size.1 * size.2],
                                         &(size_v * UNIT_CELL)).unwrap();
    c.bench_function("uniform positions 27000 particles", |b|
        b.iter(||
            initialize_particles_position(UnitCell::U,
                                          &mut state,
                                          black_box(0),
                                          black_box((0.0, 0.0, 0.0)),
                                          black_box(size),
                                          black_box(UNIT_CELL))));
}

pub fn uniform_positions_50_50_50_bench(c: &mut Criterion) {
    ParticleDatabase::add(0, "Argon", 66.335, 0.071);
    let size = (50, 50, 50);
    let size_v = Vector3::new(size.0 as f64, size.1 as f64, size.2 as f64);
    let mut state = initialize_particles(&[size.0 * size.1 * size.2],
                                         &(size_v * UNIT_CELL)).unwrap();
    c.bench_function("uniform positions 125000 particles", |b|
        b.iter(||
            initialize_particles_position(UnitCell::U,
                                          &mut state,
                                          black_box(0),
                                          black_box((0.0, 0.0, 0.0)),
                                          black_box(size),
                                          black_box(UNIT_CELL))));
}

criterion_group!(init_benches, uniform_positions_10_10_10_bench,
    uniform_positions_30_30_30_bench, uniform_positions_50_50_50_bench);
criterion_main!(init_benches);