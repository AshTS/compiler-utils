use crate::{FileWalker, Span, ParsingError, ErrorKind};

#[inline]
pub fn tag<'filedata>(s: &'static str) -> impl Fn(&mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let start = walker.get_marker();

        for c in s.chars() {
            if walker.step() != Some(c) {
                walker.pop_back(start);
                return Err(ParsingError(walker.get_location_of_marker(start).unwrap(), ErrorKind::ExpectedTag(s)));
            }
        }

        Ok(walker.span_from_marker_to_here(start).unwrap())
    }
}

#[inline]
pub fn one_of<'filedata>(s: &'static str)  -> impl Fn(&mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let start = walker.get_marker();

        if let Some(c) = walker.step() {
            if s.contains(c) {
                return Ok(walker.span_from_marker_to_here(start).unwrap());
            }
        }

        walker.pop_back(start);

        Err(ParsingError(walker.get_location_of_marker(start).unwrap(), ErrorKind::ExpectedOneOf(s)))
    }
}

#[inline]
pub fn take_while<'filedata>(
    f: impl Fn(char) -> bool, kind: &'static str
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let s = walker.current_string();
        let start = walker.get_marker();

        for c in s.chars() {
            if !f(c) {
                break;
            }
            walker.step();
        }

        if walker.get_marker() == start {
            Err(ParsingError(walker.current_location(), ErrorKind::ExpectedKind(kind)))
        }
        else {
            Ok(walker.span_from_marker_to_here(start).unwrap())
        }
    }
}

#[inline]
pub fn take_if<'filedata>(
    f: impl Fn(char) -> bool, kind: &'static str
)  -> impl Fn(&mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        let start = walker.get_marker();

        if let Some(c) = walker.step() {
            if f(c) {
                return Ok(walker.span_from_marker_to_here(start).unwrap());
            }
        }

        walker.pop_back(start);

        Err(ParsingError(walker.get_location_of_marker(start).unwrap(), ErrorKind::ExpectedOneOfKind(kind)))
    }
}

#[cfg(test)]
mod test {
    use crate::{FileWalker, Location, Span, tag, ParsingError, ErrorKind, take_while, one_of, take_if};

    #[test]
    fn tag_ok() {
        let mut walker = FileWalker::from_data("Hellö World!", "test.txt");

        // Make sure that tag will accept the presence of its value at the beginning of the incoming string
        assert_eq!(tag("Hellö")(&mut walker), Ok(Span::from_components(
            Location::from_components(0, 0, "test.txt"),
            "Hellö"
        )));

        // And make sure it stops right at the end of the tag
        assert_eq!(walker.current_string(), " World!");
    }

    #[test]
    fn tag_failure() {
        let mut walker = FileWalker::from_data("Hello World!", "test.txt");

        // Make sure that tag will reject a failed tag find
        assert_eq!(tag("World")(&mut walker), Err(ParsingError(
            Location::from_components(0, 0, "test.txt"),
            ErrorKind::ExpectedTag("World")
        )));

        // And make sure it returns the walker to its original state
        assert_eq!(walker.current_string(), "Hello World!");
    }

    #[test]
    fn tag_partial_failure() {
        let mut walker = FileWalker::from_data("High beams", "test.txt");

        // Make sure that tag will reject a tag which it starts to match
        assert_eq!(tag("Highway")(&mut walker), Err(ParsingError(
            Location::from_components(0, 0, "test.txt"),
            ErrorKind::ExpectedTag("Highway")
        )));

        // And make sure it returns the walker to its original state
        assert_eq!(walker.current_string(), "High beams");
    }

    #[test]
    fn one_of_ok() {
        let mut walker = FileWalker::from_data("Hello World!", "test.txt");

        // Make sure that tag will accept the presence of ia valid character
        assert_eq!(one_of("HAI")(&mut walker), Ok(Span::from_components(
            Location::from_components(0, 0, "test.txt"),
            "H"
        )));

        // And make sure it stops right at the end of the tag
        assert_eq!(walker.current_string(), "ello World!");

        let mut walker = FileWalker::from_data("Alphabet World!", "test.txt");

        // Make sure that tag will accept the presence of ia valid character
        assert_eq!(one_of("HAI")(&mut walker), Ok(Span::from_components(
            Location::from_components(0, 0, "test.txt"),
            "A"
        )));

        // And make sure it stops right at the end of the tag
        assert_eq!(walker.current_string(), "lphabet World!");
    }

