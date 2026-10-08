use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use crate::{
    theme::{Appearance, set_appearance},
    usage_dashboard::{
        Selection, Series, SourceKind, UsageDashboard, UsageDay, UsageModel, UsageRange, UsageSession, UsageSource,
    },
};

/// What the callbacks heard, in order.
pub type Log = Rc<RefCell<Vec<String>>>;

pub struct Options {
    pub empty: Option<&'static str>,
    pub expanded: Option<&'static str>,
}

impl Default for Options {
    fn default() -> Self {
        Self { empty: None, expanded: None }
    }
}

pub struct Host {
    pub options: Options,
    pub log: Log,
}

pub fn dashboard(options: Options) -> Host {
    Host { options, log: Log::default() }
}

fn source(id: &str, name: &str, group: &str, series: Series, limit: Option<f32>) -> UsageSource {
    UsageSource {
        id: id.to_string().into(),
        name: name.to_string().into(),
        caption: "personal".into(),
        group: group.to_string().into(),
        series,
        limit,
        value: "45%".into(),
        note: "7d".into(),
        kind: if limit.is_some() { SourceKind::Subscription } else { SourceKind::Key },
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let days = (0..14)
            .map(|d| UsageDay {
                label: d.to_string().into(),
                detail: format!("day {d}").into(),
                parts: vec![
                    (Series::new(0, 0), 2_000_000 + d as u64 * 150_000),
                    (Series::new(0, 1), 700_000),
                    (Series::new(1, 0), 500_000),
                ],
            })
            .collect();
        let log = self.log.clone();
        let (l1, l2, l3) = (log.clone(), log.clone(), log.clone());
        div().w(px(1144.)).child(
            UsageDashboard::new("usage")
                .range(UsageRange::Fortnight)
                .selection(Selection::All)
                .summary("5.84 M", "today")
                .sources(vec![
                    source("max-a", "Max", "Claude Code · 2 accounts", Series::new(0, 0), Some(0.45)),
                    source("max-b", "Max", "Claude Code · 2 accounts", Series::new(0, 1), Some(0.9)),
                    source("codex", "Codex", "Codex", Series::new(1, 0), Some(0.3)),
                    source("key", "Work", "OpenRouter", Series::new(2, 0), None),
                ])
                .days(days)
                .models(vec![UsageModel {
                    name: "Opus 5.5".into(),
                    tokens: 41_200_000,
                    series: Series::new(0, 0),
                    label: "41.2 M".into(),
                }])
                .total("60.1 M · $186")
                .sessions(vec![UsageSession {
                    id: "s1".into(),
                    title: "Fix the login bug".into(),
                    meta: "atelier · Claude Code · Opus 5.5".into(),
                    series: Series::new(0, 1),
                    tokens: "1,840,000".into(),
                    cost: "$14.20".into(),
                    days: vec![1., 3., 2., 4., 1.],
                    split: vec![("Cache read".into(), "1.52 M".into())],
                    footnote: "46 turns · 3 days".into(),
                }])
                .expanded(self.options.expanded.map(Into::into))
                .empty(self.options.empty.map(Into::into))
                .on_range(move |range, _, _| l1.borrow_mut().push(format!("range {}", range.days())))
                .on_select(move |selection, _, _| l2.borrow_mut().push(format!("select {selection:?}")))
                .on_expand(move |id, _, _| l3.borrow_mut().push(format!("expand {id:?}"))),
        )
    }
}

pub fn shown(host: Host, cx: &mut TestAppContext) -> &mut VisualTestContext {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(move |_, _| host);
    for _ in 0..3 {
        cx.run_until_parked();
    }
    cx
}

pub fn shown_with_log(options: Options, cx: &mut TestAppContext) -> (&mut VisualTestContext, Log) {
    let host = dashboard(options);
    let log = host.log.clone();
    (shown(host, cx), log)
}

/// A selector name that lives as long as the test run, for [`VisualTestContext::debug_bounds`].
pub fn name(text: impl Into<String>) -> &'static str {
    Box::leak(text.into().into_boxed_str())
}
