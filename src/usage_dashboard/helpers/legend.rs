use gpui_kit::SharedString;

use super::super::structs::{LegendEntry, UsageSource};

/// One entry for each hue the sources use, named by its first group's caption up to the first " · ".
pub fn legend(sources: &[UsageSource]) -> Vec<LegendEntry> {
    let mut out: Vec<LegendEntry> = Vec::new();
    for source in sources {
        match out.iter_mut().find(|entry| entry.hue == source.series.hue) {
            Some(entry) => {
                if !entry.shades.contains(&source.series.shade) {
                    entry.shades.push(source.series.shade);
                }
            }
            None => {
                let name = source.group.split(" · ").next().unwrap_or_default().to_string();
                out.push(LegendEntry {
                    name: SharedString::from(name),
                    hue: source.series.hue,
                    shades: vec![source.series.shade],
                });
            }
        }
    }
    out
}
