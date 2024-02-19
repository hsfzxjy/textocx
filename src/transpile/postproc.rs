use std::borrow::Cow;

type StaticString = Cow<'static, str>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
enum Type {
    #[default]
    Text,
    Math,
}

#[derive(Default)]
pub(super) struct Piece<'a> {
    typ: Type,
    content: Cow<'a, str>,
    dec: Decoration,
}

pub(super) struct PieceIter<'a>([Option<Cow<'a, str>>; 3]);

impl<'a> std::iter::Iterator for PieceIter<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        for slot in &mut self.0 {
            if let Some(c) = slot.take() {
                return Some(c);
            }
        }
        None
    }
}

impl<'a> std::iter::IntoIterator for Piece<'a> {
    type Item = Cow<'a, str>;

    type IntoIter = PieceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PieceIter([self.dec.prefix, Some(self.content), self.dec.suffix])
    }
}

impl<'a> Piece<'a> {
    pub fn from_text(t: &'a str) -> Self {
        Self {
            typ: Type::Text,
            content: t.into(),
            ..Default::default()
        }
    }
    pub fn from_math(m: Cow<'a, str>) -> Self {
        Self {
            typ: Type::Math,
            content: m,
            ..Default::default()
        }
    }
}

pub(super) fn italic_math(enabled: bool) -> impl for<'a> FnMut(Piece<'a>) -> Piece<'a> {
    move |mut p| {
        if enabled && p.typ == Type::Math {
            p.dec = Decoration {
                prefix: Some("<i>".into()),
                suffix: Some("</i>".into()),
            };
        }
        p
    }
}

#[derive(Default)]
struct Decoration {
    prefix: Option<StaticString>,
    suffix: Option<StaticString>,
}
