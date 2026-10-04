use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    App,
    ClickEvent,
    ElementId,
    FontWeight,
    InteractiveElement,
    IntoElement,
    ParentElement,
    RenderOnce,
    SharedString,
    StatefulInteractiveElement,
    Styled,
    Window,
    base::text::MarkdownExtensions,
    component::text::{TextView, TextViewStyle},
    div,
    prelude::FluentBuilder,
    rems,
};

use crate::scale::px;
use crate::{
    ClickHandler,
    copy_feedback::CopyFeedback,
    focus::PressStop,
    icon::{Icon, IconName},
    motion::{Channel, Curve, ease},
    pr::PrChipData,
    pr_chip::{PrChips, PrOpenHandler, link_prs},
    reveal::Reveal,
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::AgentTextStatus;

/// One entry in the source summary.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentTextSource {
    pub title: SharedString,
    pub domain: SharedString,
}

impl AgentTextSource {
    pub fn new(title: impl Into<SharedString>, domain: impl Into<SharedString>) -> Self {
        Self { title: title.into(), domain: domain.into() }
    }
}

#[derive(IntoElement)]
pub struct AgentText {
    id: ElementId,
    markdown: SharedString,
    status: AgentTextStatus,
    copy_text: Option<SharedString>,
    on_retry: Option<ClickHandler>,
    pub(super) sources: Vec<AgentTextSource>,
    pub(super) pr_resolver: Option<Rc<dyn Fn(u64) -> Option<PrChipData>>>,
    on_open_pr: Option<PrOpenHandler>,
    fade_tail: bool,
    fade_into: Option<gpui_kit::Hsla>,
}

impl AgentText {
    pub fn new(id: impl Into<ElementId>, markdown: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            markdown: markdown.into(),
            status: AgentTextStatus::default(),
            copy_text: None,
            on_retry: None,
            sources: Vec::new(),
            pr_resolver: None,
            on_open_pr: None,
            fade_tail: false,
            fade_into: None,
        }
    }
    /// While the text streams, draws the paragraph still growing in runs that fade in by their age ([`crate::stream_text`]).
    /// Off by default: a text shown whole, such as a description, has nothing to fade.
    pub fn fade_tail(mut self, on: bool) -> Self {
        self.fade_tail = on;
        self
    }
    /// The tone of the surface the text sits on, which a piece fades from. Defaults to `card`.
    pub fn fade_into(mut self, color: impl Into<gpui_kit::Hsla>) -> Self {
        self.fade_into = Some(color.into());
        self
    }
    /// Looks up a `#N` in the text. A number it returns data for shows as a chip.
    pub fn pr_resolver(mut self, resolve: impl Fn(u64) -> Option<PrChipData> + 'static) -> Self {
        self.pr_resolver = Some(Rc::new(resolve));
        self
    }

    /// Receives a pressed chip.
    pub fn on_open_pr(mut self, handler: impl Fn(&PrChipData, &mut Window, &mut App) + Send + Sync + 'static) -> Self {
        self.on_open_pr = Some(Arc::new(handler));
        self
    }

    pub fn status(mut self, status: AgentTextStatus) -> Self {
        self.status = status;
        self
    }

    /// Enables the copy action, writing this text to the clipboard.
    pub fn copy_text(mut self, text: impl Into<SharedString>) -> Self {
        self.copy_text = Some(text.into());
        self
    }

    pub fn on_retry(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_retry = Some(Rc::new(handler));
        self
    }

    pub fn sources(mut self, sources: Vec<AgentTextSource>) -> Self {
        self.sources = sources;
        self
    }
}

struct ResponseMotion {
    copy: CopyFeedback,
    pub(super) sources: Reveal,
    /// The actions row fading up into place.
    reveal: Channel,
}

