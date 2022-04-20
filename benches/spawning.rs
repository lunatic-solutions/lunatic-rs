use criterion::{criterion_group, criterion_main, Criterion};
use lunatic::Task;

fn spawn_benchmark(c: &mut Criterion) {
    c.bench_function("task", |b| {
        b.iter(|| {
            // Spawn task and wait for it to finish.
            let task = Task::spawn_link(0, |input| input + 1);
            assert_eq!(task.result(), 1);
        })
    });
}

criterion_group!(benches, spawn_benchmark);
criterion_main!(benches);
