use criterion::{black_box, criterion_group, criterion_main, Criterion};
use monkey_lang::{
    evaluate::{Environment, Evaluate},
    lexer::Lexer,
    parse::Parser,
};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 10", |b| {
        b.iter(|| {
            let input = r#"
            let fibonacci = fn(x) {
                if (x < 2) {
                    return 1;
                } else {
                    return fibonacci(x - 1) + fibonacci(x - 2);
                }
            };

            let collect = fn(f, n) {
                if (n == 0) {
                    [f(0)]
                } else {
                    push(collect(f, n - 1), f(n))
                }       
            }

            collect(fibonacci, 20);
            "#;

            let mut parser = Parser::new(Lexer::new(black_box(input)));
            let ast = parser.parse_program().unwrap();

            let mut environment = Environment::default();
            let _evaluated = ast.evaluate(&mut environment).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
