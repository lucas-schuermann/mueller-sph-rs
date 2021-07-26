#![warn(
    unreachable_pub,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    rust_2018_idioms,
    missing_debug_implementations
)]

use glam::Vec2;
use lazy_static::lazy_static;
use rand::random;
use rayon::prelude::*;
use std::f32::consts::PI;
use log::info;

#[derive(Debug, Clone, Copy, Default)]
pub struct Particle {
    x: Vec2,
    v: Vec2,
    f: Vec2,
    rho: f32,
    p: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: Vec2::new(x, y),
            ..Default::default()
        }
    }

    pub fn position(&self) -> Vec2 {
        self.x
    }
}

const REST_DENS: f32 = 300.0;
const GAS_CONST: f32 = 2000.0;
const H: f32 = 16.0;
const HSQ: f32 = H * H;
const MASS: f32 = 2.5;
const VISC: f32 = 200.0;
const DT: f32 = 0.0007;
const EPS: f32 = H;
const BOUND_DAMPING: f32 = -0.5;
pub const WINDOW_WIDTH: u32 = 800 * 2;
pub const WINDOW_HEIGHT: u32 = 600 * 2;
pub const VIEW_WIDTH: f32 = 1.5 * WINDOW_WIDTH as f32;
pub const VIEW_HEIGHT: f32 = 1.5 * WINDOW_HEIGHT as f32;
pub const G: Vec2 = glam::const_vec2!([0.0, -9.81]);

lazy_static! {
    static ref POLY6: f32 = 4.0 / (PI * f32::powf(H, 8.0));
    static ref SPIKY_GRAD: f32 = -10.0 / (PI * f32::powf(H, 5.0));
    static ref VISC_LAP: f32 = 40.0 / (PI * f32::powf(H, 5.0));
}

pub fn init_dam_break(particles: &mut Vec<Particle>, dam_max_particles: usize) {
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
    info!("Initialized dam break with {} particles", particles.len());
}

pub fn init_block(particles: &mut Vec<Particle>, max_block_particles: usize) {
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
    info!(
        "Initialized block of {} particles, new total {}",
        placed,
        particles.len()
    );
}

pub fn integrate(particles: &mut Vec<Particle>) {
    particles.par_iter_mut().for_each(|p| {
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
    })
}

pub fn compute_density_pressure(particles: &mut Vec<Particle>) {
    let particles_initial = particles.clone();
    particles.par_iter_mut().for_each(|pi| {
        let mut rho = 0.0f32;
        for pj in particles_initial.iter() {
            let rij = pj.x - pi.x;
            let r2 = rij.length_squared();
            if r2 < HSQ {
                rho += MASS * *POLY6 * f32::powf(HSQ - r2, 3.0);
            }
        }
        pi.rho = rho;
        pi.p = GAS_CONST * (pi.rho - REST_DENS);
    });
}

pub fn compute_forces(particles: &mut Vec<Particle>) {
    let particles_initial = particles.clone();
    particles.par_iter_mut().enumerate().for_each(|(i, pi)| {
        let mut fpress = Vec2::ZERO;
        let mut fvisc = Vec2::ZERO;
        for (j, pj) in particles_initial.iter().enumerate() {
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
        pi.f = fpress + fvisc + fgrav;
    });
}

pub fn update(particles: &mut Vec<Particle>) {
    compute_density_pressure(particles);
    compute_forces(particles);
    integrate(particles);
}
