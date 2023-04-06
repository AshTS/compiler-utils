use compiler_utils::parser::*;


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

fn main() {
    stress_test();
}
