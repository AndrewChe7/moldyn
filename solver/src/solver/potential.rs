use moldyn_core::State;
use rayon::prelude::*;

pub trait Potential: Sync + Send {
    fn get_potential(&self, r: f64) -> f64;
    fn get_force(&self, r: f64) -> f64;
    fn get_potential_and_force(&self, r: f64) -> (f64, f64);
}

pub fn update_force<T: Potential>(state: &mut State, potential: &T) {
    let number_particles = state.particles.len();

    state.particles.par_iter_mut().for_each(|particle| {
        let particle = particle.get_mut().expect("Can't lock particle");
        particle.force.x = 0.0;
        particle.force.y = 0.0;
        particle.force.z = 0.0;
        particle.potential = 0.0;
    });

    (0..number_particles).into_par_iter().for_each(|i| {
        for j in 0..i {
            let r = state.get_least_r(i, j);
            let (potential, force) = potential.get_potential_and_force(r.magnitude());
            let force_vec = r.normalize() * force;
            {
                let p1 = &mut state.particles[i].lock().expect("Can't lock particle");
                p1.force += force_vec;
                p1.potential += potential;
            }
            {
                let p2 = &mut state.particles[j].lock().expect("Can't lock particle");
                p2.force -= force_vec;
                p2.potential += potential;
            }
        }
    });
}
