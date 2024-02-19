use std::borrow::{Borrow, Cow};

use rq::{function::IntoJsFunc, IntoJs, Object};

pub trait ThrowJs {
    type Ok;
    fn throw(self, ctx: &rq::Ctx) -> Result<Self::Ok, rq::Error>;
}

impl<T, S: AsRef<str>> ThrowJs for Result<T, S> {
    type Ok = T;

    fn throw(self, ctx: &rq::Ctx) -> Result<T, rq::Error> {
        self.or_else(|errmsg| {
            errmsg
                .as_ref()
                .into_js(ctx)
                .and_then(|e| Err(ctx.throw(e)))
        })
    }
}

pub trait SetFunc<'js>: Sized {
    fn set_func<P, S, F>(self, name: S, f: F) -> rq::Result<Self>
    where
        F: IntoJsFunc<'js, P> + 'js,
        S: AsRef<str>;
}

impl<'js> SetFunc<'js> for Object<'js> {
    fn set_func<P, S, F>(self, name: S, f: F) -> rq::Result<Self>
    where
        F: IntoJsFunc<'js, P> + 'js,
        S: AsRef<str>,
    {
        let ctx = self.ctx();
        self.set(name.as_ref(), rq::Function::new(ctx.clone(), f))?;
        Ok(self)
    }
}

pub trait Bind<'a> {
    type B: 'a + ToOwned + ?Sized;
    fn bind<F>(self, f: F) -> Cow<'a, Self::B>
    where
        F: FnOnce(&Self::B) -> Cow<Self::B>;
}

impl<'a, B> Bind<'a> for Cow<'a, B>
where
    B: ToOwned + ?Sized + 'a,
{
    type B = B;

    fn bind<F>(self, f: F) -> Cow<'a, Self::B>
    where
        F: FnOnce(&Self::B) -> Cow<Self::B>,
    {
        match self {
            Cow::Borrowed(s) => f(s),
            Cow::Owned(s) => match f(s.borrow()) {
                Cow::Borrowed(_) => Cow::Owned(s),
                Cow::Owned(s) => Cow::Owned(s),
            },
        }
    }
}
