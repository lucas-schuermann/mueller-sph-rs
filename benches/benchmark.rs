use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graphics;

fn sim(n: usize, i: usize) {
    let mut particles: Vec<graphics::Particle> = Vec::new();
    graphics::init_sph(&mut particles, n);
    for _ in 0..i {
        graphics::update(&mut particles);
    }
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-10");
    group.sample_size(10);
    group.bench_function("simulation: n=5000, i=100", |b| {
        b.iter(|| sim(black_box(5000), black_box(100)))
    });
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
