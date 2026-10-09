use gpui_kit::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    Styled, Window, canvas, div, prelude::FluentBuilder,
};

use crate::{
    theme::radius,
    focus::ring_shadow,
    motion::now_millis,
    scale::px,
    theme::ActiveTheme,
    typography::FONT_FAMILY,
    update_button::{
        consts::{GAP, HEIGHT, PAD_LEFT, PAD_RIGHT, RING, SPIN_SWEEP, TEXT},
        enums::UpdateState,
        helpers::{look, paint_ring, ring_start},
        structs::{UpdateButton, UpdateMotion},
    },
};

impl RenderOnce for UpdateButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let state = self.state;
        let target = match state {
            UpdateState::Downloading(f) => f,
            _ => 0.,
        };
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, cx| UpdateMotion::new(target, cx));
        let (shown, hover, focus, moving) = motion.update(cx, |m, _| {
            if !state.pressable() {
                m.hovered = false;
            }
            let moving = m.advance(target, reduce);
            (m.fraction.value(), m.hover.value(), m.focus.clone(), moving)
        });
        let turning = matches!(state, UpdateState::Restarting) && !reduce;
        if moving || turning {
            window.request_animation_frame();
        }
        let pressable = state.pressable();
        let keyed = pressable && focus.is_focused(window) && window.last_input_was_keyboard();
        let colors = look(state, &theme, hover);
        let (arc_fraction, start) = match state {
            UpdateState::Downloading(_) => (shown, 0.),
            _ => (SPIN_SWEEP, ring_start(now_millis(), reduce)),
        };
        let (track, arc) = (colors.track, colors.arc);
        // Ready is only its label; the ring shows while the update downloads and while the app restarts.
        let lead = (!pressable).then(|| {
            div()
                .relative()
                .flex_none()
                .size(px(RING))
                .child(
                    canvas(|_, _, _| {}, move |bounds, _, window, _| paint_ring(arc_fraction, start, track, arc, bounds, window))
                        .size_full(),
                )
        });
        let on_click = self.on_click.filter(|_| pressable);
        let hover_motion = motion.clone();
        div()
            .id(self.id)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .h(px(HEIGHT))
            .pl(px(PAD_LEFT))
            .pr(px(PAD_RIGHT))
            .gap(px(GAP))
            .rounded(radius::lg())
            .bg(colors.fill)
            .text_color(colors.text)
            .font_family(FONT_FAMILY)
            .font_weight(FontWeight::NORMAL)
            .text_size(px(TEXT))
            .line_height(px(16.))
            .whitespace_nowrap()
            .when(keyed, |d| d.shadow(ring_shadow(&theme, theme.background)))
            .debug_selector(|| "update-button".into())
            .child(div().absolute().inset_0().debug_selector(move || state.selector().into()))
            .children(lead)
            .child(div().relative().child(self.label))
            .when(pressable, |d| {
                d.track_focus(&focus.tab_stop(true))
                    .cursor_pointer()
                    .on_hover(move |on, _, cx| {
                        hover_motion.update(cx, |m, cx| {
                            m.hovered = *on;
                            cx.notify();
                        })
                    })
            })
            .when_some(on_click, |d, handler| d.on_click(move |_, window, cx| handler(window, cx)))
    }
}
