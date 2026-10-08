use gpui_kit::{Rgba, rgb};

use super::*;

#[test]
fn mixing_from_clear_keeps_the_target_color() {
    let white: Hsla = rgb(0xFFFFFF).into();
    let half = mix(transparent_black(), white, 0.5).to_rgb();
    assert!((half.r - 1.).abs() < 1e-3 && (half.g - 1.).abs() < 1e-3 && (half.b - 1.).abs() < 1e-3);
    assert!((half.a - 0.5).abs() < 1e-3);
}

#[test]
fn mixing_opaque_colors_is_a_plain_blend() {
    let black: Hsla = rgb(0x000000).into();
    let white: Hsla = rgb(0xFFFFFF).into();
    let Rgba { r, a, .. } = mix(black, white, 0.25).to_rgb();
    assert!((r - 0.25).abs() < 1e-3 && (a - 1.).abs() < 1e-3);
}

#[test]
fn added_and_removed_lines_get_different_colors() {
    let theme = Theme::light();
    assert_ne!(theme.diff_line(true), theme.diff_line(false));
    assert!(theme.diff_line(true).a < 0.2);
}

#[test]
fn the_system_appearance_picks_light_or_dark() {
    use gpui_kit::WindowAppearance;
    assert_eq!(Appearance::of_system(WindowAppearance::Dark), Appearance::Dark);
    assert_eq!(Appearance::of_system(WindowAppearance::VibrantDark), Appearance::Dark);
    assert_eq!(Appearance::of_system(WindowAppearance::Light), Appearance::Light);
    assert_eq!(Appearance::of_system(WindowAppearance::VibrantLight), Appearance::Light);
}

#[test]
fn atelier_dark_holds_its_ramp_with_the_cancelled_mark_raised_just_to_3_to_1() {
    let tones: Vec<Hsla> = [0xB0B8B8, 0x9CA8A8, 0x848478, 0x6C7878].map(|c| rgb(c).into()).to_vec();
    let theme = Theme::dark();
    for (got, want) in theme.status.iter().zip(tones) {
        let (g, w) = (got.to_rgb(), want.to_rgb());
        assert!((g.r - w.r).abs() < 1e-3 && (g.g - w.g).abs() < 1e-3 && (g.b - w.b).abs() < 1e-3, "the four that passed are the ramp's");
    }
    let ratio = contrast(theme.status_tone(StatusTone::Cancelled), theme.background);
    assert!((3.0..3.4).contains(&ratio), "cancelled: {ratio:.2}, the least change that reaches 3:1");
}

#[test]
fn atelier_light_raises_its_pending_and_cancelled_marks_just_to_3_to_1() {
    let theme = Theme::light();
    for tone in [StatusTone::Pending, StatusTone::Cancelled] {
        let ratio = contrast(theme.status_tone(tone), theme.background);
        assert!((3.0..3.4).contains(&ratio), "{tone:?}: {ratio:.2}, the least change that reaches 3:1");
    }
}

#[test]
fn a_status_mark_is_most_present_when_running() {
    // The mapping is by weight, so each theme reads its own end of the ramp.
    for theme in [Theme::dark(), Theme::light()] {
        let running = theme.status_tone(StatusTone::Running);
        let cancelled = theme.status_tone(StatusTone::Cancelled);
        let page = theme.background;
        assert!(
            contrast(running, page) > contrast(cancelled, page),
            "{:?}: running must stand out more than cancelled",
            theme.appearance
        );
    }
}

#[test]
fn every_status_mark_stays_visible_in_both_themes() {
    let tones =
        [StatusTone::Running, StatusTone::Done, StatusTone::Failed, StatusTone::Pending, StatusTone::Cancelled];
    for theme in [Theme::dark(), Theme::light()] {
        for tone in tones {
            let ratio = contrast(theme.status_tone(tone), theme.background);
            // Cancelled is meant to be barely there; the rest must read at a glance.
            let floor = if tone == StatusTone::Cancelled { 1.9 } else { 2.3 };
            assert!(ratio >= floor, "{:?} {tone:?} contrast {ratio:.2}", theme.appearance);
        }
    }
}

