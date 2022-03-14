use std::f32::consts::PI;

use arrayvec::ArrayVec;
use glam::Vec2;
use lazy_static::lazy_static;
use log::info;
use rand::random;
use rayon::prelude::*;

const REST_DENS: f32 = 300.0;
const GAS_CONST: f32 = 2000.0;
const H: f32 = 16.0;
const HSQ: f32 = H * H;
const MASS: f32 = 2.5;
const VISC: f32 = 200.0;
const DT: f32 = 0.0007;
const EPS: f32 = H;
const BOUND_DAMPING: f32 = -0.5;
pub const WINDOW_WIDTH: u32 = 1200;
pub const WINDOW_HEIGHT: u32 = 800;
pub const VIEW_WIDTH: f32 = 1.5 * WINDOW_WIDTH as f32;
pub const VIEW_HEIGHT: f32 = 1.5 * WINDOW_HEIGHT as f32;
pub const G: Vec2 = glam::const_vec2!([0.0, -9.81]);

lazy_static! {
    static ref POLY6: f32 = 4.0 / (PI * f32::powf(H, 8.0));
    static ref SPIKY_GRAD: f32 = -10.0 / (PI * f32::powf(H, 5.0));
    static ref VISC_LAP: f32 = 40.0 / (PI * f32::powf(H, 5.0));
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Particle {
    x: Vec2,
    v: Vec2,
    f: Vec2,
    rho: f32,
    p: f32,
}

impl Particle {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: Vec2::new(x, y),
            rho: 3000.0 / REST_DENS,
            ..Particle::default()
        }
    }

    #[must_use]
    pub fn position(&self) -> Vec2 {
        self.x
    }
}

#[derive(Debug, Default)]
pub struct State<const M: usize> {
    pub i: Box<ArrayVec<Particle, M>>,
    pub f: Box<ArrayVec<Particle, M>>,
}

impl<const M: usize> State<M> {
    #[must_use]
    pub fn new() -> Self {
        State {
            ..Default::default()
        }
    }

    pub fn init_dam_break(&mut self, dam_max_particles: usize) {
        let particles = &mut self.i;
        let mut y = EPS;
        'outer: while y < (VIEW_HEIGHT - EPS * 2.0) {
            y += H;
            let mut x = VIEW_WIDTH / 10.0;
            while x <= VIEW_WIDTH / 2.5 {
                x += H;
                if particles.len() < dam_max_particles {
                    let jitter = random::<f32>();
                    particles.push(Particle::new(x + jitter, y));
                } else {
                    break 'outer;
                }
            }
        }
        self.f.clone_from(particles);
        info!("Initialized dam break with {} particles", particles.len());
    }

    pub fn init_block(&mut self, max_block_particles: usize) {
        let particles = &mut self.i;
        let mut placed = 0;
        let mut y = VIEW_HEIGHT / 1.5 - VIEW_HEIGHT / 10.0;
        'outer: while y < VIEW_HEIGHT / 1.5 + VIEW_HEIGHT / 10.0 {
            y += H * 0.95;
            let mut x = VIEW_WIDTH / 2.0 - VIEW_HEIGHT / 10.0;
            while x < VIEW_WIDTH / 2.0 + VIEW_HEIGHT / 10.0 {
                x += H * 0.95;
                if placed < max_block_particles {
                    particles.push(Particle::new(x, y));
                    placed += 1;
                } else {
                    break 'outer;
                }
            }
        }
        self.f.clone_from(particles);
        info!(
            "Initialized block of {} particles, new total {}",
            placed,
            particles.len()
        );
    }

    pub fn integrate(&mut self) {
        self.f.par_iter_mut().for_each(|p| {
            p.v += DT * p.f / p.rho;
            p.x += DT * p.v;

            // enforce boundary conditions
            if p.x.x - EPS < 0.0 {
                p.v.x *= BOUND_DAMPING;
                p.x.x = EPS;
            }
            if p.x.x + EPS > VIEW_WIDTH {
                p.v.x *= BOUND_DAMPING;
                p.x.x = VIEW_WIDTH - EPS;
            }
            if p.x.y - EPS < 0.0 {
                p.v.y *= BOUND_DAMPING;
                p.x.y = EPS;
            }
            if p.x.y + EPS > VIEW_HEIGHT {
                p.v.y *= BOUND_DAMPING;
                p.x.y = VIEW_HEIGHT - EPS;
            }
        });
    }

    pub fn compute_density_pressure(&mut self) {
        let si = &self.i;
        let sf = &mut self.f;
        si.par_iter().zip(sf.par_iter_mut()).for_each(|(pi, pf)| {
            let mut rho = 0.0f32;
            for pj in si.iter() {
                let rij = pj.x - pi.x;
                let r2 = rij.length_squared();
                if r2 < HSQ {
                    rho += MASS * *POLY6 * f32::powf(HSQ - r2, 3.0);
                }
            }
            pf.rho = rho;
            pf.p = GAS_CONST * (pi.rho - REST_DENS);
        });
    }

    pub fn compute_forces(&mut self) {
        let si = &self.i;
        let sf = &mut self.f;
        si.par_iter()
            .zip(sf.par_iter_mut())
            .enumerate()
            .for_each(|(i, (pi, pf))| {
                let mut fpress = Vec2::ZERO;
                let mut fvisc = Vec2::ZERO;
                for (j, pj) in si.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let rij = pj.x - pi.x;
                    let r = rij.length();
                    if r < H {
                        fpress += -rij.normalize() * MASS * (pi.p + pj.p) / (2.0 * pj.rho)
                            * *SPIKY_GRAD
                            * f32::powf(H - r, 3.0);
                        fvisc += VISC * MASS * (pj.v - pi.v) / pj.rho * *VISC_LAP * (H - r);
                    }
                }
                let fgrav = G * MASS / pi.rho;
                pf.f = fpress + fvisc + fgrav;
            });
    }

    pub fn update(&mut self) {
        self.compute_density_pressure();
        self.compute_forces();
        self.integrate();
        self.i.clone_from(&self.f);
    }
}
