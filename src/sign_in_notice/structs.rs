use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div,
};

use super::{
    helpers::words,
    types::{CANCEL, SIGN_IN, SignInState, WAITING},
};
use crate::{
    button::{Button, ButtonVariant},
    icon::{Icon, IconName},
    menu::{Lead, lead_icon},
    scale::px,
    theme::{ActiveTheme, radius},
    typography::TextSize,
};

type SignIn = Rc<dyn Fn(&mut Window, &mut App)>;

/// The box over the composer for an agent that has no sign-in. Place it where the limit notice goes.
#[derive(IntoElement)]
pub struct SignInNotice {
    id: ElementId,
    agent: SharedString,
    lead: Lead,
    account: Option<SharedString>,
    state: SignInState,
    on_sign_in: Option<SignIn>,
    on_cancel: Option<SignIn>,
    action: Option<AnyElement>,
}

impl SignInNotice {
    pub fn new(id: impl Into<ElementId>, agent: impl Into<SharedString>, lead: Lead) -> Self {
        Self {
            id: id.into(),
            agent: agent.into(),
            lead,
            account: None,
            state: SignInState::Ready,
            on_sign_in: None,
            on_cancel: None,
            action: None,
        }
    }

    /// The account the session runs on, when it is a named one.
    pub fn account(mut self, account: Option<impl Into<SharedString>>) -> Self {
        self.account = account.map(Into::into);
        self
    }

    pub fn state(mut self, state: SignInState) -> Self {
        self.state = state;
        self
    }

    pub fn on_sign_in(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_sign_in = Some(Rc::new(f));
        self
    }

    /// Leaves the wait for the browser: shown as a button only while the state is `Waiting`.
    pub fn on_cancel(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(f));
        self
    }

    /// Another way on, after the button: a handoff.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    fn button(&self) -> Option<Button> {
        let waiting = self.state == SignInState::Waiting;
        if matches!(self.state, SignInState::Elsewhere(_)) {
            return None;
        }
        let button = Button::new(ElementId::from((self.id.clone(), "sign-in")))
            .label(if waiting { WAITING } else { SIGN_IN })
            .variant(ButtonVariant::Primary)
            .debug_name("sign-in-button")
            .disabled(waiting);
        Some(match (&self.on_sign_in, waiting) {
            (Some(f), false) => {
                let f = f.clone();
                button.on_click(move |_, window, cx| f(window, cx))
            }
            _ => button,
        })
    }
}

impl SignInNotice {
    fn cancel(&self) -> Option<Button> {
        let f = self
            .on_cancel
            .clone()
            .filter(|_| self.state == SignInState::Waiting)?;
        Some(
            Button::new(ElementId::from((self.id.clone(), "cancel")))
                .label(CANCEL)
                .variant(ButtonVariant::Ghost)
                .debug_name("sign-in-cancel")
                .on_click(move |_, window, cx| f(window, cx)),
        )
    }
}

impl RenderOnce for SignInNotice {
    fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let text = words(&self.agent, self.account.as_deref(), &self.state);
        let failed = matches!(self.state, SignInState::Failed(_));
        let lead = lead_icon(&self.agent, self.lead.clone(), 14., &theme);
        let button = self.button();
        let cancel = self.cancel();
        div()
            .debug_selector(|| "sign-in-notice".into())
            .mx(px(12.))
            .mb(px(8.))
            .px(px(12.))
            .py(px(8.))
            .rounded(radius::lg())
            .bg(theme.card_strong)
            .flex()
            .items_center()
            .gap(px(8.))
            .text_size(TextSize::Xs.font_size())
            .child(Icon::new(IconName::Lock).size(px(14.)).color(if failed {
                theme.danger
            } else {
                theme.warning
            }))
            .child(lead)
            .child(div().flex_1().min_w_0().whitespace_normal().child(text))
            .children(button)
            .children(cancel)
            .children(self.action.take())
    }
}