    #[test]
    fn one_of_failure() {
        let mut walker = FileWalker::from_data("Hello World!", "test.txt");

        // Make sure that tag will reject a failed tag find
        assert_eq!(one_of("World")(&mut walker), Err(ParsingError(
            Location::from_components(0, 0, "test.txt"),
            ErrorKind::ExpectedOneOf("World")
        )));

        // And make sure it returns the walker to its original state
        assert_eq!(walker.current_string(), "Hello World!");
    }

    #[test]
    fn take_while_ok() {
        let mut walker = FileWalker::from_data("HEllo", "test.txt");

        // Make sure that tag will only take the capital letters
        assert_eq!(take_while(|c: char| c.is_uppercase(), "uppercase")(&mut walker), Ok(Span::from_components(
            Location::from_components(0, 0, "test.txt"),
            "HE"
        )));

        // And make sure it stops right at the end of the capitalized input
        assert_eq!(walker.current_string(), "llo");

        let mut walker = FileWalker::from_data("  \t\n\n  \r\n Hi", "test.txt");

        // Make sure that tag will only take the whitespace
        assert_eq!(take_while(|c: char| c.is_whitespace(), "lowercase")(&mut walker), Ok(Span::from_components(
            Location::from_components(0, 0, "test.txt"),
            "  \t\n\n  \r\n "
        )));

        // And make sure it stops right at the end of the whitespace
        assert_eq!(walker.current_string(), "Hi");
    }

    #[test]
    fn take_while_failure() {
        let mut walker = FileWalker::from_data("hello", "test.txt");

        // Make sure that tag will only take the capital letters
        assert_eq!(take_while(|c: char| c.is_uppercase(), "uppercase")(&mut walker), Err(ParsingError(
            Location::from_components(0, 0, "test.txt"),
            ErrorKind::ExpectedKind("uppercase")
        )));

        // And make sure it keeps the original text
        assert_eq!(walker.current_string(), "hello");

        let mut walker = FileWalker::from_data("This  \t\n\n  \r\n Hi", "test.txt");

        // Make sure that tag will only take the whitespace
        assert_eq!(take_while(|c: char| c.is_whitespace(), "whitespace")(&mut walker), Err(ParsingError(
            Location::from_components(0, 0, "test.txt"),
            ErrorKind::ExpectedKind("whitespace")
        )));

        // And make sure it keeps the original text
        assert_eq!(walker.current_string(), "This  \t\n\n  \r\n Hi");
    }

    #[test]
    fn take_if_ok() {
        let mut walker = FileWalker::from_data("HEllo", "test.txt");

        // Make sure that tag will only take a capital letter
        assert_eq!(take_if(|c: char| c.is_uppercase(), "uppercase")(&mut walker), Ok(Span::from_components(
            Location::from_components(0, 0, "test.txt"),
            "H"
        )));

        // And make sure it stops after the first capitalized letter
        assert_eq!(walker.current_string(), "Ello");

        let mut walker = FileWalker::from_data("  \t\n\n  \r\n Hi", "test.txt");

        // Make sure that tag will only take the whitespace
        assert_eq!(take_if(|c: char| c.is_whitespace(), "lowercase")(&mut walker), Ok(Span::from_components(
            Location::from_components(0, 0, "test.txt"),
            " "
        )));

        // And make sure it stops after the first whitespace character
        assert_eq!(walker.current_string(), " \t\n\n  \r\n Hi");
    }

    #[test]
    fn take_if_failure() {
        let mut walker = FileWalker::from_data("hello", "test.txt");

        // Make sure that tag will only take the capital letters
        assert_eq!(take_if(|c: char| c.is_uppercase(), "uppercase")(&mut walker), Err(ParsingError(
            Location::from_components(0, 0, "test.txt"),
            ErrorKind::ExpectedOneOfKind("uppercase")
        )));

        // And make sure it keeps the original text
        assert_eq!(walker.current_string(), "hello");

        let mut walker = FileWalker::from_data("This  \t\n\n  \r\n Hi", "test.txt");

        // Make sure that tag will only take the whitespace
        assert_eq!(take_if(|c: char| c.is_whitespace(), "whitespace")(&mut walker), Err(ParsingError(
            Location::from_components(0, 0, "test.txt"),
            ErrorKind::ExpectedOneOfKind("whitespace")
        )));

        // And make sure it keeps the original text
        assert_eq!(walker.current_string(), "This  \t\n\n  \r\n Hi");
    }
}