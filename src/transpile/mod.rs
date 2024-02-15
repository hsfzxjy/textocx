mod comm;
mod error;
mod util;
mod worker;
mod wrap_html;

pub use error::Error;
use std::{borrow::Cow, result::Result as stdResult, thread::JoinHandle};

use self::wrap_html::wrap_html;

pub struct Job {
    pub italic_math: bool,
    pub tex_code: String,
}

pub type Result = stdResult<String, Error>;

pub struct Solver {
    jhs: Vec<JoinHandle<stdResult<(), rq::Error>>>,
    reqch: ac::Sender<comm::Request>,
    respch: ac::Receiver<comm::Response>,
}

impl Drop for Solver {
    fn drop(&mut self) {
        for _ in 0..self.jhs.len() {
            self.reqch
                .send_blocking(comm::Request::Shutdown)
                .expect("fail to send shutdown signal");
        }
        for jh in self.jhs.drain(..) {
            jh.join()
                .unwrap()
                .expect("worker thread fails");
        }
    }
}

impl Solver {
    pub fn new(nworkers: usize) -> Solver {
        let (reqch_s, reqch_r) = ac::bounded(4);
        let (respch_s, respch_r) = ac::bounded(4);
        let jhs = (0..nworkers)
            .map(|_| worker::run(reqch_r.clone(), respch_s.clone()))
            .collect();
        Solver {
            reqch: reqch_s,
            respch: respch_r,
            jhs,
        }
    }

    pub fn solve(&self, job: Job) -> Result {
        let tex = &job.tex_code;
        let mut parts: Vec<Cow<str>> = tex
            .split('$')
            .map(Cow::Borrowed)
            .collect();
        if parts.len() % 2 == 0 {
            Error::bad_input("odd number of $")?;
        }

        let n = parts.len() / 2;
        for i in 0..n {
            self.reqch
                .send_blocking(comm::Request::Process {
                    loc: i,
                    tex: String::from(parts[i * 2 + 1].as_ref()),
                })
                .unwrap();
        }
        for _ in 0..n {
            let comm::Response { omml, loc } = self.respch.recv_blocking().unwrap();
            let mut omml = omml.map_err(Error::JS)?;
            if job.italic_math {
                omml = format!("<i>{}</i>", omml);
            }
            parts[2 * loc + 1] = Cow::Owned(omml);
        }

        Ok(wrap_html(parts))
    }
}
