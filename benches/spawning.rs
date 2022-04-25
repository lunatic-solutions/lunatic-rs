use criterion::{criterion_group, criterion_main, Criterion};
use lunatic::spawn_link;

fn spawn_benchmark(c: &mut Criterion) {
    c.bench_function("task", |b| {
        b.iter(|| {
            // Spawn task and wait for it to finish.
            let task = spawn_link!(@task |input = 1| input + 1);
            assert_eq!(task.result(), 2);
        })
    });
}

criterion_group!(benches, spawn_benchmark);
criterion_main!(benches);
