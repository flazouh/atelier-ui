/// A chart colour: a hue for the provider and a shade for the account in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Series {
    /// 0 Claude orange, 1 Codex teal, 2 OpenRouter violet, 3 Anthropic API red.
    pub hue: usize,
    /// 0 base, 1 darker, 2 lighter. Several accounts of one provider share a hue and differ by shade.
    pub shade: usize,
}

impl Series {
    pub const fn new(hue: usize, shade: usize) -> Self {
        Self { hue, shade }
    }
}