impl RenderOnce for AgentText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let streaming = self.status == AgentTextStatus::Streaming;
        let complete = self.status == AgentTextStatus::Complete;
        let can_copy = self.copy_text.is_some();
        let can_retry = self.on_retry.is_some();
        let has_sources = !self.sources.is_empty();
        let should_show_actions = !streaming && (can_copy || can_retry || complete || has_sources);

        let motion = window.use_keyed_state(self.id.clone(), cx, move |_, _| ResponseMotion {
            copy: CopyFeedback::default(),
            sources: Reveal::new(false),
            reveal: Channel::new(if should_show_actions { 1. } else { 0. }),
        });
        motion.update(cx, |m, _| {
            let want = if should_show_actions { 1. } else { 0. };
            if (m.reveal.target() - want).abs() > 0.001 {
                m.reveal.animate(want, Curve::Ease(0.22, ease::OUT), 0., reduce);
            }
        });

        let m = motion.read(cx);
        let copied = m.copy.copied();
        // The copy button's reset runs on its own background timer, not on this render loop: it never
        // needs a frame requested just to wait out the 1.6s.
        if m.reveal.is_running() || m.sources.is_moving() {
            window.request_animation_frame();
        }
        let (reveal, chevron, sources_reveal) = (m.reveal.value(), m.sources.chevron.value(), m.sources.reveal.value());
        let sources_height = m.sources.height.clone();

        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let child_id = |name: &'static str| ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), name.into());

        let action = |name: &'static str, icon: IconName, active: bool, window: &mut Window, cx: &mut App| {
            div()
                .id(child_id(name))
                .press_stop(ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), format!("{name}-focus").into()), radius::md(), window, cx)
                .flex()
                .size(px(28.))
                .items_center()
                .justify_center()
                .rounded(radius::md())
                .cursor_pointer()
                .text_color(if active { theme.foreground } else { muted })
                .when(active, |d| d.bg(theme.card_strong))
                .hover(|s| s.bg(theme.card_strong).text_color(theme.foreground))
                .child(Icon::new(icon).size(px(14.)))
        };

        let mut row = div().flex().items_center().gap(px(2.));
        if let Some(text) = self.copy_text.clone() {
            let copy_state = motion.clone();
            row = row.child(action("copy", if copied { IconName::Check } else { IconName::Copy }, false, window, cx).on_click(
                move |_, _, cx| {
                    CopyFeedback::click(&copy_state, |m: &mut ResponseMotion| &mut m.copy, text.to_string(), cx);
                },
            ));
        }
        if let Some(handler) = self.on_retry.clone() {
            row = row.child(
                action("retry", IconName::Refresh, false, window, cx)
                    .on_click(move |event, window, cx| handler(event, window, cx)),
            );
        }
        let source_count = self.sources.len();
        if has_sources {
            let chips = div().flex().items_center().children(self.sources.iter().take(3).enumerate().map(
                |(index, source)| {
                    div()
                        .when(index > 0, |d| d.ml(px(-6.)))
                        .flex()
                        .size(px(18.))
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .bg(theme.card_strong)
                        .text_size(px(9.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(muted)
                        .child(source.domain.chars().next().map(|c| c.to_ascii_uppercase().to_string()).unwrap_or_default())
                },
            ));
            let count_label =
                format!("{source_count} {}", if source_count == 1 { "source" } else { "sources" });
            let sources_toggle = motion.clone();
            row = row.child(
                div()
                    .id(child_id("sources"))
                    .ml(px(4.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .min_h(px(28.))
                    .px(px(6.))
                    .rounded(radius::md())
                    .cursor_pointer()
                    .text_size(TextSize::Xs.font_size())
                    .text_color(muted)
                    .hover(|s| s.text_color(theme.foreground))
                    .on_click(move |_, _, cx| {
                        let reduce = cx.reduce_motion();
                        sources_toggle.update(cx, |m, cx| {
                            let open = !m.sources.open;
                            m.sources.set_open(open, reduce);
                            cx.notify();
                        })
                    })
                    .child(chips)
                    .child(count_label)
                    .child(
                        Icon::new(IconName::ChevronDown)
                            .size(px(12.))
                            .color(theme.faint())
                            .turn(chevron / 360.),
                    ),
            );
        }

        let sources_panel = (has_sources && sources_reveal > 0.001).then(|| {
            let panel = div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .rounded(radius::xl())
                .bg(theme.card_strong)
                .p(px(8.))
                .children(self.sources.into_iter().map(|source| {
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(TextSize::Xs.font_size())
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground)
                                .child(source.title),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(muted)
                                .child(source.domain),
                        )
                }));
            crate::reveal::body(div().pt(px(8.)).child(panel), sources_reveal, &sources_height)
        });

        let actions = (reveal > 0.001).then(|| {
            div()
                .relative()
                .top(px(4. * (1. - reveal)))
                .opacity(reveal)
                .mt(px(12.))
                .flex()
                .flex_col()
                .child(row)
                .when_some(sources_panel, |d, panel| d.child(panel))
        });

        // Tables are tiles, as in Acepe: each cell its own rounded tile, the header's one step stronger, a row's brighter
        // under the pointer, 13px text, no frame or border.
        let ink = theme.foreground;
        let mut cell_text = gpui_kit::StyleRefinement::default();
        cell_text.text.font_size = Some(px(13.).into());
        // Scroll layout: columns fit their words, shrink to a floor as the panel narrows, and a wide table scrolls.
        let mut table_frame = gpui_kit::StyleRefinement::default();
        table_frame.overflow.x = Some(gpui_kit::Overflow::Scroll);
        let style = TextViewStyle::default()
            .paragraph_gap(rems(0.75))
            .table(table_frame)
            .table_tiles(gpui_kit::component::text::TableTiles { head: ink.opacity(0.10), cell: ink.opacity(0.055), hover: ink.opacity(0.085) })
            .table_cell(cell_text);
        // The paragraph still growing draws in runs that fade in; what is finished stays Markdown.
        let mut tail: Option<(String, Vec<crate::stream_text::Piece>)> = None;
        let mut body = self.markdown.clone();
        if streaming && self.fade_tail && !reduce {
            let flow = window.use_keyed_state(child_id("flow"), cx, |_, _| crate::stream_text::Flow::default());
            let now = std::time::Instant::now();
            flow.update(cx, |f, _| f.observe(&self.markdown, now));
            let cut = crate::stream_text::split_tail(&self.markdown);
            let rest = &self.markdown[cut..];
            if !rest.is_empty() && crate::stream_text::is_plain(rest) {
                let flow = flow.read(cx);
                if flow.is_fading(now) {
                    window.request_animation_frame();
                }
                let runs = flow.alphas(&self.markdown, cut, now).into_iter().map(|(r, a)| (r.start - cut..r.end - cut, a)).collect();
                tail = Some((rest.to_string(), runs));
                body = SharedString::from(self.markdown[..cut].trim_end_matches('\n').to_string());
            }
        }
        // A line break inside a paragraph stays one, as in Claude's own apps.
        let body = SharedString::from(crate::soft_breaks::keep(&body));
        // Known pull requests become links that the chip plugin draws.
        let (markdown, chips) = match &self.pr_resolver {
            Some(resolve) => {
                let (markdown, chips) = link_prs(&body, |n| resolve(n));
                let plugin = PrChips { id: self.id.clone(), chips: Arc::new(chips), on_open: self.on_open_pr.clone() };
                (SharedString::from(markdown), Some(MarkdownExtensions::default().plugin(plugin)))
            }
            None => (body.clone(), None),
        };
        let mut text = TextView::markdown(self.id.clone(), markdown).style(style).selectable(true);
        if let Some(chips) = chips {
            text = text.markdown_extensions(chips);
        }
        let content = div()
            .w_full()
            .text_color(theme.foreground.opacity(0.9))
            .text_size(TextSize::Sm.font_size())
            .line_height(px(24.))
            .when(!body.trim().is_empty() || tail.is_none(), |d| d.child(text))
            .when_some(tail, |d, (rest, runs)| {
                // A highlight colour is blended over the text\s own, so a piece fades by taking the surface\s tone at the
                // strength it has left to go. A GlyphText, so a fading piece keeps the kerning it will have once it settles.
                let surface = self.fade_into.unwrap_or(theme.card);
                let highlights = runs.into_iter().map(|(range, alpha)| {
                    (range, gpui_kit::HighlightStyle { color: Some(surface.opacity(1. - alpha)), ..Default::default() })
                });
                // The gap a paragraph has, so the tail lays out as one more paragraph of the same text.
                d.child(
                    div()
                        .debug_selector(|| "stream-tail".into())
                        .when(!body.trim().is_empty(), |d| d.mt(rems(0.75)))
                        .child(crate::glyph_text::GlyphText::new(rest).highlights(highlights)),
                )
            });
        div().flex().flex_col().w_full().child(content).when_some(actions, |d, a| d.child(a))
    }
}
