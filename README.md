# mueller-sph-rs
A concise 2D implementation of Müller's interactive smoothed particle hydrodynamics (SPH) paper in Rust

Reimplementation of my [previous C++ SPH](https://github.com/cerrno/mueller-sph) repository now including a parallel solver

Please see the original accompanying writeup [here](https://lucasschuermann.com/writing/implementing-sph-in-2d)

## Usage
Run with cargo:
```
RUST_LOG=info cargo r --release
```
Press `r` to reset simulation and `space` to add a block of particles

## License
[MIT](https://lucasschuermann.com/license.txt)
