use compiler_utils::*;

#[inline]
fn parens<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<(), ParsingError<'filedata>> {
    alt(
        map(triple(tag("("), opt(accepts_while(parens)), tag(")")), |_| ()),
        map(triple(tag("["),opt(accepts_while(parens)), tag("]")), |_| ())
    )(walker)
}

#[test]
fn test_simple() {
    let mut walker = FileWalker::from_data("()", "input");

    parens(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());

    let mut walker = FileWalker::from_data("[]", "input");

    parens(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());
}

#[test]
fn test_simple_failure() {
    let mut walker = FileWalker::from_data("(", "input");
    assert!(parens(&mut walker).is_err());
    
    let mut walker = FileWalker::from_data(")", "input");
    assert!(parens(&mut walker).is_err());

    let mut walker = FileWalker::from_data("[", "input");
    assert!(parens(&mut walker).is_err());

    let mut walker = FileWalker::from_data("]", "input");
    assert!(parens(&mut walker).is_err());

    let mut walker = FileWalker::from_data("[)", "input");
    assert!(parens(&mut walker).is_err());

    let mut walker = FileWalker::from_data("(]", "input");
    assert!(parens(&mut walker).is_err());
}

#[test]
fn test_nested() {
    let mut walker = FileWalker::from_data("(())", "input");

    parens(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());
    
    let mut walker = FileWalker::from_data("([])", "input");

    parens(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());
    
    let mut walker = FileWalker::from_data("[()]", "input");

    parens(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());
    
    let mut walker = FileWalker::from_data("[[]]", "input");

    parens(&mut walker).unwrap();
    assert!(walker.current_string().is_empty());
}

#[test]
fn test_nested_failure() {
    let mut walker = FileWalker::from_data("([)]", "input");
    assert!(parens(&mut walker).is_err());
    
    let mut walker = FileWalker::from_data("([)", "input");
    assert!(parens(&mut walker).is_err());
    
    let mut walker = FileWalker::from_data("[(]", "input");
    assert!(parens(&mut walker).is_err());
    
    let mut walker = FileWalker::from_data("[[]", "input");
    assert!(parens(&mut walker).is_err());
}