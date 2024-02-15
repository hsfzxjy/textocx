use std::{sync::mpsc, thread::JoinHandle};

use super::app_delegate::*;
use crate::transpile;

#[derive(Debug)]
pub(super) struct Model {
    jh: Option<JoinHandle<()>>,
    job_sender: mpsc::Sender<Option<transpile::Job>>,
}

impl Drop for Model {
    fn drop(&mut self) {
        self.job_sender
            .send(None)
            .expect("fail to stop model thread");
        if let Some(jh) = self.jh.take() {
            jh.join().unwrap()
        };
    }
}

impl Model {
    pub fn new<D: Delegate>(delegate: D) -> Self {
        let solver = transpile::Solver::new(2);
        let (jobs, jobr) = mpsc::channel::<Option<transpile::Job>>();
        let jh = std::thread::spawn(move || {
            while let Some(job) = jobr.recv().unwrap() {
                delegate.set_solving_status();
                let result = solver.solve(job);
                delegate.update_omml(result);
            }
        });
        Self {
            jh: Some(jh),
            job_sender: jobs,
        }
    }
    pub fn queue_job(&self, job: transpile::Job) {
        self.job_sender.send(Some(job)).unwrap();
    }
}
