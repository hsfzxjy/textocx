pub(super) enum Request {
    Shutdown,
    Process { loc: usize, tex: String },
}

#[derive(Debug)]
pub(super) struct Response {
    pub loc: usize,
    pub omml: Result<String, String>,
}
