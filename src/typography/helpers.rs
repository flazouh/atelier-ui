use std::borrow::Cow;

use gpui_kit::App;

pub(crate) fn load_fonts(cx: &mut App) {
    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(include_bytes!("../../assets/fonts/Geist-Regular.ttf")),
        Cow::Borrowed(include_bytes!("../../assets/fonts/Geist-Medium.ttf")),
        Cow::Borrowed(include_bytes!("../../assets/fonts/Geist-SemiBold.ttf")),
        Cow::Borrowed(include_bytes!("../../assets/fonts/GeistMono-Regular.ttf")),
        Cow::Borrowed(include_bytes!("../../assets/fonts/GeistMono-Medium.ttf")),
    ];
    cx.text_system()
        .add_fonts(fonts)
        .expect("the embedded Geist fonts are valid");
}

/// Match CSS `-webkit-font-smoothing: antialiased`, which beui uses. Without this,
/// macOS thickens light text on dark backgrounds.
///
/// GPUI reads `AppleFontSmoothing` from this app's own preferences, so the value is
/// written there and macOS keeps it in the app's preferences file. It affects only this app.
#[cfg(target_os = "macos")]
pub(crate) fn disable_font_smoothing() {
    use core_foundation::{base::TCFType, number::CFNumber, string::CFString};
    use core_foundation_sys::preferences::{
        CFPreferencesSetAppValue, kCFPreferencesCurrentApplication,
    };

    let key = CFString::new("AppleFontSmoothing");
    let zero = CFNumber::from(0i32);
    // SAFETY: both values are live CoreFoundation objects for the duration of the call.
    unsafe {
        CFPreferencesSetAppValue(
            key.as_concrete_TypeRef(),
            zero.as_CFTypeRef(),
            kCFPreferencesCurrentApplication,
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn disable_font_smoothing() {}
