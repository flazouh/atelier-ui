//! Click-to-copy feedback shared by every Copy button: the icon flips to a check for
//! [`duration::COPY_FEEDBACK`] (1.6s, beui's timing), then reverts. Each click replaces the pending
//! reset [`Task`] instead of detaching it, so dropping the old one cancels its timer and a second click
//! can never race the first.

use gpui_kit::{App, ClipboardItem, Entity, Task};

use crate::motion::duration;

/// One copy button's "copied" state. Embed a field of this type in a component's own motion state and
/// drive it from that button's `on_click` with [`Self::click`].
#[derive(Default)]
pub(crate) struct CopyFeedback {
    copied: bool,
    /// The pending revert. Replacing it (rather than detaching) cancels an earlier click's timer.
    reset: Option<Task<()>>,
}

impl CopyFeedback {
    pub fn copied(&self) -> bool {
        self.copied
    }

    /// Copies `text` to the clipboard, marks this feedback "copied", and (re)starts the reset timer.
    /// `field` reaches this `CopyFeedback` inside `entity`'s state, so one implementation serves every
    /// component that has one, whatever else that state holds.
    pub fn click<T: 'static>(
        entity: &Entity<T>,
        field: fn(&mut T) -> &mut CopyFeedback,
        text: String,
        cx: &mut App,
    ) {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        entity.update(cx, |state, cx| {
            field(state).copied = true;
            cx.notify();
        });
        let weak = entity.downgrade();
        let reset = cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(duration::COPY_FEEDBACK)
                .await;
            weak.update(cx, |state, cx| {
                field(state).copied = false;
                cx.notify();
            })
            .ok();
        });
        entity.update(cx, |state, _| {
            field(state).reset = Some(reset);
        });
    }
}
