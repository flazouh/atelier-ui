use gpui_kit::{Hsla, IntoElement, ParentElement, PathBuilder, Styled, canvas, div, point};

use super::super::{
    consts::{SPARK_HEIGHT, SPARK_STROKE, SPARK_WIDTH},
    helpers::spark_points,
};
use crate::scale::px;

/// A thin line through a session's days, in the colour of its series.
pub(super) fn sparkline(values: &[f32], color: Hsla) -> impl IntoElement {
    let points = spark_points(values);
    div().flex_none().w(px(SPARK_WIDTH)).h(px(SPARK_HEIGHT)).child(
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let mut path = PathBuilder::stroke(px(SPARK_STROKE));
                for (at, (x, y)) in points.iter().enumerate() {
                    let p = point(bounds.origin.x + bounds.size.width * *x, bounds.origin.y + bounds.size.height * *y);
                    if at == 0 {
                        path.move_to(p);
                    } else {
                        path.line_to(p);
                    }
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, color);
                }
            },
        )
        .size_full(),
    )
}
