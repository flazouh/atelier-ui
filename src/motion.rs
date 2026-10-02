//! Springs, easing curves, and durations. Presets are beui's (`lib/ease.ts`), stepped the way Motion
//! solves them.

use std::time::Instant;

/// The time every animation reads. It is the real time, except in a test that froze the clock (see `clock`): then
/// it moves only when the test moves it, so a test never depends on how fast the machine runs.
pub fn now() -> Instant {
    #[cfg(test)]
    if let Some(at) = clock::frozen() {
        return at;
    }
    Instant::now()
}

/// When the process started animating: a fixed moment to count a spinner's turns from.
pub fn epoch() -> Instant {
    static AT: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    *AT.get_or_init(Instant::now)
}

/// Milliseconds since the Unix epoch, for an id that does not repeat between runs.
pub fn now_millis() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis())
}

/// A clock a test freezes and moves by hand. It is per thread, and every test runs on its own thread.
#[cfg(test)]
pub(crate) mod clock {
    use std::{cell::Cell, time::{Duration, Instant}};

    thread_local! {
        static AT: Cell<Option<Instant>> = const { Cell::new(None) };
    }

    pub fn frozen() -> Option<Instant> {
        AT.with(|at| at.get())
    }

    /// Stops the clock where it is. Freezing a frozen clock does nothing.
    pub fn freeze() {
        AT.with(|at| {
            if at.get().is_none() {
                at.set(Some(Instant::now()));
            }
        });
    }

