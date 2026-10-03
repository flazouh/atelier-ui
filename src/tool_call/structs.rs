use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    ScrollHandle, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, relative,
};

use crate::scale::px;
use crate::{
    copy_feedback::CopyFeedback,
    file_icon::FileIcon,
    focus::PressStop,
    icon::{Icon, IconName},
    reveal::Reveal,
    status_mark::StatusMark,
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use crate::preview_clamp::{self, EXPANDED_ROWS, Press, ROW_HEIGHT};
use super::types::{CARD_HEADER_HEIGHT, CARD_HEADER_PAD_X, MAX_OUTPUT_HEIGHT, ToolStatus};
use super::helpers::follow_status;

#[derive(IntoElement)]
pub struct ToolCall {
    id: ElementId,
    pub(super) title: SharedString,
    pub(super) tool: SharedString,
    pub(super) status: ToolStatus,
    pub(super) meta: Option<SharedString>,
    pub(super) output: Option<SharedString>,
    pub(super) file: Option<SharedString>,
    pub(super) icon: Option<IconName>,
    pub(super) flat: bool,
    default_open: bool,
    collapse_on_complete: bool,
    preview_rows: Option<usize>,
    on_open: Option<OpenHandler>,
}

type OpenHandler = Rc<dyn Fn(&mut Window, &mut App)>;

impl ToolCall {
    /// `title` says what happened ("Ran tests"); `tool` names the call ("Bash", "cargo test").
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            tool: SharedString::default(),
            status: ToolStatus::Running,
            meta: None,
            output: None,
            file: None,
            icon: None,
            flat: false,
            default_open: false,
            collapse_on_complete: true,
            preview_rows: None,
            on_open: None,
        }
    }

    /// Clips the output to its last `rows` lines, which do not scroll, until the reader presses it: the press opens it to a
    /// taller view, and a second one folds it. Without this the output scrolls in a viewport of its own.
    pub fn preview_rows(mut self, rows: usize) -> Self {
        self.preview_rows = Some(rows);
        self
    }

    /// Runs when the reader presses a clipped output open (see [`Self::preview_rows`]).
    pub fn on_open(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }

    /// A plain row with no card and little height, for reading and searching, which come in runs, and for a call
    /// that already sits inside a card.
    pub fn flat(mut self) -> Self {
        self.flat = true;
        self
    }

    /// Opens on first render, when there is output to show, even if the call has finished. The user's choice wins after that.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Closes itself the moment the call ends. On by default; off, a call stays as it is until the reader folds it.
    pub fn collapse_on_complete(mut self, collapse: bool) -> Self {
        self.collapse_on_complete = collapse;
        self
    }

    /// The icon for the kind of call, before the title.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// The tool or its argument, shown in mono after the title.
    pub fn tool(mut self, tool: impl Into<SharedString>) -> Self {
        self.tool = tool.into();
        self
    }

    pub fn status(mut self, status: ToolStatus) -> Self {
        self.status = status;
        self
    }

    /// The file the call is about, shown as the tool with its own icon: `Read` and
    /// `crates/beui/src/theme.rs`.
    pub fn file(mut self, path: impl Into<SharedString>) -> Self {
        let path = path.into();
        self.tool = path.clone();
        self.file = Some(path);
        self
    }

    /// A short fact after the title, such as "1.2s".
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    pub fn output(mut self, output: impl Into<SharedString>) -> Self {
        self.output = Some(output.into());
        self
    }
}

pub(super) struct CallMotion {
    pub(super) status: ToolStatus,
    pub(super) had_body: bool,
    pub(super) disclosure: Reveal,
    copy: CopyFeedback,
    /// The output's scroll, so the wheel can stay inside it while it has more to show.
    scroll: ScrollHandle,
    /// A clipped output the reader pressed open.
    expanded: bool,
}

