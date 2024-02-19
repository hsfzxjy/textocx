use std::{
    cell::RefCell,
    sync::mpsc,
    time::{self, Duration},
};

use super::app_delegate::*;
use super::app_layout;
use super::model::Model;
use crate::transpile;
pub(super) use app_ui::AppUi;
use nwg::{CheckBoxState, NativeUi};

#[derive(Debug)]
pub(super) struct State {
    pub(super) msgr: mpsc::Receiver<Msg>,
    pub(super) model: Model,
    prev_queue_at: time::Instant,
    omml: Option<String>,
}

#[derive(nwd::NwgUi, Default)]
pub struct App {
    #[nwg_control(size:app_layout::SIZE, center:true, title: "Textocx: Convert TEX snippet to pastable MSOffice format")]
    #[nwg_events(OnMinMaxInfo: [App::on_window_minmax(SELF, EVT_DATA)], OnWindowClose: [App::on_close(SELF)])]
    pub(super) window: nwg::Window,

    #[nwg_control(parent: window, focus: true, text: "Italic Math", check_state: CheckBoxState::Checked)]
    #[nwg_events(OnButtonClick: [App::try_queue_job(SELF)])]
    pub(super) italic_check_box: nwg::CheckBox,

    #[nwg_control(parent: window, focus: true, text: "Preserve Spaces", check_state: CheckBoxState::Checked)]
    #[nwg_events(OnButtonClick: [App::try_queue_job(SELF)])]
    pub(super) preserve_spaces_check_box: nwg::CheckBox,

    #[nwg_control(parent: window, focus: true, text: "Auto Copy", check_state: CheckBoxState::Checked)]
    pub(super) auto_copy_check_box: nwg::CheckBox,

    #[nwg_control(parent: window, focus: true, text: "Copy", enabled: false)]
    #[nwg_events(OnButtonClick: [App::do_paste(SELF)])]
    pub(super) copy_button: nwg::Button,

    #[nwg_control(parent: window, focus: true, flags: "VSCROLL | VISIBLE | TAB_STOP")]
    #[nwg_events(OnTextInput: [App::try_queue_job(SELF)])]
    pub(super) tex_edit: nwg::TextBox,

    pub(super) layout: nwg::FlexboxLayout,
    pub(super) layout_toolbox: nwg::FlexboxLayout,

    #[nwg_control(parent: window)]
    pub(super) status_bar: nwg::StatusBar,

    #[nwg_control]
    #[nwg_events(OnNotice: [App::on_notice(SELF)])]
    pub(super) notice: nwg::Notice,

    #[nwg_control(max_tick: Some(1), interval: DEBOUNCE_DURATION)]
    #[nwg_events(OnTimerTick: [App::try_queue_job(SELF)])]
    pub(super) timer: nwg::AnimationTimer,

    pub(super) state: RefCell<Option<State>>,
}
const DEBOUNCE_DURATION: Duration = Duration::from_millis(600);

impl App {
    fn toggle_copy_button(&self, enabled: bool) {
        self.copy_button.set_enabled(enabled);
    }
    fn try_queue_job(&self) {
        let mut rc = self.state.borrow_mut();
        let comm = rc.as_mut().unwrap();
        let should_queue = comm.prev_queue_at.elapsed() >= DEBOUNCE_DURATION;
        if !should_queue {
            self.status_bar.set_text(0, "");
            self.timer.start();
            return;
        }
        self.toggle_copy_button(false);
        comm.model.queue_job(transpile::Job {
            italic_math: self.italic_check_box.check_state() == CheckBoxState::Checked,
            preserve_spaces: self
                .preserve_spaces_check_box
                .check_state()
                == CheckBoxState::Checked,
            tex_code: self.tex_edit.text(),
        });
        comm.prev_queue_at = time::Instant::now();
    }

    fn do_paste(&self) {
        let mut rc = self.state.borrow_mut();
        let comm = rc.as_mut().unwrap();
        let msg = match comm.omml.take() {
            Some(msg) => msg,
            _ => return,
        };
        self.paste(msg.as_bytes());
    }

    fn paste(&self, msg: &[u8]) {
        let res = (move || {
            let _cb = clipboard_win::Clipboard::new_attempts(10)?;
            let fmt = clipboard_win::raw::register_format("HTML Format").unwrap();
            clipboard_win::raw::set(fmt.get(), msg)
        })();
        self.status_bar.set_text(
            0,
            if res.is_ok() {
                "Rendered. Copied to clipboard."
            } else {
                "Rendered. Fail to write clipboard, maybe try again later."
            },
        );
        self.toggle_copy_button(res.is_err())
    }
}

fn is_checked(cb: &nwg::CheckBox) -> bool {
    cb.check_state() == CheckBoxState::Checked
}
impl App {
    fn on_close(&self) {
        nwg::stop_thread_dispatch();
        self.state.take().unwrap();
    }
    fn on_notice(&self) {
        let mut to_paste = None::<String>;
        let mut bor = self.state.borrow_mut();
        let state = bor.as_mut().unwrap();
        while let Ok(msg) = state.msgr.try_recv() {
            match msg {
                Msg::SetSolvingStatus => self
                    .status_bar
                    .set_text(0, "Rendering..."),
                Msg::UpdateOmml(res) => match res {
                    Ok(omml) => {
                        if is_checked(&self.auto_copy_check_box) {
                            to_paste.replace(omml);
                        } else {
                            state.omml.replace(omml);
                            self.status_bar.set_text(0, "Rendered.");
                            self.toggle_copy_button(true)
                        }
                    }
                    Err(e) => self
                        .status_bar
                        .set_text(0, &format!("{}", e)),
                },
            }
        }
        drop(bor);
        if let Some(msg) = to_paste {
            self.paste(msg.as_bytes())
        }
    }
}

impl App {
    fn build() -> Result<AppUi, nwg::NwgError> {
        let app = Self::build_ui(Default::default())?;
        let (msgs, msgr) = std::sync::mpsc::channel();
        let delegate = DelegateImpl {
            notice: app.notice.sender(),
            msg: msgs,
        };
        let model = Model::new(delegate);
        app.state.replace(Some(State {
            msgr,
            model,
            prev_queue_at: time::Instant::now(),
            omml: None,
        }));
        Self::layout_self(&app)?;
        Ok(app)
    }
    pub fn build_and_run() -> Result<AppUi, nwg::NwgError> {
        nwg::init().expect("fail to init Native Windows GUI");
        let mut font = Default::default();
        nwg::Font::builder()
            .family("Segoe UI")
            .build(&mut font)?;
        nwg::Font::set_global_default(Some(font));
        Self::build().inspect(|_| nwg::dispatch_thread_events())
    }
}
