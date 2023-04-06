use crate::{ErrorKind, FileWalker, ParsingError, Span};

#[inline]
pub fn map<'filedata, Input, Output>(
    combinator: impl Fn(&mut FileWalker<'filedata>) -> Result<Input, ParsingError<'filedata>>,
    f: impl Fn(Input) -> Output,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<Output, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let v = combinator(walker)?;
        Ok(f(v))
    }
}

#[inline]
pub fn pair<'filedata, A, B>(
    first: impl Fn(&mut FileWalker<'filedata>) -> Result<A, ParsingError<'filedata>>,
    second: impl Fn(&mut FileWalker<'filedata>) -> Result<B, ParsingError<'filedata>>,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<(A, B), ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let start = walker.get_marker();

        let value_a = first(walker)?;

        match second(walker) {
            Err(e) => {
                walker.pop_back(start);
                Err(e)
            }
            Ok(value_b) => Ok((value_a, value_b)),
        }
    }
}

#[inline]
pub fn triple<'filedata, A, B, C>(
    first: impl Fn(&mut FileWalker<'filedata>) -> Result<A, ParsingError<'filedata>>,
    second: impl Fn(&mut FileWalker<'filedata>) -> Result<B, ParsingError<'filedata>>,
    third: impl Fn(&mut FileWalker<'filedata>) -> Result<C, ParsingError<'filedata>>,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<(A, B, C), ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let start = walker.get_marker();

        let value_a = first(walker)?;

        let value_b = match second(walker) {
            Err(e) => {
                walker.pop_back(start);
                return Err(e);
            }
            Ok(value_b) => value_b,
        };

        match third(walker) {
            Err(e) => {
                walker.pop_back(start);
                Err(e)
            }
            Ok(value_c) => Ok((value_a, value_b, value_c)),
        }
    }
}

#[inline]
pub fn opt<'filedata, A>(
    first: impl Fn(&mut FileWalker<'filedata>) -> Result<A, ParsingError<'filedata>>,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<Option<A>, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| Ok(first(walker).ok())
}

#[inline]
pub fn alt<'filedata, A>(
    first: impl Fn(&mut FileWalker<'filedata>) -> Result<A, ParsingError<'filedata>>,
    second: impl Fn(&mut FileWalker<'filedata>) -> Result<A, ParsingError<'filedata>>,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<A, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        if let Ok(value) = first(walker) {
            Ok(value)
        } else {
            second(walker)
        }
    }
}

#[inline]
/// Accepts input that satisfies the first parser, but not the second, returns the result of the first
pub fn but_not<'filedata, A, B>(
    first: impl Fn(&mut FileWalker<'filedata>) -> Result<A, ParsingError<'filedata>>,
    second: impl Fn(&mut FileWalker<'filedata>) -> Result<B, ParsingError<'filedata>>,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<A, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let start = walker.get_marker();
        let value = first(walker)?;

        let span = walker.span_from_marker_to_here(start).unwrap();
        let mut walker_of_first = FileWalker::from_span(&span);
        let second_start = walker_of_first.get_marker();

        if second(&mut walker_of_first).is_ok() {
            let second_span = walker_of_first
                .span_from_marker_to_here(second_start)
                .unwrap();
            if second_span.data == span.data {
                return Err(ParsingError(
                    walker.get_location_of_marker(start).unwrap(),
                    ErrorKind::InverseFailedGot(span.data),
                ));
            }
        }

        Ok(value)
    }
}

#[inline]
/// Returns the span of anything that accepts the wrapped parser
pub fn accepts<'filedata, T>(
    combinator: impl Fn(&mut FileWalker<'filedata>) -> Result<T, ParsingError<'filedata>>,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let start = walker.get_marker();
        combinator(walker)?;
        Ok(walker.span_from_marker_to_here(start).unwrap())
    }
}