#[test]
fn a_running_mark_reads_at_a_glance_in_both_themes() {
    for theme in [Theme::dark(), Theme::light()] {
        let ratio = contrast(theme.status_tone(StatusTone::Running), theme.background);
        assert!(ratio >= 4., "{:?}: running contrast {ratio:.2}", theme.appearance);
    }
}

#[test]
fn the_semantic_colors_are_muted_toward_the_ramp() {
    for theme in [Theme::dark(), Theme::light()] {
        // Warning is exempt: atelier's warning is its amber (Alex, 2026-09-30), the one loud tone, like the accent.
        for color in [theme.success, theme.danger, theme.info] {
            let hsla = color;
            assert!(hsla.s <= 0.45, "{:?}: saturation {} is still loud", theme.appearance, hsla.s);
        }
        // Muted is not grey: the hues must still tell each other apart.
        assert_ne!(theme.success.h, theme.danger.h);
    }
}

mod pick {
    use gpui_kit::{Hsla, Rgba};

    use crate::{
        theme::{FILL_TEXT_CONTRAST, MARK_CONTRAST, Theme, can_be_primary, cap_patch, contrast, mark_on, text_on, with_pick},
        themes,
    };

    fn rgb(r: u8, g: u8, b: u8) -> Hsla {
        Rgba { r: r as f32 / 255., g: g as f32 / 255., b: b as f32 / 255., a: 1. }.into()
    }

    /// The web demo's blue and the demo's other accents.
    fn accents() -> Vec<Hsla> {
        [(2, 133, 247), (52, 120, 246), (146, 112, 232), (230, 106, 164), (229, 86, 86), (237, 145, 65), (229, 182, 60), (101, 166, 90), (22, 157, 131)]
            .into_iter()
            .map(|(r, g, b)| rgb(r, g, b))
            .collect()
    }

    #[test]
    fn no_pick_leaves_every_theme_as_it_is_and_the_primary_is_the_ink() {
        for theme in themes::all() {
            let same = with_pick(theme, None);
            assert_eq!((same.primary, same.primary_foreground, same.accent, same.selection), (theme.primary, theme.primary_foreground, theme.accent, theme.selection), "{}", theme.name);
        }
        for theme in [Theme::light(), Theme::dark()] {
            assert_eq!((theme.primary, theme.primary_foreground), (theme.foreground, theme.background), "the default primary is the page inverted");
        }
    }

    #[test]
    fn a_pick_becomes_the_primary_fill_in_every_theme_with_readable_text() {
        for theme in themes::all() {
            for pick in accents() {
                let shown = with_pick(theme, Some(pick));
                if can_be_primary(theme, pick) {
                    assert_eq!(shown.primary, pick, "{}: the fill is the pick", theme.name);
                    let text = shown.primary_foreground;
                    assert!(text == theme.background || text == theme.foreground, "{}: the text is the page or the ink", theme.name);
                    assert!(contrast(text, pick) >= FILL_TEXT_CONTRAST, "{}: the text reads on it", theme.name);
                } else {
                    assert_eq!(shown.primary, theme.primary, "{}: a pick no text can be read on is not taken", theme.name);
                }
            }
        }
    }

    #[test]
    fn the_pick_is_also_the_accent_unless_it_fails_three_to_one_against_the_page() {
        for theme in themes::all() {
            for pick in accents() {
                let shown = with_pick(theme, Some(pick));
                if !can_be_primary(theme, pick) {
                    continue;
                }
                if contrast(pick, theme.background) >= MARK_CONTRAST {
                    assert_eq!(shown.accent, pick, "{}", theme.name);
                    assert_eq!(shown.selection.h, pick.h, "{}: the selection wash takes its hue", theme.name);
                } else {
                    assert_eq!(shown.accent, theme.accent, "{}: it fails 3:1, so the theme's accent stays", theme.name);
                }
            }
        }
    }