impl RenderOnce for ToolCall {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let status = self.status;
        let has_body = self.output.is_some();
        // beui's `defaultOpen` is true while running; a finished call starts closed.
        let open = (status == ToolStatus::Running || self.default_open) && has_body;
        let motion = window.use_keyed_state(self.id.clone(), cx, move |_, _| CallMotion {
            status,
            had_body: has_body,
            disclosure: Reveal::new(open),
            copy: CopyFeedback::default(),
            scroll: ScrollHandle::new(),
            expanded: false,
        });
        follow_status(&motion, status, has_body, self.collapse_on_complete, reduce, cx);
        let m = motion.read(cx);
        // Nothing spins here any more, so only the open and close reveal needs frames.
        if m.disclosure.is_moving() {
            window.request_animation_frame();
        }
        let (reveal, chevron, copied, expanded) = (m.disclosure.reveal.value(), m.disclosure.chevron.value(), m.copy.copied(), m.expanded);
        let height = m.disclosure.height.clone();
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let child = |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());

        let flat = self.flat;
        let toggle = motion.clone();
        let header = div()
            .id(child("header"))
            .group("tool-header")
            .flex()
            .items_center()
            .gap(px(8.))
            .when(flat, |d| d.min_h(px(24.)).rounded(radius::md()))
            .when(!flat, |d| d.min_h(px(CARD_HEADER_HEIGHT)).px(px(CARD_HEADER_PAD_X)).rounded(radius::card()))
            .text_size(TextSize::Sm.font_size())
            .line_height(TextSize::Sm.line_height())
            .when(has_body, |d| {
                d.cursor_pointer().press_stop((self.id.clone(), "head-focus"), if flat { radius::md() } else { radius::card() }, window, cx).on_click(move |_, _, cx| {
                    let reduce = cx.reduce_motion();
                    toggle.update(cx, |m, cx| {
                        let open = !m.disclosure.open;
                        m.disclosure.set_open(open, reduce);
                        cx.notify();
                    })
                })
            })
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .items_baseline()
                    .gap(px(8.))
                    .when_some(self.icon, |d, icon| d.child(div().flex_none().self_center().text_color(theme.faint()).child(Icon::new(icon).size(px(14.)))))
                    // The name keeps its width, up to most of the row; the path gives way first.
                    .child(
                        div()
                            .flex_none()
                            .max_w(relative(0.6))
                            .truncate()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground.opacity(0.9))
                            .child(self.title),
                    )
                    .when_some(self.meta, |d, meta| {
                        d.child(div().flex_none().text_size(TextSize::Xs.font_size()).text_color(muted).child(meta))
                    })
                    .when_some(self.file.clone(), |d, path| d.child(div().flex_none().self_center().child(FileIcon::file(&path).size(px(12.)))))
                    .when(!self.tool.is_empty(), |d| {
                        d.child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .font_family(MONO_FONT_FAMILY)
                                .text_size(px(11.))
                                .text_color(muted)
                                .child(self.tool),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(4.))
                    // The status is an icon alone: the muted spinner while the call runs (nothing blue), a disc
                    // with its glyph once it has ended.
                    .child(if status == ToolStatus::Running {
                        crate::spinner::Spinner::new(child("running")).size(px(12.)).color(muted).into_any_element()
                    } else {
                        StatusMark::new(status.mark(&theme), px(12.)).into_any_element()
                    }),
            )
            .when(has_body, |d| {
                d.child(
                    div()
                        .flex_none()
                        .text_color(theme.faint())
                        .group_hover("tool-header", |s| s.text_color(muted))
                        .child(Icon::new(IconName::ChevronDown).size(px(14.)).turn(chevron / 360.)),
                )
            });

        let body = self.output.map(|output| {
            let copy_state = motion.clone();
            let text = output.to_string();
            // In a card the output runs to the card's left, right and bottom edges; flat, it is a well indented under the title.
            let body = if flat { div().pl(px(24.)).pt(px(6.)) } else { div() };
            let scroll = motion.read(cx).scroll.clone();
            // A clipped output shows its last lines, which do not scroll, until it is pressed open to a taller view.
            let total = text.lines().count();
            let clip = self.preview_rows.filter(|&rows| !expanded && total > rows);
            let shown = match clip {
                Some(rows) => text.lines().skip(total - rows).collect::<Vec<_>>().join("\n").into(),
                None => output,
            };
            let viewport = self.preview_rows.map_or(MAX_OUTPUT_HEIGHT, |_| EXPANDED_ROWS as f32 * ROW_HEIGHT + 24.);
            let text_box = div()
                .id(child("output"))
                .debug_selector(|| "tool-output".into())
                .p(px(12.))
                .font_family(MONO_FONT_FAMILY)
                .text_size(TextSize::Xs.font_size())
                .line_height(px(ROW_HEIGHT))
                .text_color(theme.foreground.opacity(0.85));
            let text_box = match clip {
                // The last lines stay in view: what does not fit runs off the top.
                Some(rows) => text_box.flex().flex_col().justify_end().max_h(px(rows as f32 * ROW_HEIGHT + 24.)).overflow_hidden().child(shown),
                None => text_box.max_h(px(viewport)).overflow_y_scroll().track_scroll(&scroll).child(shown),
            };
            // With a clip set, the output and its hint are one press target: they open, and tell the owner.
            let text_box = match self.preview_rows {
                Some(limit) => {
                    let clipped = total > limit;
                    let press = motion.clone();
                    let on_open = self.on_open.clone();
                    div()
                        .id(child("body"))
                        .cursor_pointer()
                        .on_click(move |_, window, cx| {
                            let pressed = press.update(cx, |m, cx| {
                                let pressed = Press::on(m.expanded, clipped);
                                m.expanded = pressed.expanded;
                                cx.notify();
                                pressed
                            });
                            if let (true, Some(open)) = (pressed.open, &on_open) {
                                open(window, cx);
                            }
                        })
                        .child(text_box)
                        .when(clipped, |d| d.child(preview_clamp::hint(total - limit, expanded, &theme)))
                        .into_any_element()
                }
                None => text_box.into_any_element(),
            };
            body.child(
                div()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .when(flat, |d| d.rounded(radius::xl()))
                    .bg(if flat { theme.card_strong } else { theme.background.opacity(0.5) })
                    // The output keeps the wheel while it scrolls; at its ends the wheel goes on to the panel.
                    .on_scroll_wheel(crate::scroll_chain::keep_inside(scroll.clone()))
                    .child(text_box)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.))
                            .px(px(8.))
                            .pb(px(6.))
                            .child(
                                div()
                                    .id(child("copy"))
                                    .flex()
                                    .size(px(28.))
                                    .items_center()
                                    .justify_center()
                                    .rounded(radius::md())
                                    .cursor_pointer()
                                    .text_color(muted)
                                    .hover(|s| s.bg(theme.muted_hover()).text_color(theme.foreground))
                                    .press_stop((self.id.clone(), "copy-focus"), crate::theme::radius::md(), window, cx)
                                    .on_click(move |_, _, cx| {
                                        CopyFeedback::click(&copy_state, |m: &mut CallMotion| &mut m.copy, text.clone(), cx);
                                    })
                                    .child(Icon::new(if copied { IconName::Check } else { IconName::Copy }).size(px(14.))),
                            )
                            .child(div().flex_1()),
                    ),
            )
        });

        div()
            .flex()
            .flex_col()
            .w_full()
            .when(!flat, |d| d.rounded(radius::card()).bg(theme.card_strong).overflow_hidden())
            .child(header)
            .when_some(body.filter(|_| reveal > 0.001), |d, body| {
            d.child(crate::reveal::body(body, reveal, &height))
        })
    }
}
