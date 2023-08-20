`mueller-sph-rs` is a concise 2D implementation of Müller's "Particle-Based Fluid Simulation for Interactive Applications" (SPH) [paper](https://matthias-research.github.io/pages/publications/sca03.pdf) in Rust.

This is a reimplementation of my [previous C++ SPH](https://github.com/lucas-schuermann/mueller-sph) repository now including a parallel solver using [Rayon](https://github.com/rayon-rs/rayon). Please see the original accompanying [tutorial](https://lucasschuermann.com/writing/implementing-sph-in-2d) for more information.

## Running
```bash
# install dependencies (debian/ubuntu)
apt install build-essential pkg-config cmake libfreetype6-dev libfontconfig1-dev

# build and launch demo
RUST_LOG=info cargo run --release
```
Press `r` to reset simulation or `space` to add a block of particles

## License
This project is distributed under the [MIT license](LICENSE.md).