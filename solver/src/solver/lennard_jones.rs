use crate::solver::Potential;
use rand_distr::num_traits::Pow;

pub struct LennardJonesPotential {
    sigma: f64,
    eps: f64,
    r_cut: f64,
    u_cut: f64,
}

impl Potential for LennardJonesPotential {
    fn get_potential(&self, r: f64) -> f64 {
        if r > self.r_cut {
            return 0.0;
        }
        let sigma_r = self.sigma / r;
        let sigma_r_6 = sigma_r.pow(6);
        let sigma_r_12 = sigma_r_6 * sigma_r_6;
        4.0f64 * self.eps * (sigma_r_12 - sigma_r_6) - self.u_cut
    }

    fn get_force(&self, r: f64) -> f64 {
        if r > self.r_cut {
            return 0.0;
        }
        let sigma_r = self.sigma / r;
        let sigma_r_6 = sigma_r.pow(6);
        let sigma_r_12 = sigma_r_6 * sigma_r_6;
        (24.0f64 * self.eps / r) * (sigma_r_6 - 2.0f64 * sigma_r_12)
    }

    fn get_potential_and_force(&self, r: f64) -> (f64, f64) {
        if r > self.r_cut {
            return (0.0, 0.0);
        }
        let sigma_r = self.sigma / r;
        let sigma_r_6 = sigma_r.pow(6);
        let sigma_r_12 = sigma_r_6 * sigma_r_6;
        (
            4.0f64 * self.eps * (sigma_r_12 - sigma_r_6) - self.u_cut,
            (24.0f64 * self.eps / r) * (sigma_r_6 - 2.0f64 * sigma_r_12),
        )
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
        potential.u_cut = potential.get_potential(r_cut);
        potential
    }
}
