use nom::branch::alt;
use nom::bytes::complete::{self as bc};
use nom::character::complete::{self as cc};
use nom::sequence::delimited;
use nom::{Finish, Parser};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(super) enum Type {
    Text,
    InlineMath,
    BlockMath,
    Environ,
}

impl Type {
    fn build(self, content: &str) -> Part {
        Part { typ: self, content }
    }
    fn builder(self) -> impl Fn(&str) -> Part {
        move |content| self.build(content)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Part<'a> {
    pub typ: Type,
    content: &'a str,
}

impl<'a> Part<'a> {
    pub fn as_str(&self) -> &'a str {
        self.content
    }
}

#[test]
fn test_parse() {
    use Type::*;
    for (input, res) in [
        (r"123你好", Ok(vec![Text.build("123你好")])),
        (
            r"123你好 $\sqrt{1}$",
            Ok(vec![Text.build("123你好 "), InlineMath.build(r"\sqrt{1}")]),
        ),
        (
            r"123你好 $42$ \begin{align }你abc\end{ align}$$abc$$",
            Ok(vec![
                Text.build("123你好 "),
                InlineMath.build("42"),
                Text.build(" "),
                Environ.build(r"\begin{align }你abc\end{ align}"),
                BlockMath.build("abc"),
            ]),
        ),
    ] {
        assert_eq!(parse(input), res);
    }
}

type Error<'a> = nom::error::Error<&'a str>;

pub(super) fn parse(input: &str) -> Result<Vec<Part>, Error> {
    nom::combinator::all_consuming(nom::multi::many0(alt((
        block_math,
        inline_math,
        environ,
        text,
    ))))(input)
    .finish()
    .map(|(_, parts)| parts)
}

fn text(input: &str) -> nom::IResult<&str, Part> {
    use nom::{combinator::*, multi::*};
    recognize(many1_count(alt((
        cc::char::<&str, nom::error::Error<&str>>('\\')
            .and(cc::one_of(r#"\$~{}"#))
            .map(unit),
        not(alt((
            //
            bc::tag("$"),
            bc::tag("\\begin"),
            bc::tag("\\end"),
        )))
        .and(bc::take(1usize))
        .map(unit),
    ))))(input)
    .map_output(Type::Text.builder())
}

#[test]
fn test_inline_math() {
    assert_eq!(inline_math("$1$"), Ok(("", Type::InlineMath.build("1"))));
    assert!(inline_math("$1").is_err());
}

fn inline_math(input: &str) -> nom::IResult<&str, Part> {
    delimited(cc::char('$'), bc::is_not("$"), cc::char('$'))(input)
        .map_output(Type::InlineMath.builder())
}

fn block_math(input: &str) -> nom::IResult<&str, Part> {
    delimited(bc::tag("$$"), bc::is_not("$"), bc::tag("$$"))(input)
        .map_output(Type::BlockMath.builder())
}

fn environ(input: &str) -> nom::IResult<&str, Part> {
    use nom::{combinator::*, multi::*};
    recognize(move |input| {
        let (input, name) = begin_environ(input)?;
        many0_count(not(end_environ(name)).and(cc::anychar))
            .and(end_environ(name))
            .map(unit)
            .parse(input)
    })(input)
    .map_output(Type::Environ.builder())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EnvironName<'a>(&'a str);

#[test]
fn test_begin_environ() {
    for name in "equation equation* 
    alignedat alignat alignat* aligned align align*
    gathered gather gather* CD multline darray
    dcases drcases matrix* pmatrix* 
    bmatrix* Bmatrix* vmatrix* Vmatrix*"
        .split_whitespace()
    {
        assert_eq!(
            begin_environ(&format!("\\begin{{{}}}", name)),
            Ok(("", EnvironName(name)))
        );
    }
}

fn begin_environ(input: &str) -> nom::IResult<&str, EnvironName> {
    use bc::tag;
    use nom::combinator::{opt, recognize};
    let ast = |name: &'static str| recognize(tag(name).and(opt(tag("*"))));

    // https://temml.org/docs/en/supported#environments
    delimited(
        tag("\\begin{").and(cc::space0),
        alt((
            ast("equation"),
            tag("alignedat"),
            ast("alignat"),
            tag("aligned"),
            ast("align"),
            tag("gathered"),
            ast("gather"),
            tag("CD"),
            tag("multline"),
            alt((tag("darray"), tag("dcases"), tag("drcases"))),
            recognize(opt(cc::one_of("pbBvV")).and(tag("matrix*"))),
        )),
        cc::space0.and(tag("}")),
    )(input)
    .map_output(EnvironName)
}

fn end_environ(name: EnvironName) -> impl Parser<&str, (), nom::error::Error<&str>> {
    delimited(
        bc::tag("\\end{").and(cc::space0),
        bc::tag(name.0),
        cc::space0.and(bc::tag("}")),
    )
    .map(unit)
}

trait MapOutput<I, O> {
    type Out<X>;
    fn map_output<F, O2>(self, f: F) -> Self::Out<O2>
    where
        F: FnOnce(O) -> O2;
}

impl<I, O, E> MapOutput<I, O> for nom::IResult<I, O, E> {
    type Out<X> = nom::IResult<I, X, E>;

    fn map_output<F, O2>(self, f: F) -> Self::Out<O2>
    where
        F: FnOnce(O) -> O2,
    {
        self.map(|(rest, x)| (rest, f(x)))
    }
}

fn unit<T>(_: T) {}
