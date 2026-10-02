use super::types::PATH;

pub(crate) fn bytes(path: &str) -> Option<&'static [u8]> {
    (path == PATH).then_some(include_bytes!("../../assets/atelier-mark.svg"))
}
