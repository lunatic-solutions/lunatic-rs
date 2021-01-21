//! Waiting on a criterion.rs issue to be resolved before we can use it with wasm32-wasi targets:
//! https://github.com/bheisler/criterion.rs/issues/461

use criterion::{criterion_group, criterion_main, Criterion, Fun};

fn fibonacci_slow(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci_slow(n - 1) + fibonacci_slow(n - 2),
    }
}

fn fibonacci_fast(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

fn lunatic_bench(c: &mut Criterion) {
    let fib_slow = Fun::new("Recursive", |b, i| b.iter(|| fibonacci_slow(*i)));
    let fib_fast = Fun::new("Iterative", |b, i| b.iter(|| fibonacci_fast(*i)));

    let functions = vec![fib_slow, fib_fast];

    c.bench_functions("Fibonacci", functions, 20);
}

criterion_group!(benches, lunatic_bench);
criterion_main!(benches);
