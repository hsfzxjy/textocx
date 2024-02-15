use super::{app::AppUi, App};

use nwg::{
    stretch::geometry::{Rect, Size},
    stretch::style::{Dimension as D, FlexDirection},
};

pub(super) const SIZE: (i32, i32) = (800, 600);

impl App {
    pub(super) fn on_window_minmax(&self, data: &nwg::EventData) {
        let data = data.on_min_max();
        data.set_min_size(SIZE.0, SIZE.1);
    }
    pub(super) fn layout_self(app: &AppUi) -> Result<(), nwg::NwgError> {
        nwg::FlexboxLayout::builder()
            .parent(&app.window)
            .flex_direction(FlexDirection::Column)
            .padding(Rect {
                start: D::Points(5.),
                end: D::Points(5.),
                top: D::Points(5.),
                bottom: D::Points(25.),
            })
            .child(&app.italic_check_box)
            .child_size(Size {
                width: D::Percent(1.),
                height: D::Points(25.),
            })
            .child(&app.auto_copy_check_box)
            .child_size(Size {
                width: D::Percent(1.),
                height: D::Points(25.),
            })
            .child(&app.copy_button)
            .child_size(Size {
                width: D::Percent(1.0),
                height: D::Points(35.),
            })
            .child_margin(Rect {
                start: D::Points(0.),
                end: D::Points(0.),
                top: D::Points(5.),
                bottom: D::Points(5.),
            })
            .build_partial(&app.layout_toolbox)?;

        nwg::FlexboxLayout::builder()
            .parent(&app.window)
            .padding(Rect {
                start: D::Points(0.),
                end: D::Points(5.),
                top: D::Points(5.),
                bottom: D::Points(25.),
            })
            .flex_direction(FlexDirection::Row)
            .child_layout(&app.layout_toolbox)
            .child_size(Size {
                width: D::Points(200.),
                height: D::Percent(1.),
            })
            .child(&app.tex_edit)
            .child_size(Size {
                width: D::Percent(1.),
                height: D::Percent(1.),
            })
            .build(&app.layout)
    }
}
