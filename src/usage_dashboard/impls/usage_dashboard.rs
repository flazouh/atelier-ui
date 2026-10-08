use std::rc::Rc;

use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use super::{
    super::{
        consts::PAD,
        enums::{Selection, UsageRange},
        structs::{UsageDashboard, UsageDay, UsageModel, UsageSession, UsageSource},
    },
    day_chart::day_chart,
    header::header,
    model_bars::model_bars,
    panel::panel,
    sessions::sessions,
    tiles::tiles,
};
use crate::{scale::px, theme::ActiveTheme, typography::FONT_FAMILY};

impl UsageDashboard {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            range: UsageRange::default(),
            selection: Selection::default(),
            sources: Vec::new(),
            summary: None,
            days: Vec::new(),
            models: Vec::new(),
            sessions: Vec::new(),
            expanded: None,
            total: None,
            empty: None,
            on_range: None,
            on_select: None,
            on_expand: None,
        }
    }

    pub fn range(mut self, range: UsageRange) -> Self {
        self.range = range;
        self
    }

    pub fn selection(mut self, selection: Selection) -> Self {
        self.selection = selection;
        self
    }

    pub fn sources(mut self, sources: Vec<UsageSource>) -> Self {
        self.sources = sources;
        self
    }

    /// The value and note of the "All accounts" tile: `5.84 M`, `today`.
    pub fn summary(mut self, value: impl Into<SharedString>, note: impl Into<SharedString>) -> Self {
        self.summary = Some((value.into(), note.into()));
        self
    }

    pub fn days(mut self, days: Vec<UsageDay>) -> Self {
        self.days = days;
        self
    }

    pub fn models(mut self, models: Vec<UsageModel>) -> Self {
        self.models = models;
        self
    }

    pub fn sessions(mut self, sessions: Vec<UsageSession>) -> Self {
        self.sessions = sessions;
        self
    }

    /// The id of the session that is open.
    pub fn expanded(mut self, id: Option<SharedString>) -> Self {
        self.expanded = id;
        self
    }

    /// The foot of By model: `60.1 M · $186`.
    pub fn total(mut self, total: impl Into<SharedString>) -> Self {
        self.total = Some(total.into());
        self
    }

    /// Plain words in place of the chart, the models and the sessions, for a selection with no use.
    pub fn empty(mut self, text: Option<SharedString>) -> Self {
        self.empty = text;
        self
    }

    pub fn on_range(mut self, f: impl Fn(UsageRange, &mut Window, &mut App) + 'static) -> Self {
        self.on_range = Some(Rc::new(f));
        self
    }

    pub fn on_select(mut self, f: impl Fn(Selection, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// Called with the session pressed, or `None` when the open one is pressed again.
    pub fn on_expand(mut self, f: impl Fn(Option<SharedString>, &mut Window, &mut App) + 'static) -> Self {
        self.on_expand = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for UsageDashboard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let head = header(&self, &theme, window, cx);
        let tiles = tiles(&self, &theme, window, cx);
        let rest = match &self.empty {
            Some(text) => vec![
                panel(&theme)
                    .debug_selector(|| "usage-empty".into())
                    .items_center()
                    .py(px(48.))
                    .text_size(px(13.))
                    .text_color(theme.muted_foreground)
                    .child(text.clone())
                    .into_any_element(),
            ],
            None => vec![
                div()
                    .flex()
                    .gap(px(10.))
                    .child(day_chart(&self, &theme, window, cx))
                    .child(model_bars(&self, &theme))
                    .into_any_element(),
                sessions(&self, &theme, window, cx),
            ],
        };
        div()
            .id(self.id.clone())
            .debug_selector(|| "usage-dashboard".into())
            .flex()
            .flex_col()
            .gap(px(10.))
            .w_full()
            .p(px(PAD))
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family(FONT_FAMILY)
            .child(head)
            .child(tiles)
            .children(rest)
    }
}
