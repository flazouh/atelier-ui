use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, relative,
};

use super::{
    consts::{
        EDGE, HERO_PATH, HERO_RATIO, MARK_SIZE, SIDE, TEXT_ALPHA, TEXT_LINE, TEXT_SIZE, TEXT_TOP, TEXT_WIDTH,
        TITLE_PULL, TITLE_SIZE, TITLE_TOP,
    },
    consts::WORDS_AT,
    helpers::{action, moving, shown},
};
use crate::{
    AtelierMark, Button, ButtonSize, ButtonVariant, IconName,
    entrance::Entrance,
    glyph_text::GlyphText,
    motion,
    scale::px,
    stream_text::Flow,
    theme::{Appearance, Theme},
    typography::FONT_FAMILY,
};

type Continue = Rc<dyn Fn(&mut Window, &mut App)>;

/// The first page: the gradient, the mark, a title with one line, and one button.
#[derive(IntoElement)]
pub struct WelcomePage {
    id: ElementId,
    title: SharedString,
    text: SharedString,
    action: SharedString,
    on_continue: Option<Continue>,
}

/// When the page first drew, and which pieces of the line are still fading in.
struct Motion {
    start: Instant,
    flow: Flow,
}

impl WelcomePage {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: "Welcome to Atelier.".into(),
            text: "Your workshop for crafting with agents.".into(),
            action: "Start crafting".into(),
            on_continue: None,
        }
    }
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }
    /// The line under the title. It streams in one word at a time.
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
        let mut fades = motion.update(cx, |m, _| {
            m.flow.observe(&text[..cut], now);
            m.flow.alphas(&text[..cut], 0, now)
        });
        // The words not in yet keep their place, with no ink, so the line never moves as it fills.
        if cut < text.len() {
            fades.push((cut..text.len(), 0.));
        }
        if motion.read(cx).flow.is_fading(now) || moving(&text, elapsed, reduce) {
            window.request_animation_frame();
        }
        // The picture is dark whatever the theme, so the words and the button on it are the dark theme's.
        let dark = Theme::of(Appearance::Dark);
        let (light, ground) = (dark.foreground, dark.background);
        let mut hero = gpui_kit::img(HERO_PATH).absolute().inset_0().size_full();
        hero.style().aspect_ratio = Some(HERO_RATIO);
        let hero = <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(hero, ObjectFit::Cover);
        let go = self.on_continue;
        let button = Button::new((self.id.clone(), "continue"))
            .variant(ButtonVariant::Invert)
            .fill(light)
            .ink(ground)
            .size(ButtonSize::Xl)
            .label(self.action)
            .trailing_icon(IconName::ArrowForward)
            .debug_name("welcome-continue")
            .on_click(move |_, window, cx| {
                if let Some(f) = &go {
                    f(window, cx);
                }
            });
        let words = div()
            .absolute()
            .left(px(SIDE))
            .right(px(SIDE))
            .top(relative(TITLE_TOP))
            .child(
                div()
                    .debug_selector(|| "welcome-title".into())
                    .ml(px(-TITLE_PULL))
                    .text_size(px(TITLE_SIZE))
                    .line_height(px(TITLE_SIZE))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(light)
                    .child(self.title),
            )
            // The line arrives as a message does in a session: the block fades in and rises as its first words come,
            // and the words fade in one by one.
            .child(
                Entrance::new(
                    (self.id.clone(), "line"),
                    div()
                        .debug_selector(|| "welcome-text".into())
                        .mt(px(TEXT_TOP))
                        .max_w(px(TEXT_WIDTH))
                        .text_size(px(TEXT_SIZE))
                        .line_height(px(TEXT_LINE))
                        .text_color(light.opacity(TEXT_ALPHA))
                        .child(GlyphText::new(text.clone()).fades(fades)),
                )
                .skip_initial(reduce)
                .delay(Duration::from_secs_f32(WORDS_AT)),
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
            .child(
                div()
                    .debug_selector(|| "welcome-mark".into())
                    .absolute()
                    .left(px(SIDE))
                    .top(px(EDGE))
                    .child(AtelierMark::new(MARK_SIZE)),
            )
            .child(words)
            .child(
                div()
                    .debug_selector(|| "welcome-action".into())
                    .absolute()
                    .right(px(SIDE))
                    .bottom(px(EDGE))
                    .opacity(action(&text, elapsed, reduce))
                    .child(button),
            )
    }
}
