use std::sync::mpsc;

use nwg::NoticeSender;

use crate::transpile;

pub(super) trait Delegate: Send + 'static {
    fn set_solving_status(&self);
    fn update_omml(&self, omml: transpile::Result);
}

pub(super) enum Msg {
    SetSolvingStatus,
    UpdateOmml(transpile::Result),
}

pub(super) struct DelegateImpl {
    pub(super) notice: NoticeSender,
    pub(super) msg: mpsc::Sender<Msg>,
}

impl Delegate for DelegateImpl {
    fn set_solving_status(&self) {
        self.msg
            .send(Msg::SetSolvingStatus)
            .unwrap();
        self.notice.notice();
    }

    fn update_omml(&self, omml: transpile::Result) {
        self.msg
            .send(Msg::UpdateOmml(omml))
            .unwrap();
        self.notice.notice();
    }
}
