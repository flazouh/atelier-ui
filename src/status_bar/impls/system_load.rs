use super::super::structs::SystemLoad;

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

impl SystemLoad {
    /// The memory in use, 0 to 1.
    pub fn memory_fraction(&self) -> f32 {
        if self.memory_total == 0 {
            0.
        } else {
            (self.memory_used as f64 / self.memory_total as f64).clamp(0., 1.) as f32
        }
    }

    /// `23%`.
    pub fn cpu_words(&self) -> String {
        format!("{}%", (self.cpu.clamp(0., 1.) * 100.).round() as u32)
    }

    /// `12.4 / 32 GB`.
    pub fn memory_words(&self) -> String {
        format!("{} / {} GB", gib(self.memory_used), gib(self.memory_total))
    }

    pub fn cpu_tooltip(&self) -> String {
        format!("Processor: {} in use", self.cpu_words())
    }

    pub fn memory_tooltip(&self) -> String {
        let mut lines = vec![format!(
            "Memory: {} in use ({}%)",
            self.memory_words(),
            (self.memory_fraction() * 100.).round() as u32
        )];
        lines.extend(
            self.app_memory
                .map(|bytes| format!("atelier holds {}", sized(bytes))),
        );
        lines.join("\n")
    }
}

/// Gigabytes, with a decimal below ten.
fn gib(bytes: u64) -> String {
    let gib = bytes as f64 / GIB;
    if gib >= 10. {
        format!("{gib:.0}")
    } else {
        format!("{gib:.1}")
    }
}

/// A size in the unit that fits: `512 MB`, `1.4 GB`.
fn sized(bytes: u64) -> String {
    let gib = bytes as f64 / GIB;
    if gib >= 1. {
        format!("{gib:.1} GB")
    } else {
        format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
