use criterion::{criterion_group, criterion_main, Criterion};
use monkey::{evaluate::Evaluator, parse};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 30", |b| {
        b.iter(|| {
            let input = r#"
                let fibonacci = fn(x) {
                    if (x < 3) {
                        return 1;
                    } else {
                        return fibonacci(x - 1) + fibonacci(x - 2);
                    }
                };

                fibonacci(30)
            "#;

            let program = parse::parse(input).unwrap();

            let mut evaluator = Evaluator::new();
            let _result = evaluator.evaluate(&program).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
