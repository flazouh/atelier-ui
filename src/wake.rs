//! A single timed wake-up for a component's motion state, for animations that change only now and then
//! (a sprite frame, a glimmer step, a label threshold). Between wake-ups nothing asks for frames.

use std::time::{Duration, Instant};

use gpui_kit::{Context, Task};

#[derive(Default)]
pub(crate) struct Wake {
    deadline: Option<Instant>,
    /// Dropping it cancels the wake-up.
    task: Option<Task<()>>,
}

impl Wake {
    /// Re-renders the state's view at `when`. A pending wake-up for the same moment is kept, so calling
    /// this on every render does not restart the timer.
    pub fn at<S: 'static>(&mut self, when: Instant, cx: &mut Context<S>) {
        let same = self
            .deadline
            .is_some_and(|d| d.max(when) - d.min(when) < Duration::from_millis(1));
        if same && self.task.is_some() {
            return;
        }
        self.deadline = Some(when);
        let wait = when.saturating_duration_since(Instant::now());
        self.task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(wait).await;
            this.update(cx, |_, cx| cx.notify()).ok();
        }));
    }

    pub fn cancel(&mut self) {
        self.deadline = None;
        self.task = None;
    }
}
