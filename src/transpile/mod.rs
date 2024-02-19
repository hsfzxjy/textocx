mod comm;
mod error;
mod parser;
mod postproc;
mod util;
mod worker;
mod wrap_html;

pub use error::Error;
use std::{result::Result as stdResult, thread::JoinHandle};

use self::{postproc::Piece, wrap_html::wrap_html};

pub struct Job {
    pub italic_math: bool,
    pub tex_code: String,
}

pub type Result = stdResult<String, Error>;

pub struct Solver {
    jhs: Vec<JoinHandle<stdResult<(), rq::Error>>>,
    reqch: ac::Sender<Option<comm::Request>>,
    respch: ac::Receiver<comm::Response>,
}

impl Drop for Solver {
    fn drop(&mut self) {
        for _ in 0..self.jhs.len() {
            self.reqch
                .send_blocking(None)
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

    fn solve_math_part<R: Default>(
        &self,
        part: parser::Part,
        loc: usize,
        counter: &mut usize,
    ) -> R {
        self.reqch
            .send_blocking(Some(comm::Request {
                loc,
                tex: String::from(part.as_str()),
                display_mode: part.typ != parser::Type::InlineMath,
            }))
            .unwrap();
        *counter += 1;
        Default::default()
    }

    pub fn solve(&self, job: Job) -> Result {
        let mut n_maths = 0;
        let mut pieces = parser::parse(&job.tex_code)
            .map_err(Error::bad_input)?
            .into_iter()
            .enumerate()
            .map(|(i, p)| match p.typ {
                parser::Type::Text => Piece::from_text(p.as_str()),
                _ => self.solve_math_part(p, i, &mut n_maths),
            })
            .collect::<Vec<_>>();
        for _ in 0..n_maths {
            let comm::Response { omml, loc } = self.respch.recv_blocking().unwrap();
            let omml = omml.map_err(Error::JS)?;
            pieces[loc] = postproc::Piece::from_math(omml.into());
        }

        Ok(wrap_html(
            pieces
                .into_iter()
                .map(postproc::italic_math(job.italic_math))
                .map(postproc::escape_html)
                .flatten(),
        ))
    }
}
