use compiler_utils::*;
use criterion::{criterion_group, criterion_main, Criterion};

fn opening_tag<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    map(triple(tag("<"), take_while(|c| c.is_ascii_lowercase() || c.is_ascii_uppercase(), "text"), tag(">")),
        |(_, tag_name, _)| tag_name)(walker)
}

fn closing_tag<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    map(triple(tag("</"), take_while(|c| c.is_ascii_lowercase() || c.is_ascii_uppercase(), "text"), tag(">")),
        |(_, tag_name, _)| tag_name)(walker)
}

fn tag_pair<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<(), ParsingError<'filedata>> {
    let open_text = opening_tag(walker)?;

    while tag_pair(walker).is_ok() {}

    let close_text = closing_tag(walker)?;

    if open_text.data != close_text.data {
        return Err(ParsingError(open_text.location, ErrorKind::DemoError));
    }

    Ok(())
}

fn stress_test() {
    let mut walker = FileWalker::from_data(include_str!("stress_html.txt"), "stress_html.txt");

    tag_pair(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());
}

fn html_ish_benchmark(c: &mut Criterion) {
    c.bench_function("html-ish", |b| b.iter(stress_test));
}

criterion_group!(benches, html_ish_benchmark);
criterion_main!(benches);