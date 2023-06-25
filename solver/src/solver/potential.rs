use moldyn_core::Particle;

pub trait Potential {
    fn get_potential (&self, p1: &Particle, p2: &Particle) -> f64;
    fn get_force (&self, p1: &Particle, p2: &Particle) -> f64;
}