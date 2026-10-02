use std::{cell::Cell, rc::Rc};

use gpui_kit::{Context, Hsla, IntoElement, Render, Styled, TestAppContext, Window, div};

use super::*;
use crate::theme::{ActiveTheme, StatusTone, contrast, mix, set_theme};
use crate::theme::{MARK_CONTRAST, TEXT_CONTRAST};

#[test]
fn every_theme_loads_in_its_family() {
    let names: Vec<&str> = all().iter().map(|t| t.name.as_ref()).collect();
    for name in ["atelier Light", "atelier Dark", "GitHub Light", "GitHub Dark", "Catppuccin Latte", "Catppuccin Frappé", "Catppuccin Macchiato", "Catppuccin Mocha"] {
        assert!(names.contains(&name), "{name} is offered: {names:?}");
    }
    assert!(names.contains(&"Cursor Dark") && names.contains(&"Cursor Light"));
    let families: Vec<&str> = families().iter().map(|(f, _)| *f).collect();
    assert_eq!(families[0], "atelier", "the default first");
    assert_eq!(named("catppuccin mocha").map(|t| t.name.as_ref()), Some("Catppuccin Mocha"), "by name, case aside");
}

#[test]
fn every_theme_meets_wcag_aa_for_text_and_marks() {
    let mut failures = Vec::new();
    for t in all() {
        let mut check = |what: &str, fg: Hsla, bg: Hsla, least: f32| {
            let ratio = contrast(fg, bg);
            if ratio < least {
                failures.push(format!("{}: {what} {ratio:.2} < {least}", t.name));
            }
        };
        for (surface, bg) in [("page", t.background), ("card", t.card), ("card_strong", t.card_strong)] {
            check(&format!("text on {surface}"), t.foreground, bg, TEXT_CONTRAST);
        }
        for (surface, bg) in [("page", t.background), ("card", t.card)] {
            check(&format!("muted on {surface}"), t.muted_foreground, bg, TEXT_CONTRAST);
        }
        for (name, colour) in [("green", t.success), ("red", t.danger), ("yellow", t.warning), ("blue", t.info)] {
            check(&format!("{name} text on page"), colour, t.background, TEXT_CONTRAST);
        }
        for (name, wash) in [("added", t.diff_added), ("removed", t.diff_removed)] {
            let band = mix(t.background, Hsla { a: 1., ..wash }, wash.a);
            check(&format!("text on the {name} wash"), t.foreground, band, TEXT_CONTRAST);
        }
        for tone in [StatusTone::Running, StatusTone::Done, StatusTone::Failed, StatusTone::Pending, StatusTone::Cancelled] {
            check(&format!("{tone:?} mark"), t.status_tone(tone), t.background, MARK_CONTRAST);
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn a_faint_mark_is_quieter_than_muted_and_still_seen_on_every_surface() {
    for t in all() {
        let faint = t.faint();
        for (surface, bg) in [
            ("page", t.background),
            ("card", t.card),
            ("card_strong", t.card_strong),
        ] {
            let ratio = contrast(faint, bg);
            assert!(
                ratio >= MARK_CONTRAST,
                "{}: faint on {surface} {ratio:.2} < {MARK_CONTRAST}",
                t.name
            );
        }
        assert!(
            contrast(faint, t.card) <= contrast(t.muted_foreground, t.card),
            "{}: faint is no louder than muted",
            t.name
        );
    }
}

#[test]
fn a_box_inside_a_card_parts_from_it() {
    for t in all() {
        let ratio = contrast(t.card_strong, t.card);
        assert!(ratio >= 1.05, "{}: card_strong on card {ratio:.3}", t.name);
    }
}

struct Probe {
    seen: Rc<Cell<Option<Hsla>>>,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let card = cx.theme().card;
        self.seen.set(Some(card));
        div().size_full().bg(card)
    }
}

#[gpui_kit::test]
fn a_switch_repaints_an_open_element_in_the_new_theme(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_theme(atelier(crate::theme::Appearance::Dark).clone(), cx);
    });
    let seen = Rc::new(Cell::new(None));
    let probe_seen = seen.clone();
    let (_view, cx) = cx.add_window_view(move |_, _| Probe { seen: probe_seen });
    cx.run_until_parked();
    assert_eq!(seen.get(), Some(atelier(crate::theme::Appearance::Dark).card));
    let mocha = named("Catppuccin Mocha").unwrap().clone();
    cx.update(|_, cx| set_theme(mocha.clone(), cx));
    cx.run_until_parked();
    assert_eq!(seen.get(), Some(mocha.card), "the open view painted the new theme's card");
    let editor_syntax = cx.update(|_, cx| gpui_kit::component::Theme::global(cx).highlight_theme.clone());
    assert!(std::sync::Arc::ptr_eq(&editor_syntax, &mocha.syntax), "and the code editor's colours are the new theme's");
}

