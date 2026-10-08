use super::types::SignInState;

/// What the notice says about `agent`, signed in as `account` when it has a named one.
pub fn words(agent: &str, account: Option<&str>, state: &SignInState) -> String {
    match state {
        SignInState::Ready => match account {
            Some(account) => format!("{agent} is not signed in (account {account})."),
            None => format!("{agent} is not signed in."),
        },
        SignInState::Waiting => format!("Finish signing in to {agent} in your browser."),
        SignInState::Failed(why) => format!("{agent} is not signed in. {why}"),
        SignInState::Elsewhere(host) => format!("{agent} is not signed in on {host}. Sign in there, then send your message again."),
    }
}
