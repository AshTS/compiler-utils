use compiler_utils::*;

fn asc_symbol<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    one_of("!#$%&*+./<=>?@\\^|=~:")(walker)
}

fn asc_digit<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    one_of("0123456789")(walker)
}

#[allow(dead_code)]
fn asc_octit<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    one_of("01234567")(walker)
}

#[allow(dead_code)]
fn asc_hexit<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    one_of("0123456789ABCDEFabcdef")(walker)
}

fn digit<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    asc_digit(walker)
}

fn asc_large<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    take_if(|c| c.is_ascii_uppercase(), "ASCII Uppercase")(walker)
}

fn uni_large<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    but_not(
        take_if(|c| c.is_uppercase(), "Any Uppercase"),
        take_if(|c| c.is_ascii_uppercase(), "ASCII Uppercase"))(walker)
}

fn asc_small<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    take_if(|c| c.is_ascii_lowercase(), "ASCII Lowercase")(walker)
}

fn uni_small<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    but_not(
        take_if(|c| c.is_lowercase(), "Any Lowercase"),
        take_if(|c| c.is_ascii_lowercase(), "ASCII Lowercase"))(walker)
}

fn large<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    alt(asc_large, uni_large)(walker)
}

fn small<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    alt(asc_small, uni_small)(walker)
}

fn special<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    one_of("(),;[]`{}")(walker)
}

fn symbol<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    but_not(
        asc_symbol,
        alt(
            special,
            one_of("_\"'")
        )
    )(walker)
}

fn graphic<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    alt(
        alt(one_of("\"'"),
            special),
        alt(
            alt(small, large),
            alt(symbol, digit))
    )(walker)
}

fn any<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    alt(graphic, one_of("\t "))(walker)
}

fn aany<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    alt(any, white_char)(walker)
}

fn newline<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    alt(tag("\r\n"), one_of("\r\n"))(walker)
}

fn white_char<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    take_if(|c| c.is_whitespace(), "whitespace")(walker)
}

fn opencom<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    tag("{-")(walker)
}

fn closecom<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    tag("-}")(walker)
}

fn dashes<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    let start = walker.get_marker();
    tag("--")(walker)?;

    while let Ok(Some(_)) = opt(tag("-"))(walker) {}

    Ok(walker.span_from_marker_to_here(start).unwrap())
}

fn comment<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    accepts(
        triple(
            dashes,
            opt(
                pair(
                    but_not(any, symbol),
                    accepts_while(any)
                )
            ),
            newline
        )
    )(walker)
}

fn ncomment<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    accepts(
        triple(
            opencom,
                opt(accepts_while(
                    alt(
                            ncomment,
                                
                            alt(
                                but_not(aany, tag("{")),
                                accepts(pair(tag("{"), but_not(aany, tag("-")))),
                            ))
                )),
            closecom
        )
    )(walker)
}

#[test]
fn test_comment() {
    let mut walker = FileWalker::from_data("--- :Hello World   \r\n", "input");

    comment(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());
}

#[test]
fn test_ncomment() {
    let mut walker = FileWalker::from_data("{- Hello {- World! } -} {-  \n Hi\r\n      -}  dsfds f {--} -}", "input");

    let _ = ncomment(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());
}