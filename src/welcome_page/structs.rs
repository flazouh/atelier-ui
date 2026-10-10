use std::{rc::Rc, time::Instant};

use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, linear_color_stop, linear_gradient, relative,
};

use super::{
    consts::{ACTION_TOP, HERO_B_PATH, HERO_PATH, HERO_RATIO, LINE_ALPHA, LINE_SIZE, LINE_TOP, MARK_SIZE, RISE, SCRIM_ALPHA},
    helpers::{frame, intro_done},
};
use crate::{
    AtelierMark, Button, ButtonSize, ButtonVariant, IconName,
    motion,
    scale::px,
    theme::{Appearance, Theme},
    typography::FONT_FAMILY,
};

type Continue = Rc<dyn Fn(&mut Window, &mut App)>;

/// The first page: the gradient, the mark, one line and one button.
#[derive(IntoElement)]
pub struct WelcomePage {
    id: ElementId,
    line: SharedString,
    action: SharedString,
    on_continue: Option<Continue>,
}

/// When the page first drew, so every frame knows how far the opening has come.
struct Motion {
    start: Instant,
}

impl WelcomePage {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            line: "Welcome. Your workshop for building with agents.".into(),
            action: "Start crafting".into(),
            on_continue: None,
        }
    }
    /// The line of welcome under the mark.
    pub fn line(mut self, line: impl Into<SharedString>) -> Self {
        self.line = line.into();
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

/// A picture that fills the page at `zoom` times its size, centred, so a zoom shows no edge.
fn picture(path: &'static str, zoom: f32, opacity: f32) -> impl IntoElement {
    let edge = -(zoom - 1.) / 2.;
    let mut img = gpui_kit::img(path).absolute().w(relative(zoom)).h(relative(zoom)).left(relative(edge)).top(relative(edge));
    img.style().aspect_ratio = Some(HERO_RATIO);
    <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(img, ObjectFit::Cover).opacity(opacity)
}

impl RenderOnce for WelcomePage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, _| Motion { start: motion::now() });
        let elapsed = motion::now().duration_since(motion.read(cx).start).as_secs_f32();
        let f = frame(elapsed, reduce);
        // The breath never ends, so the page asks for the next frame as long as it is shown.
        if !reduce {
            window.request_animation_frame();
        }
        let _ = intro_done(elapsed);
        // The picture is dark whatever the theme, so the words on it are the dark theme's.
        let dark = Theme::of(Appearance::Dark);
        let (light, ground) = (dark.foreground, dark.background);
        let mut scrim_top = ground;
        scrim_top.a = 0.;
        let mut scrim_bottom = ground;
        scrim_bottom.a = SCRIM_ALPHA;
        let backdrop = div()
            .debug_selector(|| "welcome-picture".into())
            .absolute()
            .inset_0()
            .overflow_hidden()
            .child(picture(HERO_PATH, f.zoom, f.picture))
            .child(picture(HERO_B_PATH, f.zoom, f.picture * f.breath))
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .bg(linear_gradient(180., linear_color_stop(scrim_top, 0.4), linear_color_stop(scrim_bottom, 1.))),
            );
        let rise = |part: f32| px(RISE * (1. - part));
        let go = self.on_continue;
        let button = Button::new((self.id.clone(), "continue"))
            .variant(ButtonVariant::Invert)
            .fill(light)
            .ink(ground)
            .size(ButtonSize::Lg)
            .label(self.action)
            .trailing_icon(IconName::ArrowForward)
            .debug_name("welcome-continue")
            .on_click(move |_, window, cx| {
                if let Some(f) = &go {
                    f(window, cx);
                }
            });
        let stack = div()
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(
                div()
                    .debug_selector(|| "welcome-mark".into())
                    .relative()
                    .top(rise(f.mark))
                    .opacity(f.mark)
                    .child(AtelierMark::new(MARK_SIZE)),
            )
            .child(
                div()
                    .debug_selector(|| "welcome-line".into())
                    .relative()
                    .mt(px(LINE_TOP))
                    .top(rise(f.line))
                    .opacity(f.line)
                    .text_size(px(LINE_SIZE))
                    .text_color(light.opacity(LINE_ALPHA))
                    .child(self.line),
            )
            .child(
                div()
                    .debug_selector(|| "welcome-action".into())
                    .relative()
                    .mt(px(ACTION_TOP))
                    .top(rise(f.action))
                    .opacity(f.action)
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
            .child(backdrop)
            .child(stack)
    }
}