    /// Moves a frozen clock on by `by`.
    pub fn advance(by: Duration) {
        AT.with(|at| at.set(Some(at.get().expect("freeze the clock first") + by)));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl Spring {
    /// Press feedback on buttons and other tappable surfaces.
    pub const PRESS: Self = Self { stiffness: 500., damping: 30., mass: 0.6 };
    /// Label or icon slots trading places inside a control.
    pub const SWAP: Self = Self { stiffness: 460., damping: 30., mass: 0.55 };
    /// Overlay panels summoned by the pointer.
    pub const PANEL: Self = Self { stiffness: 420., damping: 40., mass: 0.5 };
    /// A slider's handle and fill following a drag (`SPRING_GLIDE`): stiff and critically damped, so it
    /// follows the pointer and never rebounds off an end.
    pub const GLIDE: Self = Self { stiffness: 700., damping: 50., mass: 0.5 };
    /// A slider's handle stretching when it is grabbed: bouncy on purpose.
    pub const GRAB: Self = Self { stiffness: 500., damping: 14., mass: 0.7 };
    /// Pills and indicators gliding between positions.
    /// The toast stack's spring (`stiffness: 420, damping: 34, mass: 0.75`): a toast coming in, springing back from a
    /// drag, and the toasts round it moving.
    pub const STACK: Self = Self { stiffness: 420., damping: 34., mass: 0.75 };
    /// The bloom menu's box, a folder opening with a touch of overshoot (`stiffness: 300, damping: 32, mass: 0.9`).
    pub const FOLDER: Self = Self { stiffness: 300., damping: 32., mass: 0.9 };
    /// A choice of the bloom menu arriving (`stiffness: 440, damping: 34`).
    pub const BLOOM_ITEM: Self = Self { stiffness: 440., damping: 34., mass: 1. };
    pub const LAYOUT: Self = Self { stiffness: 360., damping: 32., mass: 0.6 };
    /// Tailwind's `transition-colors` (150ms), as a spring that restarts smoothly from anywhere.
    pub const TINT: Self = Self::critical(36.);
    /// mem0's arrow chip: the arrows sliding through it, fitted to its hover.
    pub const CHIP_SLIDE: Self = Self::critical(29.);
    /// mem0's arrow chip: its color change, a little faster than the slide.
    pub const CHIP_TINT: Self = Self::critical(36.);

    /// Motion's `{ type: "spring", duration, bounce }`, in seconds: the same conversion Motion does
    /// (mass 1, `stiffness = (2π / duration)²`, `damping = 2 (1 - bounce) √stiffness`).
    pub fn bouncy(duration: f32, bounce: f32) -> Self {
        let stiffness = (std::f32::consts::TAU / duration).powi(2);
        Self { stiffness, damping: 2. * (1. - bounce) * stiffness.sqrt(), mass: 1. }
    }

    /// A spring that settles as fast as possible without overshoot, at `omega` radians per second.
    pub const fn critical(omega: f32) -> Self {
        Self { stiffness: omega * omega, damping: 2. * omega, mass: 1. }
    }

    /// MorphSelect's shared layout: the trigger grows into the panel and back, `{ type: "spring", duration: 0.5,
    /// bounce: 0.22 }` on the web; here twice as fast (Alex, 2026-10-01), so `duration: 0.25`. The chevron turns on it too.
    pub fn select_morph() -> Self {
        Self::bouncy(SELECT_MORPH_SECONDS, 0.22)
    }
}

/// How long the select's morph takes.
pub const SELECT_MORPH_SECONDS: f32 = 0.25;

/// A value driven toward its target by a spring.
#[derive(Clone, Debug)]
pub struct Animated {
    spring: Spring,
    value: f32,
    velocity: f32,
    target: f32,
}

impl Animated {
    pub fn new(spring: Spring, value: f32) -> Self {
        Self { spring, value, velocity: 0., target: value }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Close enough to rest: within 0.0005 of the target, scaled by its size. A pixel-sized target needs the
    /// scale: near 237 a 1ms step of an f32 can no longer move the value by 0.0005, and the spring would sit
    /// just short for ever, asking for a frame each time.
    pub fn is_settled(&self) -> bool {
        let scale = self.target.abs().max(1.);
        (self.value - self.target).abs() < 0.0005 * scale && self.velocity.abs() < 0.01 * scale
    }

    /// Advances by `dt` seconds in 1ms steps. Returns true while still moving.
    pub fn step(&mut self, dt: f32, reduce_motion: bool) -> bool {
        if reduce_motion {
            self.value = self.target;
            self.velocity = 0.;
            return false;
        }
        let Spring { stiffness, damping, mass } = self.spring;
        // Clamp so a long pause (app in background) never makes a single huge step.
        let mut left = dt.min(0.064);
        while left > 0. {
            let h = left.min(0.001);
            let force = -stiffness * (self.value - self.target) - damping * self.velocity;
            self.velocity += force / mass * h;
            self.value += self.velocity * h;
            left -= h;
        }
        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.;
        }
        !self.is_settled()
    }
}

/// CSS easing curves as cubic-bezier control points.
pub mod ease {
    /// beui's `EASE_OUT`.
    pub const OUT: [f32; 4] = [0.16, 1., 0.3, 1.];
    /// Motion's default tween curve (`easeOut`), used when beui gives only a duration.
    pub const STANDARD_MOTION: [f32; 4] = [0., 0., 0.58, 1.];
    /// beui's `EASE_IN_OUT`, for things already on screen.
    pub const IN_OUT: [f32; 4] = [0.77, 0., 0.175, 1.];
    /// beui's `EASE_DRAWER`.
    pub const DRAWER: [f32; 4] = [0.32, 0.72, 0., 1.];
    /// Fluid Functionalism's press curve, used by Button.
    pub const FLUID: [f32; 4] = [0.23, 1., 0.32, 1.];
    /// The Claude app's text morph (`Mr`): `[0.2, 0, 0, 1]`.
    pub const MORPH: [f32; 4] = [0.2, 0., 0., 1.];
    /// Motion's default tween for opacity and filter when a variant gives none: `[0.25, 0.1, 0.35, 1]`.
    pub const MOTION_DEFAULT: [f32; 4] = [0.25, 0.1, 0.35, 1.];
    /// CSS `ease-in-out`, the Claude app's thinking breath.
    pub const BREATH: [f32; 4] = [0.42, 0., 0.58, 1.];
}

/// How far, in pixels, text rises as it morphs out and in.
pub const MORPH_RISE: f32 = 3.;

/// How far, in pixels, a new chat item rises as it enters. A whole block travels further than one
/// status word, so this is twice [`MORPH_RISE`].
pub const ENTER_RISE: f32 = 6.;

/// Chat items that arrive in the same frame start this far apart (beui's list stagger).
pub const STAGGER_STEP: std::time::Duration = std::time::Duration::from_millis(35);

/// A staggered list settles by this time: its last item starts early enough to finish its entrance.
pub const STAGGER_CAP: std::time::Duration = std::time::Duration::from_millis(300);
/// The Claude app's thinking breath dips the label to this opacity and back.
pub const BREATH_LOW: f32 = 0.75;

/// The Claude Code CLI's status glimmer (2.1.283): three graphemes in `claudeShimmer` walking across
/// the label, then a pad of empty steps on each side before the next pass.
pub mod glimmer {
    /// One grapheme per step while thinking or responding (`wo`).
    pub const STEP_MS: u64 = 200;
    /// One grapheme per step while requesting.
    pub const REQUESTING_STEP_MS: u64 = 50;
    /// Steps the glimmer spends off each end of the text (`Te`).
    pub const PAD: i32 = 10;
    /// The smooth band fades to nothing this many graphemes from its center, so it covers the same
    /// three graphemes the CLI lights.
    pub const BAND_REACH: f32 = 1.5;
}

/// How long one-shot transitions take.
pub mod duration {
    use std::time::Duration;

    /// Hidden content fading in when the user opens it, such as tool output.
    pub const REVEAL: Duration = Duration::from_millis(150);
    /// A button sinking under the pointer, like Fluid Functionalism's collapsing ring.
    pub const PRESS_DOWN: Duration = Duration::from_millis(80);
    /// The button coming back up.
    pub const PRESS_UP: Duration = Duration::from_millis(180);
    /// One full turn of a spinner: lucide's `animate-spin`.
    pub const SPIN: Duration = Duration::from_millis(1000);
    /// TodoList's in-progress arc: beui sets this one spinner to a slower turn than every other.
    pub const TODO_ARC_SPIN: Duration = Duration::from_millis(1100);
    /// The Claude app's text morph: a label swap or a new segment rising in.
    pub const MORPH: Duration = Duration::from_millis(180);
    /// A new chat item fading and rising in, as threadmail's agent turn entrance (ADR 003).
    pub const ENTER: Duration = Duration::from_millis(200);
    /// The same entrance under Reduce Motion: a shorter fade with no rise.
    pub const ENTER_REDUCED: Duration = Duration::from_millis(120);
    /// One breath of the thinking label.
    pub const BREATH: Duration = Duration::from_millis(2000);
    /// How long a new thinking label stays still before it starts to breathe.
    pub const BREATH_DELAY: Duration = Duration::from_millis(3000);
    /// A decided hunk's closing rows fading out, before its edit runs.
    pub const RESOLVE_FADE: Duration = Duration::from_millis(100);
    /// A hunk closing when the user accepts or rejects it: the rows below slide up over this, after
    /// [`RESOLVE_FADE`].
    pub const RESOLVE: Duration = Duration::from_millis(190);
    /// How long a finished subagent's row keeps its check in the strip before it leaves.
    pub const FINISH_HOLD: Duration = Duration::from_millis(1200);
    /// How long a "copied" check mark shows before a copy button reverts to its icon.
    pub const COPY_FEEDBACK: Duration = Duration::from_millis(1600);
    /// Caps a continuous idle repaint (the thinking breath, the smooth glimmer band) at 30fps
    /// instead of asking for every display frame.
    pub const REPAINT_CAP: Duration = Duration::from_millis(33);
}

/// beui's `AgentDisclosure` reveal, as a [`Curve`]: 220ms opening, 140ms closing, both eased out. Every
/// section that opens and closes on its own: TodoList, ToolCall, FileDiff, ToolApproval, and AgentText's
/// source panel, animates through this same curve via `crate::reveal::Reveal`.
pub fn disclosure(open: bool) -> Curve {
    Curve::Ease(if open { 0.22 } else { 0.14 }, ease::OUT)
}

/// Measures the time between animation frames.
#[derive(Clone, Debug, Default)]
pub struct FrameClock {
    last: Option<Instant>,
}

impl FrameClock {
    /// Seconds since the previous frame, or 0 on the first frame after a rest.
    pub fn tick(&mut self) -> f32 {
        let now = now();
        let dt = self.last.map_or(0., |last| (now - last).as_secs_f32());
        self.last = Some(now);
        dt
    }

    /// Call when nothing moves, so the next animation does not see the idle gap.
    pub fn rest(&mut self) {
        self.last = None;
    }
}

/// CSS `cubic-bezier(x1, y1, x2, y2)` at time `t`.
pub fn cubic_bezier([x1, y1, x2, y2]: [f32; 4], t: f32) -> f32 {
    let curve = |a: f32, b: f32, s: f32| 3. * a * s * (1. - s).powi(2) + 3. * b * s * s * (1. - s) + s.powi(3);
    let (mut lo, mut hi) = (0., 1.);
    for _ in 0..30 {
        let mid = (lo + hi) / 2.;
        if curve(x1, x2, mid) < t {
            lo = mid
        } else {
            hi = mid
        }
    }
    curve(y1, y2, (lo + hi) / 2.)
}

#[cfg(test)]
mod tests;

/// How a [`Channel`] moves to a new target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    Spring(Spring),
    /// A cubic-bezier ease over a duration, in seconds.
    Ease(f32, [f32; 4]),
    /// Jump at once.
    Instant,
}

/// One animated number with Motion's per-property timing: a curve and a delay. It is a pure function of
/// time, solved in closed form, so frames can drop without changing the path. Retargeting mid-flight keeps
/// the current value and speed, as Motion does.
#[derive(Clone, Debug)]
pub struct Channel {
    from: f32,
    velocity: f32,
    to: f32,
    curve: Curve,
    delay: f32,
    start: Instant,
}

impl Channel {
    pub fn new(value: f32) -> Self {
        Self { from: value, velocity: 0., to: value, curve: Curve::Instant, delay: 0., start: now() }
    }

