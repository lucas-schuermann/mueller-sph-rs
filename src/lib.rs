use std::f32::consts::PI;

use glam::Vec2;
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
pub const G: Vec2 = Vec2::from_array([0.0, -9.81]);

// manually write out powers since `f32::powf` is not yet a `const fn`
static POLY6: f32 = 4.0 / (PI * H * H * H * H * H * H * H * H);
static SPIKY_GRAD: f32 = -10.0 / (PI * H * H * H * H * H);
static VISC_LAP: f32 = 40.0 / (PI * H * H * H * H * H);

#[derive(Debug, Default)]
pub struct Simulation<const MAX_PARTICLES: usize> {
    pub view_width: f32,
    pub view_height: f32,

    pub num_particles: usize,
    pub x: Vec<Vec2>,
    v: Vec<Vec2>,
    f: Vec<Vec2>,
    rho: Vec<f32>,
    p: Vec<f32>,
}

impl<const MAX_PARTICLES: usize> Simulation<MAX_PARTICLES> {
    #[must_use]
    pub fn new(view_width: f32, view_height: f32) -> Self {
        Simulation {
            view_width,
            view_height,
            num_particles: 0,
            x: Vec::with_capacity(MAX_PARTICLES),
            v: Vec::with_capacity(MAX_PARTICLES),
            f: Vec::with_capacity(MAX_PARTICLES),
            rho: Vec::with_capacity(MAX_PARTICLES),
            p: Vec::with_capacity(MAX_PARTICLES),
        }
    }

    pub fn clear(&mut self) {
        self.num_particles = 0;
        self.x.clear();
        self.v.clear();
        self.f.clear();
        self.rho.clear();
        self.p.clear();
    }

    pub fn push_particle(&mut self, x: f32, y: f32) {
        self.num_particles += 1;
        self.x.push(Vec2::new(x, y));
        self.v.push(Vec2::ZERO);
        self.f.push(Vec2::ZERO);
        self.rho.push(1.0);
        self.p.push(0.0);
    }

    pub fn init_dam_break(&mut self, dam_max_particles: usize) {
        let mut placed = 0;
        let mut y = EPS;
        'outer: while y < (self.view_height - EPS * 2.0) {
            y += H;
            let mut x = self.view_width / 10.0;
            while x <= self.view_width / 2.5 {
                x += H;
                if placed == dam_max_particles || self.num_particles == MAX_PARTICLES {
                    break 'outer;
                }
                let jitter = random::<f32>();
                self.push_particle(x + jitter, y);
                placed += 1;
            }
        }
        info!("Initialized dam break with {placed} particles");
    }

    pub fn init_block(&mut self, max_block_particles: usize) {
        let mut placed = 0;
        let mut y = self.view_height / 1.5 - self.view_height / 10.0;
        'outer: while y < self.view_height / 1.5 + self.view_height / 10.0 {
            y += H * 0.95;
            let mut x = self.view_width / 2.0 - self.view_height / 10.0;
            while x < self.view_width / 2.0 + self.view_height / 10.0 {
                x += H * 0.95;
                if placed == max_block_particles || self.num_particles == MAX_PARTICLES {
                    break 'outer;
                }
                self.push_particle(x, y);
                placed += 1;
            }
        }
        info!(
            "Initialized block of {placed} particles, new total {}",
            self.num_particles
        );
    }

    pub fn integrate(&mut self) {
        self.x
            .par_iter_mut()
            .zip_eq(self.v.par_iter_mut())
            .enumerate()
            .for_each(|(i, (x, v))| {
                *v += DT * self.f[i] / self.rho[i];
                *x += DT * *v;

                // enforce boundary conditions
                if x.x - EPS < 0.0 {
                    v.x *= BOUND_DAMPING;
                    x.x = EPS;
                }
                if x.x + EPS > self.view_width {
                    v.x *= BOUND_DAMPING;
                    x.x = self.view_width - EPS;
                }
                if x.y - EPS < 0.0 {
                    v.y *= BOUND_DAMPING;
                    x.y = EPS;
                }
                if x.y + EPS > self.view_height {
                    v.y *= BOUND_DAMPING;
                    x.y = self.view_height - EPS;
                }
            });
    }

    pub fn compute_density_pressure(&mut self) {
        self.rho.par_iter_mut()
            .zip_eq(self.p.par_iter_mut())
            .enumerate()
            .for_each(|(i,(rho, p))| {
                *rho = 0.0;
                for j in 0..self.num_particles {
                    let rij = self.x[j] - self.x[i];
                    let r2 = rij.length_squared();
                    if r2 < HSQ {
                        *rho += MASS * POLY6 * f32::powf(HSQ - r2, 3.0);
                    }
                }
                *p = GAS_CONST * (*rho - REST_DENS);
            });
    }

    pub fn compute_forces(&mut self) {
        self.f.par_iter_mut().enumerate()
            .for_each(|(i, f)| {
                let mut fpress = Vec2::ZERO;
                let mut fvisc = Vec2::ZERO;
                for j in 0..self.num_particles {
                    if i == j {
                        continue;
                    }
                    let rij = self.x[j] - self.x[i];
                    let r = rij.length();
                    if r < H {
                        fpress += -rij.normalize() * MASS * (self.p[i] + self.p[j])
                            / (2.0 * self.rho[j])
                            * SPIKY_GRAD
                            * f32::powf(H - r, 3.0);
                        fvisc += VISC * MASS * (self.v[j] - self.v[i]) / self.rho[j]
                            * VISC_LAP
                            * (H - r);
                    }
                }
                let fgrav = G * MASS / self.rho[i];
                *f = fpress + fvisc + fgrav;
            });
    }

    pub fn update(&mut self) {
        self.compute_density_pressure();
        self.compute_forces();
        self.integrate();
    }
}
