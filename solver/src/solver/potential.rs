use lazy_static::lazy_static;
use moldyn_core::State;
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use rand_distr::num_traits::Pow;
use serde::{Deserialize, Serialize};

/// Enum to keep data for potential calculation
#[derive(Clone, Serialize, Deserialize)]
pub enum Potential {
    LennardJones {
        sigma: f64,
        eps: f64,
        r_cut: f64,
        u_cut: f64,
    },
    Custom {
        name: String,
        custom_data: Vec<f64>,
    },
}

impl Potential {
    /// Creates Lennard-Jones potential object
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

    pub fn get_potential_and_force(&self, r: f64) -> (f64, f64) {
        match self {
            Potential::LennardJones { sigma, eps, r_cut, u_cut } => {
                if r > *r_cut {
                    return (0.0, 0.0);
                }
                let sigma_r = *sigma / r;
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

    pub fn get_radius_cut(&self) -> f64 {
        match self {
            Potential::LennardJones { r_cut, .. } => {
                r_cut.clone()
            }
            Potential::Custom { .. } => {
                todo!()
            }
        }
    }
}

lazy_static! {
    static ref POTENTIALS_DATA: RwLock<HashMap<(u16, u16), Arc<Potential>>> = RwLock::new(HashMap::new());
    static ref DEFAULT_POTENTIAL: Arc<Potential> = Arc::new(Potential::new_lennard_jones(0.3418, 1.712));
}

/// Get potential object from potentials database
pub fn get_potential(id0: u16, id1: u16) -> Arc<Potential> {
    let db = POTENTIALS_DATA.read()
        .expect("Can't lock potentials database");
    let key = if id0 > id1 { (id1, id0) } else { (id0, id1) };
    if db.contains_key(&key) {
        Arc::clone(db.get(&key).unwrap())
    } else {
        Arc::clone(&DEFAULT_POTENTIAL)
    }
}

/// Setup potentials and forces for each particle in `state`
pub fn update_force(state: &mut State) {
    let particle_type_count = state.particles.len();
    let bb = &state.boundary_box;
    state.particles.iter_mut().for_each(|particle_type| {
        particle_type.iter_mut().for_each(|particle| {
            let particle = particle.get_mut().expect("Can't lock particle");
            particle.force.x = 0.0;
            particle.force.y = 0.0;
            particle.force.z = 0.0;
            particle.potential = 0.0;
            particle.temp = 0.0;
        });
    });

    for particle_type1 in 0..particle_type_count {
        for particle_type2 in particle_type1..particle_type_count {
            let potential = get_potential(particle_type1 as u16, particle_type2 as u16);
            let r_cut = potential.get_radius_cut();
            let number_particles1 = state.particles[particle_type1].len();
            (0..number_particles1).into_iter().for_each(|i| {
                let number_particles2 = if particle_type1 == particle_type2 {
                    i
                } else {
                    state.particles[particle_type2].len()
                };
                let mut p1 = state.particles[particle_type1][i].write().expect("Can't lock particle");
                for j in 0..number_particles2 {
                    let mut p2 = state.particles[particle_type2][j].write().expect("Can't lock particle");
                    let mut r = p2.position - p1.position;
                    if r.x < -bb.x / 2.0 {
                        r.x += bb.x;
                    } else if r.x > bb.x / 2.0 {
                        r.x -= bb.x;
                    }
                    if r.y < -bb.y / 2.0 {
                        r.y += bb.y;
                    } else if r.y > bb.y / 2.0 {
                        r.y -= bb.y;
                    }
                    if r.z < -bb.z / 2.0 {
                        r.z += bb.z;
                    } else if r.z > bb.z / 2.0 {
                        r.z -= bb.z;
                    }
                    if r.magnitude() > r_cut {
                        continue;
                    }

                    let (potential, force) = potential.get_potential_and_force(r.magnitude());
                    let force_vec = r.normalize() * force;
                    let t = force_vec.x * r.x + force_vec.y * r.y + force_vec.z * r.z;
                    {
                        p1.force += force_vec;
                        p1.potential += potential;
                        p1.temp += t;
                    }
                    {
                        p2.force -= force_vec;
                        p2.potential += potential;
                    }
                }
            });
        }
    }
}

/// Save potentials database to file
pub fn save_potentials_to_file (path: &PathBuf) {
    let db = POTENTIALS_DATA.read()
        .expect("Can't lock potentials database");
    let mut new_db: HashMap<(u16, u16), Potential> = HashMap::new();
    for (k, v) in db.iter() {
        new_db.insert(k.clone(), v.as_ref().clone()).unwrap();
    }
    let file = if path.exists() {
        OpenOptions::new().truncate(true).write(true).open(path).expect("Can't open file")
    } else {
        File::create(path).expect("Can't create file")
    };
    let mut buf_writer = BufWriter::new(file);
    serde_json::ser::to_writer_pretty(&mut buf_writer, &new_db)
        .expect("Can't save potential settings");
}

/// Load potentials database to file
pub fn load_potentials_from_file (path: &PathBuf) {
    let file = File::open(path).expect("Can't open file");
    let buf_reader = BufReader::new(file);
    let data: HashMap<(u16, u16), Potential> = serde_json::de::from_reader(buf_reader)
        .expect("Can't load data from file");
    let mut db = POTENTIALS_DATA.write()
        .expect("Can't lock potentials database");
    for (id, potential) in data {
        db.insert(id, Arc::new(potential));
    }
}
