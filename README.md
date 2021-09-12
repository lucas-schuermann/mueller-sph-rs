# mueller-sph-rs
A concise 2D implementation of Müller's "Particle-Based Fluid Simulation for Interactive Applications" (SPH) [paper](https://matthias-research.github.io/pages/publications/sca03.pdf) in Rust

Reimplementation of my [previous C++ SPH](https://github.com/cerrno/mueller-sph) repository now including a parallel solver using [Rayon](https://github.com/rayon-rs/rayon)

Please see the original accompanying [tutorial](https://lucasschuermann.com/writing/implementing-sph-in-2d) for more information.

## Usage
Run with cargo:
```
RUST_LOG=info cargo r --release
```
Press `r` to reset simulation or `space` to add a block of particles
