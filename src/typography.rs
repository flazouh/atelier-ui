//! Geist and Geist Mono, embedded so the app never depends on installed fonts.

use std::borrow::Cow;

use gpui_kit::{App, Pixels, };
use crate::scale::px;

pub const FONT_FAMILY: &str = "Geist";
/// Code, paths, commands, and numbers that change.
pub const MONO_FONT_FAMILY: &str = "Geist Mono";

/// Space between the segments of a status line, such as "Done in 38s" and "12 tool calls". Segments
/// part by space alone, never by a glyph such as a middle dot.
pub const SEGMENT_GAP: f32 = 12.;

/// Tailwind sizes used by beui, as (font size, line height).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
}

impl TextSize {
    pub fn font_size(self) -> Pixels {
        px(match self {
            Self::Xs => 12.,
            Self::Sm => 14.,
            Self::Base => 16.,
            Self::Lg => 18.,
            Self::Xl => 20.,
        })
    }

    pub fn line_height(self) -> Pixels {
        px(match self {
            Self::Xs => 16.,
            Self::Sm => 20.,
            Self::Base => 24.,
            Self::Lg => 28.,
            Self::Xl => 28.,
        })
    }
}

pub(crate) fn load_fonts(cx: &mut App) {
    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Regular.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Medium.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-SemiBold.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono-Regular.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono-Medium.ttf")),
    ];
    cx.text_system().add_fonts(fonts).expect("the embedded Geist fonts are valid");
}

/// Match CSS `-webkit-font-smoothing: antialiased`, which beui uses. Without this,
/// macOS thickens light text on dark backgrounds.
///
/// GPUI reads `AppleFontSmoothing` from this app's own preferences, so the value is
/// written there and macOS keeps it in the app's preferences file. It affects only this app.
#[cfg(target_os = "macos")]
pub(crate) fn disable_font_smoothing() {
    use core_foundation::{base::TCFType, number::CFNumber, string::CFString};
    use core_foundation_sys::preferences::{CFPreferencesSetAppValue, kCFPreferencesCurrentApplication};

    let key = CFString::new("AppleFontSmoothing");
    let zero = CFNumber::from(0i32);
    // SAFETY: both values are live CoreFoundation objects for the duration of the call.
    unsafe {
        CFPreferencesSetAppValue(key.as_concrete_TypeRef(), zero.as_CFTypeRef(), kCFPreferencesCurrentApplication);
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn disable_font_smoothing() {}
