use criterion::{criterion_group, criterion_main, Criterion};

pub fn add_benchmark(c: &mut Criterion) {
    let mut rvg = ipvm::test_utils::Rvg::deterministic();
    let int_val_1 = rvg.sample(&(0..100i32));
    let int_val_2 = rvg.sample(&(0..100i32));

    c.bench_function("add", |b| {
        b.iter(|| {
            ipvm::add(int_val_1, int_val_2);
        })
    });
}

criterion_group!(benches, add_benchmark);
criterion_main!(benches);