#[test]
fn the_warning_tones_are_ambers_that_read_and_a_fill_that_shows() {
    use crate::theme::{contrast, TEXT_CONTRAST};
    let hue_of = |c: Hsla| c.h * 360.;
    for name in ["atelier Light", "atelier Dark"] {
        let t = named(name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(t.warning_fill, Hsla::from(gpui_kit::Rgba { r: 249. / 255., g: 168. / 255., b: 37. / 255., a: 1. }), "{name}: the fill is atelier's amber");
        assert!((30.0..48.0).contains(&hue_of(t.warning)), "{name}: the text tone is an amber, not a brown grey: hue {:.0}", hue_of(t.warning));
        assert!(t.warning.s > 0.6, "{name}: and saturated: {:.2}", t.warning.s);
        for surface in [t.background, t.card, t.card_strong] {
            assert!(contrast(t.warning, surface) >= TEXT_CONTRAST, "{name}: the text tone reads on every surface");
        }
        // A badge of the fill takes the ink or the page for its words at 4.5:1.
        let best = contrast(t.foreground, t.warning_fill).max(contrast(t.background, t.warning_fill));
        assert!(best >= TEXT_CONTRAST, "{name}: words on the fill: {best:.1}");
    }
}

#[test]
fn a_status_colour_raised_for_contrast_keeps_its_hue() {
    use crate::theme::{contrast, raise, TEXT_CONTRAST};
    let cream = Hsla { h: 0.12, s: 0.3, l: 0.93, a: 1. };
    let ink = Hsla { h: 0.62, s: 0.2, l: 0.08, a: 1. };
    let amber = Hsla { h: 0.103, s: 0.95, l: 0.56, a: 1. };
    let raised = raise(amber, ink, &[cream], TEXT_CONTRAST).expect("it did not read");
    assert!(contrast(raised, cream) >= TEXT_CONTRAST);
    assert_eq!((raised.h, raised.s), (amber.h, amber.s), "hue and saturation held, only lightness moved");
    assert!(raised.l < amber.l);
    let dark_page = Hsla { h: 0.62, s: 0.2, l: 0.08, a: 1. };
    let light_ink = Hsla { h: 0., s: 0., l: 0.95, a: 1. };
    let dim_amber = Hsla { h: 0.103, s: 0.9, l: 0.12, a: 1. };
    let lifted = raise(dim_amber, light_ink, &[dark_page], TEXT_CONTRAST).expect("it did not read");
    assert!(lifted.l > dim_amber.l && lifted.h == dim_amber.h, "on a dark page it moves lighter");
    let grey = Hsla { h: 0., s: 0.05, l: 0.7, a: 1. };
    let moved = raise(grey, ink, &[cream], TEXT_CONTRAST).expect("it did not read");
    assert!(contrast(moved, cream) >= TEXT_CONTRAST, "a grey moves toward the ink");
    assert!(raise(ink, ink, &[cream], TEXT_CONTRAST).is_none());
}

#[test]
#[ignore = "prints the tones of every theme"]
fn print_tones() {
    use crate::theme::contrast;
    for t in crate::themes::all() {
        let hex = |c: gpui_kit::Hsla| { let r = c.to_rgb(); format!("#{:02X}{:02X}{:02X}", (r.r * 255.).round() as u8, (r.g * 255.).round() as u8, (r.b * 255.).round() as u8) };
        println!("TONES {:28} page {} card {} warn {} ({:.1}/{:.1}) accent {} succ {} dang {} info {}", t.name, hex(t.background), hex(t.card), hex(t.warning), contrast(t.warning, t.background), contrast(t.warning, t.card_strong), hex(t.accent), hex(t.success), hex(t.danger), hex(t.info));
    }
}

#[test]
#[ignore = "writes the swatch sheet of the status tones to /tmp/tones.svg"]
fn write_tone_sheet() {
    let hex = |c: gpui_kit::Hsla| {
        let r = c.to_rgb();
        format!("#{:02X}{:02X}{:02X}", (r.r * 255.).round() as u8, (r.g * 255.).round() as u8, (r.b * 255.).round() as u8)
    };
    let mut svg = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1500\" height=\"1180\" font-family=\"DejaVu Sans, sans-serif\" font-size=\"11\"><rect width=\"1500\" height=\"1180\" fill=\"#fff\"/>");
    for (i, t) in crate::themes::all().iter().enumerate() {
        let (x, y) = (10 + (i % 2) * 745, 10 + (i / 2) * 230);
        svg += &format!("<g transform=\"translate({x},{y})\"><text x=\"0\" y=\"12\" font-weight=\"bold\" font-size=\"13\">{}</text>", t.name);
        for (row, (surface, bg)) in [("page", t.background), ("card", t.card)].into_iter().enumerate() {
            let ry = 22 + row * 100;
            svg += &format!("<rect x=\"0\" y=\"{ry}\" width=\"725\" height=\"96\" rx=\"8\" fill=\"{}\"/><text x=\"8\" y=\"{}\" fill=\"{}\">on the {surface}</text>", hex(bg), ry + 14, hex(t.muted_foreground));
            for (col, (name, tone)) in [("warning", t.warning), ("warning fill", t.warning_fill), ("success", t.success), ("danger", t.danger), ("info", t.info)].into_iter().enumerate() {
                let cx = 8 + col * 142;
                svg += &format!("<rect x=\"{cx}\" y=\"{}\" width=\"36\" height=\"36\" rx=\"18\" fill=\"{}\"/><text x=\"{}\" y=\"{}\" fill=\"{}\" font-weight=\"bold\" font-size=\"13\">{name}</text><text x=\"{}\" y=\"{}\" fill=\"{}\">{} {:.1}:1</text><text x=\"{cx}\" y=\"{}\" fill=\"{}\">Needs approval</text>", ry + 24, hex(tone), cx + 44, ry + 40, hex(tone), cx + 44, ry + 54, hex(t.muted_foreground), hex(tone), crate::theme::contrast(tone, bg), ry + 82, hex(tone));
            }
        }
        svg += "</g>";
    }
    svg += "</svg>";
    std::fs::write("/tmp/tones.svg", svg).unwrap();
}

#[test]
fn a_theme_saved_under_the_old_name_finds_atelier_s() {
    assert_eq!(named("lathe Dark").map(|t| t.name.as_ref()), Some("atelier Dark"));
    assert_eq!(named("lathe light").map(|t| t.name.as_ref()), Some("atelier Light"));
    assert!(named("lathe").is_none());
}