    pub fn target(&self) -> f32 {
        self.to
    }

    /// Moves to `to` after `delay` seconds. With `reduce_motion` it jumps.
    pub fn animate(&mut self, to: f32, curve: Curve, delay: f32, reduce_motion: bool) {
        self.animate_at(to, curve, delay, reduce_motion, now());
    }

    pub fn animate_at(&mut self, to: f32, curve: Curve, delay: f32, reduce_motion: bool, now: Instant) {
        let (value, velocity) = self.sample(now);
        *self = Self {
            from: value,
            velocity,
            to,
            curve: if reduce_motion { Curve::Instant } else { curve },
            delay: if reduce_motion { 0. } else { delay },
            start: now,
        };
    }

    pub fn value(&self) -> f32 {
        self.value_at(now())
    }

    pub fn value_at(&self, now: Instant) -> f32 {
        self.sample(now).0
    }

    /// True while the value still changes, so the caller keeps asking for frames.
    pub fn is_running(&self) -> bool {
        self.is_running_at(now())
    }

    pub fn is_running_at(&self, now: Instant) -> bool {
        let t = now.saturating_duration_since(self.start).as_secs_f32() - self.delay;
        match self.curve {
            Curve::Instant => false,
            Curve::Ease(duration, _) => t < duration,
            Curve::Spring(_) => {
                let (value, velocity) = self.sample(now);
                t < 0. || (value - self.to).abs() > 0.01 || velocity.abs() > 0.1
            }
        }
    }

