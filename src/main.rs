use compiler_utils::*;

fn digit<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    one_of("0123456789")(walker)
}

fn alpha<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    take_if(|c| c.is_alphabetic(), "alphabetic")(walker)
}

fn alpha_numeric<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    take_if(|c| c.is_alphanumeric(), "alphanumeric")(walker)
}

fn ident_symbol<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    alt(alpha_numeric, one_of("_"))(walker)
}

fn ws_text<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<(), ParsingError<'filedata>> {
    map(opt(accepts_while(one_of("\r\n\t "))), |_| ())(walker)
}

pub fn ws<'filedata, Output>(
    combinator: impl Fn(&mut FileWalker<'filedata>) -> Result<Output, ParsingError<'filedata>>,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<Output, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        ws_text(walker)?;
        let result = combinator(walker);
        ws_text(walker)?;

        result
    }
}

pub fn ws_del<'filedata, Output>(
    combinator: impl Fn(&mut FileWalker<'filedata>) -> Result<Output, ParsingError<'filedata>>,
) -> impl Fn(&mut FileWalker<'filedata>) -> Result<Output, ParsingError<'filedata>> {
    move |walker: &mut FileWalker<'filedata>| {
        ws_text(walker)?;
        let result = combinator(walker)?;
        one_of("\r\n\t ")(walker)?;
        ws_text(walker)?;

        Ok(result)
    }
}


fn number<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    ws(accepts_while(digit))(walker)
}

fn identifier<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Span<'filedata>, ParsingError<'filedata>> {
    ws(accepts(
        pair(alpha_numeric, accepts_while(ident_symbol))
    ))(walker)
}

#[derive(Debug)]
enum Value<'filedata> {
    Identifier(Span<'filedata>),
    Number(Span<'filedata>)
}

#[derive(Debug)]
enum Instruction<'filedata> {
    Return(Value<'filedata>)
}

fn funcdecl<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<(Span<'filedata>, Vec<Span<'filedata>>), ParsingError<'filedata>> {
    ws_del(tag("fn"))(walker)?;
    let name = ws(identifier)(walker)?;

    let open = ws(tag("("))(walker)?;
    let close = ws(tag(")"))(walker)?;
    
    let open_2 = ws(tag("{"))(walker)?;

    while ws(instruction)(walker)?.is_some() {}
    
    let close_2 = ws(tag("}"))(walker)?;

    Ok((name, vec![open, close, open_2, close_2]))
}

fn instruction<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Option<Instruction<'filedata>>, ParsingError<'filedata>> {
    if let Ok(Some(_)) = opt(tag("return"))(walker) {
        let inst = Ok(Some(Instruction::Return(value(walker)?)));
        
        tag(";")(walker)?;

        inst
    }
    else {
        Ok(None)
    }
}

fn value<'filedata>(walker: &mut FileWalker<'filedata>) -> Result<Value<'filedata>, ParsingError<'filedata>> {
    if let Ok(value) = number(walker) {
        Ok(Value::Number(value))
    }
    else {
        Ok(Value::Identifier(identifier(walker)?))
    }
}


fn main() {
    let mut data = FileWalker::from_data(include_str!("test_data.txt"), "test_data.txt");

    let (name, tags) = funcdecl(&mut data).unwrap();

    let settings = ErrorDisplaySettings{ colored: true };

    let error_render = ErrorRender::new(ErrorLevel::Warning, &settings, "Pointing out the name", &name.location, vec![
        Note::new(&name, "This is the name", ErrorLevel::Warning),
        Note::new(&tags[0], "This is the open tag", ErrorLevel::Error),
        Note::new(&tags[2], "Open", ErrorLevel::Info),
        Note::new(&tags[3], "Close", ErrorLevel::Info),
    ], &data);

    println!("{}", error_render);
}
