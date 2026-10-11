use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, ImgResourceLoader, InteractiveElement, IntoElement, ObjectFit, ParentElement,
    RenderOnce, Resource, SharedString, Styled, Window, div,
};

use super::{
    consts::{EDGE, HERO_PATH, HERO_RATIO, LIFT, MARK_SIZE, SIDE, TEXT_ALPHA, TEXT_LINE, TEXT_SIZE, TEXT_TOP, TEXT_WIDTH, TITLE_SIZE},
    helpers::{Pace, action, moving, picture, shown, text_pace, title_pace},
};
use crate::{
    AtelierMark, Button, ButtonSize, ButtonVariant, IconName,
    entrance::Entrance,
    motion,
    scale::px,
    stream_text::Streamed,
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

/// When the picture was loaded, which starts the page.
struct Motion {
    start: Option<Instant>,
}

/// A text that arrives as a message does in a session. The block fades in and rises as its first word comes
/// ([`Entrance`]), and its words come one at a time at `pace`, each fading in as an answer's do ([`Streamed`]).
/// Before the page has started the text is only placed, with no ink: the entrance counts from the frame it first
/// draws in, so it is drawn from the start on.
fn arriving(id: &ElementId, part: &'static str, text: &SharedString, pace: Pace, since: Option<f32>, reduce: bool) -> AnyElement {
    let words = |shown: usize| Streamed::new((id.clone(), SharedString::from(format!("{part}-words"))), text.clone()).shown(shown);
    match since {
        None => words(0).into_any_element(),
        Some(elapsed) => Entrance::new((id.clone(), part), words(shown(text, pace, elapsed, reduce)))
            .skip_initial(reduce)
            .delay(Duration::from_secs_f32(pace.at))
            .into_any_element(),
    }
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
    /// The title. It streams in one word at a time, before the line.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }
    /// The line under the title. It streams in after the title.
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
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, _| Motion { start: None });
        let now = motion::now();
        // The page starts when the picture is loaded, so no word comes in over an empty page. The window draws
        // this view again when the load ends. A picture that cannot be read starts the page too.
        let loaded = window.use_asset::<ImgResourceLoader>(&Resource::Embedded(HERO_PATH.into()), cx).is_some();
        let start = motion.update(cx, |m, _| {
            if loaded && m.start.is_none() {
                m.start = Some(now);
            }
            m.start
        });
        let since = start.map(|at| now.duration_since(at).as_secs_f32());
        let elapsed = since.unwrap_or(0.);
        let (title, text) = (self.title, self.text);
        let title_block = arriving(&self.id, "title", &title, title_pace(), since, reduce);
        let text_block = arriving(&self.id, "line", &text, text_pace(&title), since, reduce);
        // The words fade by themselves; the page asks for frames while a word or the button is still to come.
        if start.is_some() && moving(&title, &text, elapsed, reduce) {
            window.request_animation_frame();
        }
        // The picture is dark whatever the theme, so the words and the button on it are the dark theme's.
        let dark = Theme::of(Appearance::Dark);
        let (light, ground) = (dark.foreground, dark.background);
        let mut hero = gpui_kit::img(HERO_PATH).absolute().inset_0().size_full();
        hero.style().aspect_ratio = Some(HERO_RATIO);
        let hero = <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(hero, ObjectFit::Cover).opacity(picture(since, reduce));
        let go = self.on_continue;
        let button = Button::new((self.id.clone(), "continue"))
            // The design system's chip button: the arrow slides in its chip under the pointer.
            .variant(ButtonVariant::Primary)
            .fill(light)
            .ink(ground)
            .size(ButtonSize::Xl)
            .label(self.action)
            .chip(IconName::ArrowForward)
            .debug_name("welcome-continue")
            .on_click(move |_, window, cx| {
                if let Some(f) = &go {
                    f(window, cx);
                }
            });
        let words = div()
            .absolute()
            .inset_0()
            .px(px(SIDE))
            .pb(px(LIFT))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(
                div()
                    .debug_selector(|| "welcome-title".into())
                    .text_size(px(TITLE_SIZE))
                    .line_height(px(TITLE_SIZE))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(light)
                    .child(title_block),
            )
            .child(
                div()
                    .debug_selector(|| "welcome-text".into())
                    .mt(px(TEXT_TOP))
                    .max_w(px(TEXT_WIDTH))
                    .text_size(px(TEXT_SIZE))
                    .line_height(px(TEXT_LINE))
                    .text_color(light.opacity(TEXT_ALPHA))
                    .child(text_block),
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
                    .opacity(if start.is_some() { action(&title, &text, elapsed, reduce) } else { 0. })
                    .child(button),
            )
    }
}
