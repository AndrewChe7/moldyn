use lazy_static::lazy_static;
use moldyn_core::State;
use rayon::prelude::*;
use std::sync::Mutex;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use rand_distr::num_traits::Pow;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Potential {
    LennardJones {
        sigma: f64,
        eps: f64,
        r_cut: f64,
        u_cut: f64,
    },
    Custom{
        name: String,
        custom_data: Vec<f64>,
    },
}

impl Potential {
    pub fn new_lennard_jones(sigma: f64, eps: f64) -> Potential {
        let r_cut = sigma * 2.5;
        let mut potential = Potential::LennardJones {
            sigma,
            eps,
            r_cut,
            u_cut: 0.0,
        };
        potential = match potential {
            Potential::LennardJones {
                sigma,
                eps,
                r_cut,
                ..
            } => {
                let (u_cut, _) = potential.get_potential_and_force(r_cut.clone());
                Potential::LennardJones {
                    sigma,
                    eps,
                    r_cut,
                    u_cut,
                }
            }
            _ => {
                panic!("It's impossible to get another potential")
            }
        };
        potential
    }

    pub fn get_potential_and_force(self, r: f64) -> (f64, f64) {
        match self {
            Potential::LennardJones { sigma, eps, r_cut, u_cut } => {
                if r > r_cut {
                    return (0.0, 0.0);
                }
                let sigma_r = sigma / r;
                let sigma_r_6 = sigma_r.pow(6);
                let sigma_r_12 = sigma_r_6 * sigma_r_6;
                (
                    4.0f64 * eps * (sigma_r_12 - sigma_r_6) - u_cut,
                    (24.0f64 * eps / r) * (sigma_r_6 - 2.0f64 * sigma_r_12),
                )
            }
            Potential::Custom { .. } => {
                todo!()
            }
        }
    }
}

lazy_static! {
    static ref POTENTIALS_DATA: Mutex<HashMap<(u16, u16), Potential>> = Mutex::new(HashMap::new());
}

pub fn get_potential(state: &State, i: usize, j: usize) -> Potential {
    let db = POTENTIALS_DATA.lock()
        .expect("Can't lock potentials database");
    let id0 = state.particles[i].lock().expect("Can't lock particle").id;
    let id1 = state.particles[j].lock().expect("Can't lock particle").id;
    let key = if id0 > id1 { (id1, id0) } else { (id0, id1) };
    let potential = db.get(&key);
    if let Some(potential) = potential {
        potential.clone()
    } else {
        Potential::new_lennard_jones(0.3418, 1.712)
    }
}

pub fn update_force(state: &mut State) {
    let number_particles = state.particles.len();
    state.particles.par_iter_mut().for_each(|particle| {
        let particle = particle.get_mut().expect("Can't lock particle");
        particle.force.x = 0.0;
        particle.force.y = 0.0;
        particle.force.z = 0.0;
        particle.potential = 0.0;
        particle.temp = 0.0;
    });

    (0..number_particles).into_par_iter().for_each(|i| {
        for j in 0..i {
            let r = state.get_least_r(i, j);
            let potential = get_potential(state, i, j);
            let (potential, force) = potential.get_potential_and_force(r.magnitude());
            let force_vec = r.normalize() * force;
            let t = force_vec.x * r.x + force_vec.y * r.y + force_vec.z * r.z;
            {
                let p1 = &mut state.particles[i].lock().expect("Can't lock particle");
                p1.force += force_vec;
                p1.potential += potential;
                p1.temp += t;
            }
            {
                let p2 = &mut state.particles[j].lock().expect("Can't lock particle");
                p2.force -= force_vec;
                p2.potential += potential;
            }
        }
    });
}

pub fn save_potentials_to_file (path: &PathBuf) {
    let db = POTENTIALS_DATA.lock()
        .expect("Can't lock potentials database");
    let file = if path.exists() {
        File::open(path).expect("Can't open file")
    } else {
        File::create(path).expect("Can't create file")
    };
    serde_json::ser::to_writer_pretty(file, &db.clone())
        .expect("Can't save potential settings");
}

pub fn load_potentials_from_file (path: &PathBuf) {
    let file = File::open(path).expect("Can't open file");
    let data: HashMap<(u16, u16), Potential> = serde_json::de::from_reader(&file)
        .expect("Can't load data from file");
    let mut db = POTENTIALS_DATA.lock()
        .expect("Can't lock potentials database");
    for (id, potential) in data {
        db.insert(id, potential);
    }
}
