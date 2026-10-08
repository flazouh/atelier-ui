use super::*;

#[gpui_kit::test]
fn the_default_is_a_for_the_tabs_and_a_choice_applies_at_once(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        assert_eq!(tabs(cx), 0);
        init(Some(1), cx);
        assert_eq!(tabs(cx), 1);
        set_tabs(3, cx);
        assert_eq!(tabs(cx), 3);
        init(Some(9), cx);
        assert_eq!(tabs(cx), 3, "a stranger is held to the last design");
    });
}

#[test]
fn each_design_has_a_word_and_the_editor_designs_map_to_the_tab_variants() {
    assert_eq!(TABS_DESIGNS.len(), 4);
    assert!((0..4).all(|i| tab_variant(i).is_editor()));
    assert_ne!(tab_variant(0), tab_variant(1));
}

/// design preview: remove after Alex picks. The elevation is C until a choice says otherwise, and a stranger is held.
#[test]
fn the_elevation_defaults_to_c_and_a_choice_applies_at_once() {
    assert_eq!(elevation(), 2);
    set_elevation(0);
    assert_eq!(elevation(), 0);
    set_elevation(9);
    assert_eq!(elevation(), 3);
    init_elevation(None);
    assert_eq!(elevation(), 2);
    init_elevation(Some(1));
    assert_eq!(elevation(), 1);
    init_elevation(None);
}

#[test]
fn a_border_keeps_the_old_panel_and_the_tones_drop_the_border() {
    set_strength(100);
    for theme in crate::themes::all() {
        let base = vec![gpui_kit::BoxShadow {
            color: theme.shadow,
            offset: gpui_kit::point(gpui_kit::px(0.), gpui_kit::px(4.)),
            blur_radius: gpui_kit::px(6.),
            spread_radius: gpui_kit::px(0.),
            inset: false,
        }];
        assert_eq!(panel_fill(theme, 0, theme.popover), theme.popover, "{}: A keeps the fill", theme.name);
        assert!(panel_edge(theme, 0).a > 0., "{}: A has an edge", theme.name);
        assert_eq!(panel_shadows(theme, 0, base.clone()).len(), 1, "{}: A keeps the shadow it had", theme.name);
        for design in 1..4 {
            assert_eq!(panel_edge(theme, design).a, 0., "{} {design}: no border", theme.name);
            let (raised, rest) = (panel_fill(theme, design, theme.popover), theme.popover);
            let step = raised.l - rest.l;
            assert!(step > 0., "{} {design}: one step lighter ({} vs {})", theme.name, raised.l, rest.l);
        }
        assert!(panel_shadows(theme, 1, base.clone()).is_empty(), "{}: B has no shadow", theme.name);
        let c = panel_shadows(theme, 2, base.clone());
        assert_eq!(c.len(), 2, "{}: C is a contact and a soft shadow", theme.name);
        assert_eq!((f32::from(c[0].offset.y), f32::from(c[0].blur_radius)), (1., 2.));
        assert_eq!((f32::from(c[1].offset.y), f32::from(c[1].blur_radius)), (8., 24.));
        assert!(c.iter().all(|s| !s.inset));
        let d = panel_shadows(theme, 3, base.clone());
        assert_eq!(d.len(), 3, "{}: D adds the top light", theme.name);
        assert!(d[2].inset && (d[2].color.a - 0.06).abs() < 1e-4 && f32::from(d[2].offset.y) == 1.);
        assert!(panel_shadows(theme, 2, Vec::new()).is_empty(), "{}: a panel with no shadow stays without", theme.name);
    }
}

#[test]
fn the_shadow_is_stronger_in_dark_than_in_light() {
    set_strength(100);
    let themes = crate::themes::all();
    let light = themes.iter().find(|t| t.appearance == crate::theme::Appearance::Light).unwrap();
    let dark = themes.iter().find(|t| t.appearance == crate::theme::Appearance::Dark).unwrap();
    let alpha = |t: &crate::Theme| panel_shadows(t, 2, vec![panel_shadows(t, 1, Vec::new()).into_iter().next().unwrap_or(gpui_kit::BoxShadow { color: t.shadow, offset: Default::default(), blur_radius: Default::default(), spread_radius: Default::default(), inset: false })]);
    let (l, d) = (alpha(light), alpha(dark));
    assert!((l[0].color.a - 0.12).abs() < 1e-4 && (l[1].color.a - 0.10).abs() < 1e-4);
    assert!((d[0].color.a - 0.40).abs() < 1e-4 && (d[1].color.a - 0.35).abs() < 1e-4);
}

#[test]
fn the_strength_starts_at_50_and_a_choice_applies_at_once() {
    assert_eq!(strength(), 50);
    set_strength(80);
    assert_eq!(strength(), 80);
    set_strength(400);
    assert_eq!(strength(), 100);
    init_strength(None);
    assert_eq!(strength(), 50);
}

/// The default is a soft lift: 25% toward white in light, 3% in dark, and half C's shadow.
#[test]
fn the_default_strength_is_soft() {
    let themes = crate::themes::all();
    let light = themes.iter().find(|t| t.appearance == crate::theme::Appearance::Light).unwrap();
    let dark = themes.iter().find(|t| t.appearance == crate::theme::Appearance::Dark).unwrap();
    let white = gpui_kit::hsla(0., 0., 1., 1.);
    let near = |a: f32, b: f32| (a - b).abs() < 1e-4;
    let lifted = panel_fill(light, 2, light.popover);
    let want = crate::theme::mix(light.popover, white, 0.25);
    assert!(near(lifted.l, want.l), "{} vs {}", lifted.l, want.l);
    let lifted = panel_fill(dark, 2, dark.popover);
    let want = crate::theme::mix(dark.popover, white, 0.03);
    assert!(near(lifted.l, want.l));
    let base = vec![gpui_kit::BoxShadow { color: dark.shadow, offset: Default::default(), blur_radius: Default::default(), spread_radius: Default::default(), inset: false }];
    let c = panel_shadows(dark, 2, base);
    assert!(near(c[0].color.a, 0.20) && near(c[1].color.a, 0.175));
}

#[test]
fn a_strength_of_zero_is_no_lift_and_no_shadow() {
    set_strength(0);
    for theme in crate::themes::all() {
        let lifted = panel_fill(theme, 2, theme.popover);
        assert!((lifted.l - theme.popover.l).abs() < 1e-4, "{}", theme.name);
        let base = vec![gpui_kit::BoxShadow { color: theme.shadow, offset: Default::default(), blur_radius: Default::default(), spread_radius: Default::default(), inset: false }];
        assert!(panel_shadows(theme, 2, base).iter().all(|s| s.color.a == 0.), "{}", theme.name);
    }
    set_strength(50);
}

/// The row pill comes from the panel's own fill, one step toward the ink, and keeps at least the contrast the
/// page-level pill (`card_strong` on `card`) had, in every theme, at every lift and every design.
#[test]
fn the_row_pill_keeps_its_contrast_at_every_lift() {
    for theme in crate::themes::all() {
        let today = crate::theme::contrast(theme.card_strong, theme.card);
        for strength in [0, 25, 50, 75, 100] {
            set_strength(strength);
            for design in 0..4 {
                let panel = panel_fill(theme, design, theme.popover);
                let pill = row_tone(theme, panel);
                let now = crate::theme::contrast(pill, panel);
                assert!(now >= today - 1e-3, "{} s{strength} d{design}: {now:.4} vs {today:.4}", theme.name);
            }
        }
    }
    set_strength(50);
}