    #[test]
    fn the_text_is_the_lighter_tone_at_three_to_one_else_the_darker_else_none() {
        let theme = Theme::light();
        let (page, ink) = (theme.background, theme.foreground);
        let dark = rgb(20, 30, 90);
        let blue = rgb(2, 133, 247);
        let yellow = rgb(229, 182, 60);
        assert_eq!(text_on(&theme, dark), Some(page), "light text on a dark fill");
        assert!(contrast(page, blue) >= FILL_TEXT_CONTRAST && contrast(page, blue) < 4.5);
        assert_eq!(text_on(&theme, blue), Some(page), "white on the web's blue, though it is under 4.5:1");
        assert!(contrast(page, yellow) < FILL_TEXT_CONTRAST);
        assert_eq!(text_on(&theme, yellow), Some(ink), "the darker tone where the lighter one fails 3:1");
        // Dark theme: the ink is the lighter tone.
        let night = Theme::dark();
        assert_eq!(text_on(&night, dark), Some(night.foreground), "light text is the ink in a dark theme");
        assert_eq!(text_on(&night, yellow), Some(night.background));
        // A fill neither tone reaches 3:1 on is refused, and so is not a primary.
        let mid = rgb(130, 130, 130);
        let none = contrast(page, mid) < FILL_TEXT_CONTRAST && contrast(ink, mid) < FILL_TEXT_CONTRAST;
        assert_eq!(text_on(&theme, mid).is_none(), none);
        assert_eq!(can_be_primary(&theme, mid), !none);
    }

    #[test]
    fn a_mark_is_the_same_tone_as_the_words() {
        let theme = Theme::light();
        for fill in accents().into_iter().chain([theme.foreground, rgb(250, 240, 200)]) {
            assert_eq!(Some(mark_on(&theme, fill)), text_on(&theme, fill).or(Some(theme.foreground)));
        }
        assert_eq!(mark_on(&theme, theme.foreground), theme.background, "on the default fill it is the page");
        assert!(contrast(theme.background, rgb(2, 133, 247)) >= MARK_CONTRAST);
    }

    #[test]
    fn the_key_cap_on_a_primary_fill_keeps_three_to_one_in_every_theme_with_and_without_a_pick() {
        for theme in themes::all() {
            let mut fills: Vec<Option<Hsla>> = vec![None];
            fills.extend(accents().into_iter().map(Some));
            for pick in fills {
                let shown = with_pick(theme, pick);
                let (fill, text) = (shown.primary, shown.primary_foreground);
                let patch = cap_patch(fill, text);
                let name = format!("{} {:?}", theme.name, pick.map(|p| p.to_rgb()));
                assert!(contrast(text, patch) >= FILL_TEXT_CONTRAST, "{name}: the cap text reads on its patch: {:.2}", contrast(text, patch));
                // A tone of the fill: it lies between the fill and the text, never past the wash.
                assert!(contrast(fill, patch) <= contrast(fill, text) + 0.01, "{name}: the patch is not further from the fill than the text");
                assert_ne!(patch, text, "{name}");
            }
        }
    }

    #[test]
    fn the_cap_patch_moves_off_the_fill_where_there_is_room_and_stays_on_it_where_there_is_none() {
        let theme = Theme::light();
        let room = cap_patch(theme.foreground, theme.background);
        assert_ne!(room, theme.foreground, "black ink has room for a patch");
        let text = theme.background;
        // No room at all: a fill that only just reaches 3:1 gets a patch equal to the fill.
        let tight = (0..255u8).map(|g| rgb(g, g, g)).find(|c| contrast(text, *c) >= FILL_TEXT_CONTRAST && contrast(text, *c) < FILL_TEXT_CONTRAST + 0.03);
        if let Some(tight) = tight {
            assert_eq!(cap_patch(tight, text), tight);
        }
    }
}

/// A floating dropdown or menu panel has a subtle 1px edge: seen, but well under the ink.
#[test]
fn the_dropdown_edge_is_subtle_but_there_in_every_theme() {
    use crate::theme::contrast;
    for theme in crate::themes::all() {
        let edge = dropdown_edge(theme);
        let c = contrast(edge, theme.popover);
        assert!(c > 1.05 && c < 2.0, "{}: {c:.2}", theme.name);
    }
}
