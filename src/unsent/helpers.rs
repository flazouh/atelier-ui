use gpui_kit::SharedString;

pub fn count_text(count: usize) -> SharedString {
    if count == 1 {
        "1 unsent comment".into()
    } else {
        format!("{count} unsent comments").into()
    }
}

pub fn send_text(count: usize) -> SharedString {
    if count == 1 {
        "Send it".into()
    } else {
        format!("Send all {count}").into()
    }
}
