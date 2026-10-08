use std::time::Instant;

use gpui_kit::{Bounds, Pixels};

use super::helpers::list_height;
use crate::{
    motion::{Channel, Curve, Spring},
    task_edit::{Field, Picker},
};

/// A picker that opens from the chip of its field: the chip grows into the picker as one surface, as a select does
/// (`crate::select`), and the header row turns into the filter. The owner keeps one, measures each chip into it, calls
/// [`PickerMorph::sync`] every render and draws [`morph_popover`](crate::task_picker::morph_popover).
#[derive(Clone, Debug)]
pub struct PickerMorph {
    /// 0 the chip, 1 the picker: the shared layout's spring.
    pub(super) morph: Channel,
    /// The list's height, which follows the rows the filter leaves.
    pub(super) list: Channel,
    pub(super) field: Option<Field>,
    /// The last open picker, kept while the surface goes back to its chip.
    pub(super) last: Option<Picker>,
    pub(super) open: bool,
    pub(super) opened: Option<Instant>,
    /// Reduce Motion was on at the last sync: the rows are there at once.
    pub(super) reduce: bool,
    pub(super) anchors: [Option<Bounds<Pixels>>; 4],
}

impl Default for PickerMorph {
    fn default() -> Self {
        Self {
            morph: Channel::new(0.),
            list: Channel::new(0.),
            field: None,
            last: None,
            open: false,
            opened: None,
            reduce: false,
            anchors: [None; 4],
        }
    }
}

impl PickerMorph {
    /// Where the chip of `field` is, from its last layout.
    pub fn set_anchor(&mut self, field: Field, bounds: Bounds<Pixels>) {
        self.anchors[field.slot()] = Some(bounds);
    }
    /// The picker of `field` can grow out of a chip.
    pub fn can_morph(&self, field: Field) -> bool {
        self.anchors[field.slot()].is_some()
    }
    /// Follows the owner's picker: call every render with the picker that is open, or none.
    pub fn sync(&mut self, open: Option<&Picker>, reduce: bool) {
        self.reduce = reduce;
        match open {
            Some(picker) => {
                let field = picker.field();
                if !self.open || self.field != Some(field) {
                    self.open = true;
                    self.field = Some(field);
                    self.opened = Some(Instant::now());
                    self.morph = Channel::new(0.);
                    self.morph
                        .animate(1., Curve::Spring(Spring::select_morph()), 0., reduce);
                    self.list = Channel::new(list_height(picker));
                } else {
                    let want = list_height(picker);
                    if self.list.target() != want {
                        self.list
                            .animate(want, Curve::Spring(Spring::critical(30.)), 0., reduce);
                    }
                }
                self.last = Some(picker.clone());
            }
            None => {
                if self.open {
                    self.open = false;
                    self.morph
                        .animate(0., Curve::Spring(Spring::select_morph()), 0., reduce);
                } else if !self.morph.is_running() && self.morph.value().abs() < 0.002 {
                    self.field = None;
                    self.last = None;
                }
            }
        }
    }
    /// The field whose picker the surface shows.
    pub fn field(&self) -> Option<Field> {
        self.field
    }
    /// How far the chip has grown into the picker, 0 to 1 (a little over while the spring overshoots).
    pub fn level(&self) -> f32 {
        self.morph.value()
    }
    /// The surface is drawn: the picker is open, or on its way back to the chip.
    pub fn shown(&self) -> bool {
        self.field.is_some()
            && self.last.is_some()
            && (self.open || self.morph.is_running() || self.morph.value().abs() > 0.002)
    }
    /// The surface stands in for the chip of `field`, which is drawn clear.
    pub fn hides(&self, field: Field) -> bool {
        self.shown() && self.field == Some(field)
    }
    /// The rows the surface shows.
    pub fn rows(&self) -> Option<&Picker> {
        self.last.as_ref()
    }
    pub fn is_moving(&self) -> bool {
        self.morph.is_running()
            || self.list.is_running()
            || self.opened.is_some_and(|at| {
                self.shown() && at.elapsed().as_secs_f32() < crate::select::opens_in(1) + 0.6
            })
    }
}

/// What the chip looks like at rest, for the surface to begin from: its fill, its corner and the inset of its content.
#[derive(Clone, Copy, Debug)]
pub struct Chip {
    pub fill: gpui_kit::Hsla,
    pub radius: f32,
    pub inset: f32,
}
