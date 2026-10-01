//! What loading the themes costs at start: every bundled file parsed or imported, as `themes::all()`
//! does once, before the first window. Target: under 30 ms for all of them together.
//!     cargo test --release -p beui --test theme_bench -- --ignored --nocapture

use std::{hint::black_box, time::{Duration, Instant}};

const ATELIER: [&str; 2] = [include_str!("../assets/themes/atelier-light.json"), include_str!("../assets/themes/atelier-dark.json")];
const VSCODE: &[&str] = &[
    include_str!("../assets/themes/vscode/github-light.json"),
    include_str!("../assets/themes/vscode/github-dark.json"),
    include_str!("../assets/themes/vscode/catppuccin-latte.json"),
    include_str!("../assets/themes/vscode/catppuccin-frappe.json"),
    include_str!("../assets/themes/vscode/catppuccin-macchiato.json"),
    include_str!("../assets/themes/vscode/catppuccin-mocha.json"),
    include_str!("../assets/themes/vscode/cursor-dark.json"),
    include_str!("../assets/themes/vscode/cursor-light.json"),
];

#[test]
#[ignore]
fn loading_every_theme() {
    let mut samples: Vec<Duration> = (0..20)
        .map(|_| {
            let at = Instant::now();
            for json in ATELIER {
                black_box(atelier_ui::theme_file::parse(json).unwrap());
            }
            for json in VSCODE {
                black_box(atelier_ui::theme_import::import(json, "x", "x").unwrap());
            }
            at.elapsed()
        })
        .collect();
    samples.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1000.;
    println!("all 10 themes loaded: median {:.2} ms, p95 {:.2} ms, target < 30 ms", ms(samples[10]), ms(samples[18]));
}
