//! Modal: beui.dev's Morphing Modal (`components/motion/morphing-modal.tsx`), placed in the centre. One panel
//! over a scrim, for the dialogs that ask something of the reader (the SSH form, the folder picker, a new
//! task). The owner keeps the open state: it draws a `Modal` while the dialog is open and stops when it
//! closes.
//!
//! - Panel: atelier's look, not the web's: the old dialogs' panel: a 12px corner, no border, the popover fill and shadow, and 16px of
//!   padding round the view. Its width is the owner's (384px in the web).
//! - Enter: 20px below and clear, then up into place on `Spring::PANEL` (`{ 420, 40, 0.5 }`). The scrim
//!   fades in over 200ms. The web blurs what is behind by 14px; here a dim theme overlay stands in for it.
//! - Morph: when the view's height changes (its `view` key changed, and its words are longer or shorter),
//!   the panel's height follows on the same spring, and the new view comes in 8px below and clear over 240ms.
//! - Close: Escape or a press on the scrim asks the owner to close (`on_close`), and focus goes back to
//!   what had it when the modal opened. A press in the panel does not close it. The first focusable thing
//!   named by [`Modal::focus`] takes focus on open.
//! - Under Reduce Motion the panel does not move and its height follows at once.
//!
//! What gpui cannot draw is left out: the panel's 0.97 scale, the blur of the views as they change, and the
//! exit (the owner removes the modal at once).

mod helpers;
mod structs;
mod types;

pub use helpers::{panel_height, scrim};
pub use structs::Modal;
pub use types::{BORDER, CORNER, ENTER_Y, PAD, PANEL, VIEW_Y};

#[cfg(test)]
mod tests;
