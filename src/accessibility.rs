//! Accessibility settings read from macOS.

use gpui_kit::App;

/// Copies macOS "Reduce motion" (System Settings > Accessibility > Display) into GPUI, so
/// `cx.reduce_motion()` matches it. [`crate::init`] calls it at startup and [`crate::watch_system`] each
/// time a window becomes active, because the user can change the setting while the app runs.
pub fn sync_reduce_motion(cx: &mut App) {
    cx.set_reduce_motion(system_reduce_motion());
}

#[cfg(target_os = "macos")]
fn system_reduce_motion() -> bool {
    use core_foundation::{
        base::{CFType, TCFType},
        boolean::CFBoolean,
        string::CFString,
    };
    use core_foundation_sys::preferences::{CFPreferencesAppSynchronize, CFPreferencesCopyAppValue};

    let domain = CFString::new("com.apple.universalaccess");
    let key = CFString::new("reduceMotion");
    // SAFETY: `domain` and `key` are live CoreFoundation strings for both calls. The copied value
    // follows the Create rule, so it is wrapped once and released on drop.
    unsafe {
        CFPreferencesAppSynchronize(domain.as_concrete_TypeRef());
        let value = CFPreferencesCopyAppValue(key.as_concrete_TypeRef(), domain.as_concrete_TypeRef());
        if value.is_null() {
            return false;
        }
        CFType::wrap_under_create_rule(value).downcast::<CFBoolean>().is_some_and(bool::from)
    }
}

#[cfg(not(target_os = "macos"))]
fn system_reduce_motion() -> bool {
    false
}
