use std::borrow::Cow;

#[derive(Debug)]
pub enum Error {
    BadInput(Cow<'static, str>),
    JS(String),
}

impl Error {
    pub(super) fn bad_input<S: std::error::Error>(msg: S) -> Error {
        Error::BadInput(msg.to_string().into())
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadInput(x) => {
                write!(f, "Bad Input: ")?;
                f.write_str(x.as_ref())
            }
            Self::JS(x) => {
                write!(f, "JS Exception: ")?;
                f.write_str(x)
            }
        }
    }
}
