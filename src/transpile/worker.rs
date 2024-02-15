use std::{cell::Cell, rc::Rc, thread::JoinHandle};

use rq::function::IntoJsFunc;

use super::comm;
use super::util::*;

static BUNDLE: rq::loader::Bundle = rq::embed! {
    "temml": "./Temml/dist/temml.mjs",
    "m2o": "mathml2omml/dist/index.js"
};

const JS_CODE: &str = r#"
import temml from "temml";
import {mml2omml} from "m2o";
Object.hasOwn = Object.hasOwnProperty;
while (true) {
    const input = __wait();
    if (input === undefined) break;
    let res;
    try {
        const mml = temml.renderToString(input, {throwOnError: true});
        res = {omml: mml2omml(mml).replace(` xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"`, "")};
    } catch(e) {
        res = {error: e.toString()};
    }
    __respond(res);
}
"#;

pub(super) fn run(
    reqch: ac::Receiver<comm::Request>,
    respch: ac::Sender<comm::Response>,
) -> JoinHandle<Result<(), rq::Error>> {
    std::thread::spawn(move || {
        let rt = rq::Runtime::new()?;
        rt.set_loader(BUNDLE, BUNDLE);
        rq::Context::full(&rt)?.with(|ctx| {
            let state = Rc::new(State {
                respch,
                reqch,
                ploc: Default::default(),
            });
            ctx.globals()
                .set_func("__wait", state.clone().wait())?
                .set_func("__respond", state.respond())?;
            ctx.clone()
                .compile("worker", JS_CODE)
                .map(|_| ())
                .inspect_err(|_| panic!("{:?}", ctx.catch()))
        })
    })
}

struct State {
    reqch: ac::Receiver<comm::Request>,
    respch: ac::Sender<comm::Response>,
    ploc: Cell<Option<usize>>,
}

impl State {
    fn wait<'js>(self: Rc<Self>) -> impl IntoJsFunc<'js, (rq::Ctx<'js>,)> {
        move |ctx| {
            use comm::Request::*;
            self.reqch
                .recv_blocking()
                .map_err(|_| "recv channel closed")
                .map(|req| match req {
                    Shutdown => None,
                    Process { loc, tex } => {
                        self.ploc.replace(Some(loc));
                        Some(tex)
                    }
                })
                .throw(&ctx)
        }
    }
    fn respond<'js>(self: Rc<Self>) -> impl IntoJsFunc<'js, (rq::Object<'js>,)> {
        move |s: rq::Object| {
            let ctx = s.ctx();
            let res = match s.get("omml") {
                Ok(s) => Ok(s),
                _ => match s.get("error") {
                    Ok(e) => Err(e),
                    Err(x) => panic!("{:?}", x),
                },
            };
            self.ploc
                .take()
                .ok_or("no loc")
                .and_then(|loc| {
                    self.respch
                        .send_blocking(comm::Response { loc, omml: res })
                        .map_err(|_| "send channel closed")
                })
                .throw(ctx)
        }
    }
}
