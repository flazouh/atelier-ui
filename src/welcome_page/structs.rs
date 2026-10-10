use std::{rc::Rc, time::Instant};

use gpui_kit::{
    App, ElementId, HighlightStyle, InteractiveElement, IntoElement, ObjectFit, ParentElement, RenderOnce,
    SharedString, Styled, Window, div,
};

use super::{
    consts::{
        ACTION_TOP, CARD_CORNER, CARD_PAD, CARD_WIDTH, HERO_PATH, HERO_RATIO, MARK_SIZE, TEXT_LINE, TEXT_SIZE,
        TEXT_TOP,
    },
    helpers::{action, moving, shown},
};
use crate::{
    AtelierMark, Button, ButtonSize, ButtonVariant, IconName,
    glyph_text::GlyphText,
    motion,
    scale::px,
    stream_text::Flow,
    theme::{Appearance, Theme, popover_shadow},
    typography::FONT_FAMILY,
};

type Continue = Rc<dyn Fn(&mut Window, &mut App)>;

/// The first page: the gradient, and one card with the mark, the words and one button.
#[derive(IntoElement)]
pub struct WelcomePage {
    id: ElementId,
    text: SharedString,
    action: SharedString,
    on_continue: Option<Continue>,
}

/// When the page first drew, and which pieces of the words are still fading in.
struct Motion {
    start: Instant,
    flow: Flow,
}

impl WelcomePage {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            text: "Welcome to atelier, your workshop for building with agents. Open a folder, start a session, and craft."
                .into(),
            action: "Start crafting".into(),
            on_continue: None,
        }
    }
    /// The words of welcome. They stream in one at a time.
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = text.into();
        self
    }
    /// The button's words.
    pub fn action(mut self, action: impl Into<SharedString>) -> Self {
        self.action = action.into();
        self
    }
    /// Fires when the button is pressed.
    pub fn on_continue(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_continue = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for WelcomePage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, _| Motion { start: motion::now(), flow: Flow::default() });
        let now = motion::now();
        let elapsed = now.duration_since(motion.read(cx).start).as_secs_f32();
        // The words in so far: the flow notes each new piece, and says how far each is through its fade.
        let text = self.text;
        let cut = shown(&text, elapsed, reduce);
        let pieces = motion.update(cx, |m, _| {
            m.flow.observe(&text[..cut], now);
            m.flow.alphas(&text[..cut], 0, now)
        });
        let fading = motion.read(cx).flow.is_fading(now);
        if fading || moving(&text, elapsed, reduce) {
            window.request_animation_frame();
        }
        // The picture is dark whatever the theme, so the card and the words on it are the dark theme's.
        let dark = Theme::of(Appearance::Dark);
        let (light, ground, surface) = (dark.foreground, dark.background, dark.popover);
        // A piece fades by taking the card's tone at the strength it has left to go; the words not in yet take it whole.
        let mut highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = pieces
            .into_iter()
            .map(|(range, alpha)| (range, HighlightStyle { color: Some(dark.popover.opacity(1. - alpha)), ..Default::default() }))
            .collect();
        if cut < text.len() {
            highlights.push((cut..text.len(), HighlightStyle { color: Some(dark.popover), ..Default::default() }));
        }
        let mut hero = gpui_kit::img(HERO_PATH).absolute().inset_0().size_full();
        hero.style().aspect_ratio = Some(HERO_RATIO);
        let hero = <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(hero, ObjectFit::Cover);
        let go = self.on_continue;
        let button = Button::new((self.id.clone(), "continue"))
            .variant(ButtonVariant::Invert)
            .fill(light)
            .ink(ground)
            .size(ButtonSize::Lg)
            .wide()
            .label(self.action)
            .trailing_icon(IconName::ArrowForward)
            .debug_name("welcome-continue")
            .on_click(move |_, window, cx| {
                if let Some(f) = &go {
                    f(window, cx);
                }
            });
        let card = div()
            .debug_selector(|| "welcome-card".into())
            .w(px(CARD_WIDTH))
            .p(px(CARD_PAD))
            .rounded(px(CARD_CORNER))
            .bg(surface)
            .shadow(popover_shadow(&dark))
            .flex()
            .flex_col()
            .items_start()
            .child(div().debug_selector(|| "welcome-mark".into()).child(AtelierMark::new(MARK_SIZE)))
            .child(
                div()
                    .debug_selector(|| "welcome-text".into())
                    .mt(px(TEXT_TOP))
                    .w_full()
                    .text_size(px(TEXT_SIZE))
                    .line_height(px(TEXT_LINE))
                    .text_color(light)
                    .child(GlyphText::new(text.clone()).highlights(highlights)),
            )
            .child(
                div()
                    .debug_selector(|| "welcome-action".into())
                    .mt(px(ACTION_TOP))
                    .w_full()
                    .opacity(action(&text, elapsed, reduce))
                    .child(button),
            );
        div()
            .id(self.id)
            .debug_selector(|| "welcome-page".into())
            .relative()
            .size_full()
            .overflow_hidden()
            .font_family(FONT_FAMILY)
            .bg(ground)
            .child(div().debug_selector(|| "welcome-picture".into()).absolute().inset_0().overflow_hidden().child(hero))
            .child(div().absolute().inset_0().flex().items_center().justify_center().child(card))
    }
}
