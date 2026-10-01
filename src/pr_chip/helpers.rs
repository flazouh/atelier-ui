use std::collections::HashMap;

use crate::{pr::PrChipData, pr_refs::pr_refs};
use super::types::SCHEME;

/// Rewrites every `#N` that `resolve` knows into a Markdown link to `atelier-pr:N`, and returns the data
/// for each. Numbers it does not know stay plain text.
pub fn link_prs(markdown: &str, resolve: impl Fn(u64) -> Option<PrChipData>) -> (String, HashMap<u64, PrChipData>) {
    let mut chips = HashMap::new();
    let mut out = String::with_capacity(markdown.len());
    let mut at = 0;
    for (range, number) in pr_refs(markdown) {
        let known = chips.contains_key(&number) || resolve(number).map(|pr| chips.insert(number, pr)).is_some();
        if known {
            out.push_str(&markdown[at..range.start]);
            out.push_str(&format!("[#{number}]({SCHEME}{number})"));
            at = range.end;
        }
    }
    out.push_str(&markdown[at..]);
    (out, chips)
}

/// The number in a link that [`link_prs`] wrote.
pub fn chip_number(url: &str) -> Option<u64> {
    url.strip_prefix(SCHEME)?.parse().ok()
}