#[inline]
/// Returns the span of anything that accepts any count of the wrapped parser
pub fn accepts_while<'filedata, T>(
    combinator: impl Fn(&mut FileWalker<'filedata>) -> Result<T, ParsingError<'filedata>>,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let start = walker.get_marker();
        combinator(walker)?;
        while combinator(walker).is_ok() {}
        Ok(walker.span_from_marker_to_here(start).unwrap())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        accepts_while, alt, but_not, map, one_of, opt, pair, tag, take_while, triple, ErrorKind,
        FileWalker, Location, ParsingError, take_if,
    };

    #[test]
    fn map_ok() {
        fn comb<'filedata>(
            walker: &mut FileWalker<'filedata>,
        ) -> Result<usize, ParsingError<'filedata>> {
            Ok(walker.current_string().len())
        }

        assert_eq!(comb(&mut FileWalker::from_data("Hi!", "input")), Ok(3));
        assert_eq!(comb(&mut FileWalker::from_data("Hello!", "input")), Ok(6));

        assert_eq!(
            map(comb, |v| v + 3)(&mut FileWalker::from_data("Hi!", "input")),
            Ok(6)
        );
        assert_eq!(
            map(comb, |v| v + 3)(&mut FileWalker::from_data("Hello!", "input")),
            Ok(9)
        );
    }

    #[test]
    fn map_failure() {
        fn comb<'filedata>(
            walker: &mut FileWalker<'filedata>,
        ) -> Result<usize, ParsingError<'filedata>> {
            Err(ParsingError(
                walker.current_location(),
                crate::ErrorKind::DemoError,
            ))
        }

        assert_eq!(
            map(comb, |v| v + 3)(&mut FileWalker::from_data("Hi!", "input")).unwrap_err(),
            comb(&mut FileWalker::from_data("Hi!", "input")).unwrap_err()
        );
        assert_eq!(
            map(comb, |v| v + 3)(&mut FileWalker::from_data("Hello!", "input")).unwrap_err(),
            comb(&mut FileWalker::from_data("Hello!", "input")).unwrap_err()
        );
    }

    #[test]
    fn pair_ok() {
        let comb_a = tag("Hello");
        let comb_b = tag("World");
        let comb_c = tag("!");

        let (a, b) = pair(&comb_a, &comb_c)(&mut FileWalker::from_data("Hello!", "input")).unwrap();
        assert_eq!(a.data, "Hello");
        assert_eq!(b.data, "!");

        let (a, b) = pair(&comb_b, &comb_c)(&mut FileWalker::from_data("World!", "input")).unwrap();
        assert_eq!(a.data, "World");
        assert_eq!(b.data, "!");

        let (a, b) =
            pair(&comb_a, &comb_b)(&mut FileWalker::from_data("HelloWorld!", "input")).unwrap();
        assert_eq!(a.data, "Hello");
        assert_eq!(b.data, "World");
    }

    #[test]
    fn pair_failure() {
        let comb_a = tag("Hello");
        let comb_b = tag("World");
        let comb_c = tag("!");

        assert_eq!(
            pair(&comb_a, &comb_b)(&mut FileWalker::from_data("Hello !", "input")),
            Err(ParsingError(
                Location::from_components(5, 0, "input"),
                ErrorKind::ExpectedTag("World")
            ))
        );

        assert_eq!(
            pair(&comb_b, &comb_c)(&mut FileWalker::from_data("Hello !", "input")),
            Err(ParsingError(
                Location::from_components(0, 0, "input"),
                ErrorKind::ExpectedTag("World")
            ))
        );

        assert_eq!(
            pair(&comb_a, &comb_b)(&mut FileWalker::from_data("Hello", "input")),
            Err(ParsingError(
                Location::from_components(5, 0, "input"),
                ErrorKind::ExpectedTag("World")
            ))
        );
    }

    #[test]
    fn triple_ok() {
        let comb_a = tag("Hello");
        let comb_b = tag("World");
        let comb_c = tag(" ");

        let (a, b, c) =
            triple(&comb_a, &comb_c, &comb_b)(&mut FileWalker::from_data("Hello World", "input"))
                .unwrap();
        assert_eq!(a.data, "Hello");
        assert_eq!(b.data, " ");
        assert_eq!(c.data, "World");
    }

    #[test]
    fn triple_failure() {
        let comb_a = tag("Hello");
        let comb_b = tag("World");
        let comb_c = tag(" ");

        assert_eq!(
            triple(&comb_a, &comb_c, &comb_b)(&mut FileWalker::from_data("hello World", "input")),
            Err(ParsingError(
                Location::from_components(0, 0, "input"),
                ErrorKind::ExpectedTag("Hello")
            ))
        );

        assert_eq!(
            triple(&comb_a, &comb_c, &comb_b)(&mut FileWalker::from_data("Hello_World", "input")),
            Err(ParsingError(
                Location::from_components(5, 0, "input"),
                ErrorKind::ExpectedTag(" ")
            ))
        );

        assert_eq!(
            triple(&comb_a, &comb_c, &comb_b)(&mut FileWalker::from_data("Hello world", "input")),
            Err(ParsingError(
                Location::from_components(6, 0, "input"),
                ErrorKind::ExpectedTag("World")
            ))
        );

        assert_eq!(
            triple(&comb_a, &comb_c, &comb_b)(&mut FileWalker::from_data("Hello ", "input")),
            Err(ParsingError(
                Location::from_components(6, 0, "input"),
                ErrorKind::ExpectedTag("World")
            ))
        );
    }

    #[test]
    fn opt_ok() {
        let comb_a = tag("Hello");

        let v = opt(&comb_a)(&mut FileWalker::from_data("Hello World", "input"))
            .unwrap()
            .unwrap();
        assert_eq!(v.data, "Hello");

        assert!(opt(&comb_a)(&mut FileWalker::from_data("World", "input"))
            .unwrap()
            .is_none())
    }

    #[test]
    fn alt_ok() {
        let comb_a = tag("Hello");
        let comb_b = tag("World");

        let v = alt(&comb_a, &comb_b)(&mut FileWalker::from_data("Hello World", "input")).unwrap();
        assert_eq!(v.data, "Hello");

        let v = alt(&comb_a, &comb_b)(&mut FileWalker::from_data("World Hello", "input")).unwrap();
        assert_eq!(v.data, "World");
    }

    #[test]
    fn alt_err() {
        let comb_a = tag("Hello");
        let comb_b = tag("World");

        assert_eq!(
            alt(&comb_a, &comb_b)(&mut FileWalker::from_data("hello World", "input")),
            Err(ParsingError(
                Location::from_components(0, 0, "input"),
                ErrorKind::ExpectedTag("World")
            ))
        );
    }

    #[test]
    fn but_not_ok() {
        let comb_a = take_while(|c| c.is_uppercase(), "uppercase");
        let comb_b = one_of("HW");

        let v = but_not(&comb_a, &comb_b)(&mut FileWalker::from_data("Balcony", "input")).unwrap();
        assert_eq!(v.data, "B");

        let comb_a = take_while(|c| c.is_uppercase(), "uppercase");
        let comb_b = one_of("HW");

        let v =
            but_not(&comb_a, &comb_b)(&mut FileWalker::from_data("HEllo World!", "input")).unwrap();
        assert_eq!(v.data, "HE");
    }

    #[test]
    fn but_not_err() {
        let comb_a = take_while(|c| c.is_uppercase(), "uppercase");
        let comb_b = one_of("HW");

        assert_eq!(
            but_not(&comb_a, &comb_b)(&mut FileWalker::from_data("Hello", "input")),
            Err(ParsingError(
                Location::from_components(0, 0, "input"),
                ErrorKind::InverseFailedGot("H")
            ))
        );

        let comb_a = take_while(|c| c.is_uppercase(), "uppercase");
        let comb_b = take_while(|c| c == 'H' || c == 'W', "'H' or 'W'");

        assert_eq!(
            but_not(&comb_a, &comb_b)(&mut FileWalker::from_data("HWllo", "input")),
            Err(ParsingError(
                Location::from_components(0, 0, "input"),
                ErrorKind::InverseFailedGot("HW")
            ))
        );
    }

    #[test]
    fn accepts_while_ok() {
        let comb = alt(tag("Ba"), tag("lc"));

        let v = accepts_while(&comb)(&mut FileWalker::from_data("Balcony", "input")).unwrap();
        assert_eq!(v.data, "Balc");

        let comb = take_if(|c| c.is_ascii_uppercase(), "uppercase");

        let v = accepts_while(&comb)(&mut FileWalker::from_data("HARmony", "input")).unwrap();
        assert_eq!(v.data, "HAR");

        let v = accepts_while(&comb)(&mut FileWalker::from_data("Below", "input")).unwrap();
        assert_eq!(v.data, "B");
    }

    #[test]
    fn accepts_while_err() {
        let comb = alt(tag("Balance"), tag("alcony"));

        assert_eq!(
            accepts_while(&comb)(&mut FileWalker::from_data("Balcony", "input")),
            Err(ParsingError(
                Location::from_components(0, 0, "input"),
                ErrorKind::ExpectedTag("alcony")
            ))
        );

        let comb = take_if(|c| c.is_uppercase(), "uppercase");

        assert_eq!(
            accepts_while(&comb)(&mut FileWalker::from_data("bALCONY", "input")),
            Err(ParsingError(
                Location::from_components(0, 0, "input"),
                ErrorKind::ExpectedOneOfKind("uppercase")
            ))
        );
    }

    #[test]
    fn accepts_ok() {
        let comb_a = tag("Hello");
        let comb_b = tag("World");
        let comb_c = tag("!");

        let (a, b) = pair(&comb_a, &comb_c)(&mut FileWalker::from_data("Hello!", "input")).unwrap();
        assert_eq!(a.data, "Hello");
        assert_eq!(b.data, "!");

        let (a, b) = pair(&comb_b, &comb_c)(&mut FileWalker::from_data("World!", "input")).unwrap();
        assert_eq!(a.data, "World");
        assert_eq!(b.data, "!");

        let (a, b) =
            pair(&comb_a, &comb_b)(&mut FileWalker::from_data("HelloWorld!", "input")).unwrap();
        assert_eq!(a.data, "Hello");
        assert_eq!(b.data, "World");
    }

    #[test]
    fn accepts_failure() {
        let comb_a = tag("Hello");
        let comb_b = tag("World");
        let comb_c = tag("!");

        assert_eq!(
            pair(&comb_a, &comb_b)(&mut FileWalker::from_data("Hello !", "input")),
            Err(ParsingError(
                Location::from_components(5, 0, "input"),
                ErrorKind::ExpectedTag("World")
            ))
        );

        assert_eq!(
            pair(&comb_b, &comb_c)(&mut FileWalker::from_data("Hello !", "input")),
            Err(ParsingError(
                Location::from_components(0, 0, "input"),
                ErrorKind::ExpectedTag("World")
            ))
        );

        assert_eq!(
            pair(&comb_a, &comb_b)(&mut FileWalker::from_data("Hello", "input")),
            Err(ParsingError(
                Location::from_components(5, 0, "input"),
                ErrorKind::ExpectedTag("World")
            ))
        );
    }
}
