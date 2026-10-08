use super::structs::Step;

/// The progress line's length.
pub(super) const LINE_WIDTH: f32 = 56.;

/// Space between the bar's parts, and on each side of it.
pub(super) const GAP: f32 = 8.;

pub(super) const PADDING: f32 = 12.;

/// Layout rounds each box to whole pixels, so the parts can add up to one or two pixels less than the whole needs.
pub(super) const FIT_SLACK: f32 = 2.;

/// The steps, widest first.
pub const STEPS: [Step; 8] = [
    Step {
        line: true,
        totals: true,
        nav_words: true,
        short_scopes: false,
        short_summary: false,
        scope_cap: true,
        count_only: false,
    },
    Step {
        line: false,
        totals: true,
        nav_words: true,
        short_scopes: false,
        short_summary: false,
        scope_cap: true,
        count_only: false,
    },
    Step {
        line: false,
        totals: false,
        nav_words: true,
        short_scopes: false,
        short_summary: false,
        scope_cap: true,
        count_only: false,
    },
    Step {
        line: false,
        totals: false,
        nav_words: false,
        short_scopes: false,
        short_summary: false,
        scope_cap: true,
        count_only: false,
    },
    Step {
        line: false,
        totals: false,
        nav_words: false,
        short_scopes: false,
        short_summary: true,
        scope_cap: true,
        count_only: false,
    },
    Step {
        line: false,
        totals: false,
        nav_words: false,
        short_scopes: true,
        short_summary: true,
        scope_cap: true,
        count_only: false,
    },
    Step {
        line: false,
        totals: false,
        nav_words: false,
        short_scopes: true,
        short_summary: true,
        scope_cap: false,
        count_only: false,
    },
    Step {
        line: false,
        totals: false,
        nav_words: false,
        short_scopes: true,
        short_summary: true,
        scope_cap: false,
        count_only: true,
    },
];
