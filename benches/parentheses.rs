use compiler_utils::*;
use criterion::{criterion_group, criterion_main, Criterion};

fn parens<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<(), ParsingError<'filedata>> {
    alt(
        map(triple(tag("("), opt(accepts_while(parens)), tag(")")), |_| ()),
        map(triple(tag("["),opt(accepts_while(parens)), tag("]")), |_| ())
    )(walker)
}

fn stress_test() {
    let mut walker = FileWalker::from_data(include_str!("stress.txt"), "stress.txt");

    parens(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());
}

fn stress_benchmark(c: &mut Criterion) {
    c.bench_function("stress test", |b| b.iter(stress_test));
}

criterion_group!(benches, stress_benchmark);
criterion_main!(benches);