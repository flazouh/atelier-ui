use super::super::structs::Work;

impl Work {
    pub fn new(working: usize, needs_you: usize) -> Self {
        Self { working, needs_you }
    }

    pub fn is_idle(&self) -> bool {
        self.working == 0 && self.needs_you == 0
    }

    /// `2 working`, `1 needs you`.
    pub fn working_words(&self) -> String {
        format!("{} working", self.working)
    }

    pub fn needs_you_words(&self) -> String {
        if self.needs_you == 1 {
            "1 needs you".into()
        } else {
            format!("{} need you", self.needs_you)
        }
    }
}
