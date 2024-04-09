use criterion::{criterion_group, criterion_main, Criterion};

fn parser_benchmark(c: &mut Criterion) {
    c.bench_function("parse fibonacci", |b| {
        b.iter(|| {
            let input = r#"
                let fibonacci = fn(x) {
                    if (x < 2) {
                        return 1;
                    } else {
                        return fibonacci(x - 1) + fibonacci(x - 2);
                    }
                };
            "#;

            let _program = monkey_lang::parse::parse(input).unwrap();
        });
    });
}

criterion_group!(parser, parser_benchmark);
criterion_main!(parser);
