//! WelcomePage: the first step of onboarding. A band of the changelog gradient holds the Atelier mark at its top left
//! and the title at its bottom left. Under it stand one muted sentence and one button, and at the foot a row of bars
//! shows how far the setup is. There is nothing else on the page: it asks for no choice, only a press.
//!
//! - It fills its parent. Put it in the window's content area.
//! - The picture is the one the [`ReleaseSheet`](crate::release_sheet::ReleaseSheet) uses, so the two pages share it.
mod consts;
mod structs;
pub use structs::WelcomePage;
#[cfg(test)]
mod tests;