    /// Value and speed per second at `now`.
    fn sample(&self, now: Instant) -> (f32, f32) {
        let t = now.saturating_duration_since(self.start).as_secs_f32() - self.delay;
        if t < 0. {
            return (self.from, 0.);
        }
        match self.curve {
            Curve::Instant => (self.to, 0.),
            Curve::Ease(duration, curve) => {
                if t >= duration {
                    return (self.to, 0.);
                }
                let at = |t: f32| self.from + (self.to - self.from) * cubic_bezier(curve, (t / duration).clamp(0., 1.));
                let h = 0.001;
                (at(t), (at(t + h) - at((t - h).max(0.))) / (h + h.min(t)))
            }
            Curve::Spring(spring) => {
                let (d, v) = spring_offset(spring, self.from - self.to, self.velocity, t);
                (self.to + d, v)
            }
        }
    }
}

/// Offset from the target and speed of a damped spring after `t` seconds, starting at offset `d0` with
/// speed `v0`.
fn spring_offset(Spring { stiffness, damping, mass }: Spring, d0: f32, v0: f32, t: f32) -> (f32, f32) {
    let omega = (stiffness / mass).sqrt();
    let zeta = damping / (2. * (stiffness * mass).sqrt());
    if (zeta - 1.).abs() < 1e-4 {
        let b = v0 + omega * d0;
        let e = (-omega * t).exp();
        (e * (d0 + b * t), e * (b - omega * (d0 + b * t)))
    } else if zeta < 1. {
        let wd = omega * (1. - zeta * zeta).sqrt();
        let (a, b) = (d0, (v0 + zeta * omega * d0) / wd);
        let e = (-zeta * omega * t).exp();
        let (s, c) = (wd * t).sin_cos();
        let d = e * (a * c + b * s);
        (d, -zeta * omega * d + e * (-a * wd * s + b * wd * c))
    } else {
        let root = (zeta * zeta - 1.).sqrt();
        let (r1, r2) = (-omega * (zeta - root), -omega * (zeta + root));
        let c2 = (v0 - r1 * d0) / (r2 - r1);
        let c1 = d0 - c2;
        let (e1, e2) = ((r1 * t).exp(), (r2 * t).exp());
        (c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
    }
}

/// Motion keyframes: `values` reached at `times` (fractions of `duration` seconds), each segment eased by
/// `curve`. Like Motion, they always start from the first value.
pub fn keyframes(values: &[f32], times: &[f32], duration: f32, curve: [f32; 4], elapsed: f32) -> f32 {
    let p = (elapsed / duration).clamp(0., 1.);
    let i = times.windows(2).position(|w| p <= w[1]).unwrap_or(times.len() - 2);
    let span = (times[i + 1] - times[i]).max(f32::EPSILON);
    let local = ((p - times[i]) / span).clamp(0., 1.);
    values[i] + (values[i + 1] - values[i]) * cubic_bezier(curve, local)
}
