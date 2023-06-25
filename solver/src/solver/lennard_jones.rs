use rand_distr::num_traits::Pow;
use moldyn_core::Particle;
use crate::solver::Potential;

pub struct LennardJonesPotential {
    sigma: f64,
    eps: f64,
    r_cut: f64,
    u_cut: f64,
}

impl Potential for LennardJonesPotential {
    fn get_potential(&self, p1: &Particle, p2: &Particle) -> f64 {
        let r = (p1.position - p2.position).magnitude();
        if r > self.r_cut {
            return 0.0;
        }
        let sigma_r = self.sigma / r;
        let sigma_r_6 = sigma_r.pow(6);
        let sigma_r_12 = sigma_r_6 * sigma_r_6;
        return 4.0f64 * self.eps * (sigma_r_12 - sigma_r_6) - self.u_cut;
    }

    fn get_force(&self, p1: &Particle, p2: &Particle) -> f64 {
        let r = (p1.position - p2.position).magnitude();
        let sigma_r = self.sigma / r;
        let sigma_r_6 = sigma_r.pow(6);
        let sigma_r_12 = sigma_r_6 * sigma_r_6;
        return (24.0f64 * self.eps / r) * (2.0f64 * sigma_r_12 - sigma_r_6);
    }
}

impl LennardJonesPotential {
    pub fn new(sigma: f64, eps: f64) -> Self {
        let r_cut = sigma * 2.5;
        let mut potential = Self {
            sigma,
            eps,
            r_cut,
            u_cut: 0.0,
        };
        let p1 = Particle::new();
        let mut p2 = Particle::new();
        p2.position.x = r_cut;
        let u_cut = potential.get_potential(&p1, &p2);
        potential.u_cut = u_cut;
        potential
    }
}