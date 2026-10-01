//! The theme picker: every theme by name, grouped by family, opening upward when it sits at the foot
//! of a pane. A pick applies at once to every window; the owner hears it to remember it.

use gpui_kit::{App, ElementId, IntoElement};

use crate::{
    select::{Select, SelectOption},
    theme::{Theme, set_theme},
    themes,
};

pub fn theme_picker(id: impl Into<ElementId>, current: &Theme, on_pick: impl Fn(&Theme, &mut App) + 'static) -> impl IntoElement {
    let all = themes::all();
    let options = all.iter().map(|t| SelectOption::from(t.name.clone()).group(t.family.clone()));
    Select::new(id, options).selected(all.iter().position(|t| t.name == current.name)).on_change(move |i, _, cx| {
        let Some(picked) = themes::all().get(i) else { return };
        set_theme(picked.clone(), cx);
        on_pick(picked, cx);
    })
}
