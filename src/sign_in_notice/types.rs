use gpui_kit::SharedString;

/// Where the sign-in stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignInState {
    /// Not started: the button signs in.
    Ready,
    /// The browser is open and the reader signs in there.
    Waiting,
    /// The last try did not finish; the words say why. The button tries again.
    Failed(SharedString),
    /// The agent runs on another host, where a browser here cannot sign it in. The host's name.
    Elsewhere(SharedString),
}

/// The button's words.
pub const SIGN_IN: &str = "Sign in";
/// The button's words while the browser waits.
pub const WAITING: &str = "Waiting…";
/// The button that leaves the wait.
pub const CANCEL: &str = "Cancel";
