use std::rc::Rc;
use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ObjectFit, ParentElement, RenderOnce,
    SharedString, Styled, Window, div,
};
use crate::scale::px;
use crate::{
    AtelierMark, Button, ButtonSize, ButtonVariant,
    release_sheet::HERO_PATH,
    theme::{ActiveTheme, Appearance, Theme},
    typography::FONT_FAMILY,
};
use super::consts::{
    ACTION_TOP, BAND_HEIGHT, BAND_WIDTH, BAR_GAP, BAR_HEIGHT, BAR_REST_ALPHA, BAR_WIDTH, BARS_BOTTOM,
    MARK_SIZE, MARK_TOP, SIDE, TEXT_SIZE, TEXT_TOP, TITLE_BOTTOM, TITLE_SIZE,
};
type Continue = Rc<dyn Fn(&mut Window, &mut App)>;
/// The first page of onboarding: a title on the changelog gradient, one sentence and one button.
#[derive(IntoElement)]
pub struct WelcomePage {
    id: ElementId,
    title: SharedString,
    text: SharedString,
    action: SharedString,
    step: usize,
    steps: usize,
    on_continue: Option<Continue>,
}
impl WelcomePage {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: "Welcome to Atelier".into(),
            text: "A code editor for working with agents.".into(),
            action: "Get started".into(),
            step: 1,
            steps: 4,
            on_continue: None,
        }
    }
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = text.into();
        self
    }
    /// The button's word.
    pub fn action(mut self, action: impl Into<SharedString>) -> Self {
        self.action = action.into();
        self
    }
    /// Where the setup stands: this page is `step` of `steps`, counted from 1.
    pub fn step(mut self, step: usize, steps: usize) -> Self {
        self.step = step;
        self.steps = steps;
        self
    }
    /// Fires when the button is pressed.
    pub fn on_continue(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_continue = Some(Rc::new(f));
        self
    }
}
impl RenderOnce for WelcomePage {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme: Theme = cx.theme().clone();
        // The picture is dark at the top left whatever the theme, so the words over it are the dark theme's.
        let light = Theme::of(Appearance::Dark).foreground;
        let mut hero = gpui_kit::img(HERO_PATH).w_full().h(px(BAND_HEIGHT));
        hero.style().aspect_ratio = Some(BAND_WIDTH / BAND_HEIGHT);
        let picture = div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h(px(BAND_HEIGHT))
            .overflow_hidden()
            .child(<gpui_kit::Img as gpui_kit::StyledImage>::object_fit(hero, ObjectFit::Fill));
        let band = div()
            .debug_selector(|| "welcome-band".into())
            .relative()
            .flex_none()
            .h(px(BAND_HEIGHT))
            .child(picture)
            .child(
                div()
                    .debug_selector(|| "welcome-mark".into())
                    .absolute()
                    .left(px(SIDE))
                    .top(px(MARK_TOP))
                    .child(AtelierMark::new(MARK_SIZE)),
            )
            .child(
                div()
                    .debug_selector(|| "welcome-title".into())
                    .absolute()
                    .left(px(SIDE))
                    .bottom(px(TITLE_BOTTOM))
                    .font_family(FONT_FAMILY)
                    .text_size(px(TITLE_SIZE))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(light)
                    .child(self.title),
            );
        let go = self.on_continue;
        let button = Button::new((self.id.clone(), "continue"))
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Lg)
            .label(self.action)
            .cap("⏎")
            .debug_name("welcome-continue")
            .on_click(move |_, window, cx| {
                if let Some(f) = &go {
                    f(window, cx);
                }
            });
        let bars = (1..=self.steps).map(|n| {
            let mut tone = theme.foreground;
            if n > self.step {
                tone.a *= BAR_REST_ALPHA;
            }
            div()
                .w(px(BAR_WIDTH))
                .h(px(BAR_HEIGHT))
                .rounded(px(BAR_HEIGHT / 2.))
                .bg(tone)
        });
        div()
            .id(self.id)
            .debug_selector(|| "welcome-page".into())
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .font_family(FONT_FAMILY)
            .bg(theme.background)
            .child(band)
            .child(
                div()
                    .px(px(SIDE))
                    .child(
                        div()
                            .debug_selector(|| "welcome-text".into())
                            .mt(px(TEXT_TOP))
                            .text_size(px(TEXT_SIZE))
                            .text_color(theme.muted_foreground)
                            .child(self.text),
                    )
                    .child(
                        div()
                            .debug_selector(|| "welcome-action".into())
                            .mt(px(ACTION_TOP))
                            .flex()
                            .child(button),
                    ),
            )
            .child(
                div()
                    .debug_selector(|| "welcome-bars".into())
                    .absolute()
                    .left(px(SIDE))
                    .bottom(px(BARS_BOTTOM))
                    .flex()
                    .gap(px(BAR_GAP))
                    .children(bars),
            )
    }
}
