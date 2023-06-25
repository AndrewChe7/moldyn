use rayon::prelude::*;
use moldyn_core::State;

pub trait Potential: Sync + Send {
    fn get_potential (&self, r: f64) -> f64;
    fn get_force (&self, r: f64) -> f64;
}

pub fn update_force <T: Potential > (state: &mut State, potential: &T) {
    let number_particles = state.particles.len();

    (0..number_particles).into_par_iter().for_each(|i| {
        for j in 0..i {
            let r = {
                let p1 = &state.particles[i].lock().expect("Can't lock particle");
                let p2 = &state.particles[j].lock().expect("Can't lock particle");
                p2.position - p1.position
            };
            let force = potential.get_force(r.magnitude());
            let force_vec = r.normalize() * force;
            {
                let p1 = &mut state.particles[i].lock().expect("Can't lock particle");
                p1.force += force_vec;
            }
            {
                let p2 = &mut state.particles[j].lock().expect("Can't lock particle");
                p2.force -= force_vec;
            }
        }
    });
}