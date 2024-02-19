#[derive(Debug)]
pub(super) struct Request {
    pub loc: usize,
    pub tex: String,
    pub display_mode: bool,
}

impl<'js> rq::IntoJs<'js> for Request {
    fn into_js(self, ctx: &rq::Ctx<'js>) -> rq::Result<rq::Value<'js>> {
        Ok({
            let obj = rq::Object::new(ctx.clone())?;
            obj.set("input", self.tex.into_js(ctx)?)?;
            obj.set("displayMode", self.display_mode.into_js(ctx)?)?;
            obj.into()
        })
    }
}

#[derive(Debug)]
pub(super) struct Response {
    pub loc: usize,
    pub omml: Result<String, String>,
}
