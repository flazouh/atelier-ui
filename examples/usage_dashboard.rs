//! The usage dashboard on its own, with the app's state held here: press a tile, a range or a session.
//! `USAGE_THEME=light|dark`, `USAGE_EXPANDED=1` opens the first session, `USAGE_EMPTY=1` shows the empty state.
use atelier_ui::{
    Selection, Series, SourceKind, UsageDashboard, UsageDay, UsageModel, UsageRange, UsageSession, UsageSource,
    theme::{ActiveTheme, Appearance, set_appearance},
};
use gpui_kit::{
    AppContext, Bounds, Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, WindowBounds,
    WindowOptions, div, px, size,
};

struct Page {
    range: UsageRange,
    selection: Selection,
    expanded: Option<SharedString>,
    empty: bool,
}

fn source(
    id: &str,
    name: &str,
    caption: &str,
    group: &str,
    series: Series,
    limit: Option<f32>,
    value: &str,
    note: &str,
) -> UsageSource {
    UsageSource {
        id: id.to_string().into(),
        name: name.to_string().into(),
        caption: caption.to_string().into(),
        group: group.to_string().into(),
        series,
        limit,
        value: value.to_string().into(),
        note: note.to_string().into(),
        kind: if limit.is_some() { SourceKind::Subscription } else { SourceKind::Key },
    }
}

fn days(count: usize) -> Vec<UsageDay> {
    let mut seed = 11u64;
    let mut next = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as f32 / (1u64 << 31) as f32
    };
    (0..count)
        .map(|d| {
            let base = 0.3 + 0.7 * next();
            let base = if d == count - 1 { 1.0 } else { base };
            let mut part = |lo: f32, hi: f32| ((lo + (hi - lo) * next()) * base * 8_000_000.) as u64;
            UsageDay {
                label: format!("{}", (d + 26) % 30 + 1).into(),
                detail: "Thu 9 · 5.84 M tokens · $18.40".into(),
                parts: vec![
                    (Series::new(0, 0), part(0.30, 0.40)),
                    (Series::new(0, 1), part(0.12, 0.22)),
                    (Series::new(0, 2), part(0.04, 0.08)),
                    (Series::new(1, 0), part(0.15, 0.28)),
                    (Series::new(2, 0), part(0.05, 0.12)),
                ],
            }
        })
        .collect()
}

fn session(id: &str, title: &str, meta: &str, series: Series, tokens: &str, cost: &str, seed: f32) -> UsageSession {
    UsageSession {
        id: id.to_string().into(),
        title: title.to_string().into(),
        meta: meta.to_string().into(),
        series,
        tokens: tokens.to_string().into(),
        cost: cost.to_string().into(),
        days: (0..14).map(|i| 0.35 + 0.65 * ((i as f32 * 1.7 + seed).sin() * 0.5 + 0.5)).collect(),
        split: vec![
            ("Cache read".into(), "1.52 M".into()),
            ("Input".into(), "0.21 M".into()),
            ("Output".into(), "0.11 M".into()),
        ],
        footnote: "46 turns · 3 days".into(),
    }
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity().downgrade();
        let (a, b, c) = (this.clone(), this.clone(), this);
        let theme = cx.theme().clone();
        let mut dashboard = UsageDashboard::new("usage")
            .range(self.range)
            .selection(self.selection.clone())
            .summary("5.84 M", "today")
            .sources(vec![
                source(
                    "max-p",
                    "Max",
                    "personal",
                    "Claude Code · 3 accounts",
                    Series::new(0, 0),
                    Some(0.45),
                    "45%",
                    "7d",
                ),
                source("max-w", "Max", "work", "Claude Code · 3 accounts", Series::new(0, 1), Some(0.82), "82%", "7d"),
                source("team", "Team", "seat", "Claude Code · 3 accounts", Series::new(0, 2), Some(0.18), "18%", "7d"),
                source("codex", "Codex", "subscription", "Codex", Series::new(1, 0), Some(0.31), "31%", "30d"),
                source("or-w", "Work", "key", "OpenRouter · 2 keys", Series::new(2, 0), Some(0.21), "$4.20", "of $20"),
                source("or-p", "Personal", "key", "OpenRouter · 2 keys", Series::new(2, 0), None, "$1.10", "month"),
                source("api", "API", "key", "Anthropic API", Series::new(3, 0), Some(0.76), "$38", "month"),
            ])
            .days(days(match self.range {
                UsageRange::Week => 7,
                UsageRange::Fortnight => 14,
                UsageRange::Month => 30,
            }))
            .models(vec![
                UsageModel {
                    name: "Opus 5.5".into(),
                    tokens: 41_200_000,
                    series: Series::new(0, 0),
                    label: "41.2 M".into(),
                },
                UsageModel {
                    name: "Sonnet 5.5".into(),
                    tokens: 9_800_000,
                    series: Series::new(0, 0),
                    label: "9.8 M".into(),
                },
                UsageModel {
                    name: "GPT-5".into(),
                    tokens: 7_100_000,
                    series: Series::new(1, 0),
                    label: "7.1 M".into(),
                },
                UsageModel {
                    name: "Kimi K2".into(),
                    tokens: 2_000_000,
                    series: Series::new(2, 0),
                    label: "2.0 M".into(),
                },
            ])
            .total("60.1 M · $186")
            .sessions(vec![
                session(
                    "s1",
                    "Fix the login bug",
                    "atelier · Claude Code · Max work · Opus 5.5",
                    Series::new(0, 1),
                    "1,840,000",
                    "$14.20",
                    0.,
                ),
                session(
                    "s2",
                    "Landing page copy",
                    "ori · Claude Code · Team seat · Sonnet 5.5",
                    Series::new(0, 2),
                    "960,000",
                    "$3.10",
                    2.,
                ),
                session(
                    "s3",
                    "Refactor the review crate",
                    "atelier · Codex · GPT-5",
                    Series::new(1, 0),
                    "720,000",
                    "$5.40",
                    4.,
                ),
            ])
            .expanded(self.expanded.clone())
            .on_range(move |range, _, cx| {
                a.update(cx, |page, cx| {
                    page.range = range;
                    cx.notify();
                })
                .ok();
            })
            .on_select(move |selection, _, cx| {
                b.update(cx, |page, cx| {
                    page.selection = selection;
                    cx.notify();
                })
                .ok();
            })
            .on_expand(move |id, _, cx| {
                c.update(cx, |page, cx| {
                    page.expanded = id;
                    cx.notify();
                })
                .ok();
            });
        if self.empty {
            dashboard = dashboard.empty(Some("No use in this range.".into()));
        }
        div().size_full().bg(theme.background).child(dashboard)
    }
}

fn main() {
    gpui_kit::application().with_assets(atelier_ui::icon::Assets).run(|cx| {
        atelier_ui::init(cx);
        match std::env::var("USAGE_THEME").as_deref() {
            Ok("light") => set_appearance(Appearance::Light, cx),
            Ok("dark") => set_appearance(Appearance::Dark, cx),
            _ => {}
        }
        let bounds = Bounds::centered(None, size(px(1200.), px(1000.)), cx);
        cx.open_window(
            WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
            |_, cx| {
                cx.new(|_| Page {
                    range: UsageRange::Fortnight,
                    selection: Selection::All,
                    expanded: std::env::var("USAGE_EXPANDED").ok().map(|_| "s1".into()),
                    empty: std::env::var("USAGE_EMPTY").is_ok(),
                })
            },
        )
        .expect("open the window");
        cx.activate(true);
    });
}
