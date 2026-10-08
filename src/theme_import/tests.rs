use super::*;

/// Every bundled VS Code theme: its name, file, the tokens it leaves to derive, and the colours moved
/// for contrast. `docs/themes.md` lists the same.
fn bundled() -> Vec<(
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
)> {
    let list: Vec<(&str, &str, &[&str], &[&str])> = vec![
        (
            "GitHub Light",
            include_str!("../../assets/themes/vscode/github-light.json"),
            &[
                "card_strong",
                "divider",
                "warning_fill",
                "popover",
                "shadow",
                "chip_rest",
                "chip_hover",
                "chip_arrow",
                "status",
            ],
            &["muted", "info", "danger", "warning"],
        ),
        (
            "GitHub Dark",
            include_str!("../../assets/themes/vscode/github-dark.json"),
            &[
                "card_strong",
                "divider",
                "warning_fill",
                "shadow",
                "chip_rest",
                "chip_hover",
                "chip_arrow",
                "status",
            ],
            &[],
        ),
        (
            "Catppuccin Latte",
            include_str!("../../assets/themes/vscode/catppuccin-latte.json"),
            &[
                "divider",
                "warning_fill",
                "chip_rest",
                "chip_hover",
                "chip_arrow",
                "status",
            ],
            &["muted", "info", "danger", "success", "warning"],
        ),
        (
            "Catppuccin Frappé",
            include_str!("../../assets/themes/vscode/catppuccin-frappe.json"),
            &[
                "divider",
                "warning_fill",
                "chip_rest",
                "chip_hover",
                "chip_arrow",
                "status",
            ],
            &["muted", "info", "danger", "warning"],
        ),
        (
            "Catppuccin Macchiato",
            include_str!("../../assets/themes/vscode/catppuccin-macchiato.json"),
            &[
                "divider",
                "warning_fill",
                "chip_rest",
                "chip_hover",
                "chip_arrow",
                "status",
            ],
            &["muted"],
        ),
        (
            "Catppuccin Mocha",
            include_str!("../../assets/themes/vscode/catppuccin-mocha.json"),
            &[
                "divider",
                "warning_fill",
                "chip_rest",
                "chip_hover",
                "chip_arrow",
                "status",
            ],
            &["muted"],
        ),
        (
            "Cursor Dark",
            include_str!("../../assets/themes/vscode/cursor-dark.json"),
            &[
                "divider",
                "warning_fill",
                "chip_rest",
                "chip_hover",
                "chip_arrow",
                "status",
            ],
            &["muted"],
        ),
        (
            "Cursor Light",
            include_str!("../../assets/themes/vscode/cursor-light.json"),
            &[
                "card_strong",
                "divider",
                "warning_fill",
                "chip_rest",
                "chip_hover",
                "chip_arrow",
                "status",
            ],
            &["warning"],
        ),
    ];
    list
}

#[test]
fn each_bundled_theme_imports_with_the_tokens_it_derives_listed() {
    for (name, json, derived, raised) in bundled() {
        let imported = import(json, name, "x").unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(imported.derived, derived, "{name}: derived");
        assert_eq!(imported.raised, raised, "{name}: raised for contrast");
        let t = &imported.theme;
        let opaque = [
            ("page", t.background),
            ("ink", t.foreground),
            ("card", t.card),
            ("card_strong", t.card_strong),
            ("muted", t.muted_foreground),
            ("accent", t.accent),
            ("info", t.info),
            ("danger", t.danger),
            ("success", t.success),
            ("warning", t.warning),
            ("popover", t.popover),
        ];
        for (token, colour) in opaque {
            assert!(
                (colour.a - 1.).abs() < 1e-6,
                "{name}: {token} is laid over the page, so opaque"
            );
        }
        assert_eq!(
            (t.primary, t.primary_foreground),
            (t.foreground, t.background),
            "{name}: the page inverted"
        );
    }
}

#[test]
fn a_theme_says_its_own_appearance() {
    let latte = import(
        include_str!("../../assets/themes/vscode/catppuccin-latte.json"),
        "Latte",
        "x",
    )
    .unwrap();
    let mocha = import(
        include_str!("../../assets/themes/vscode/catppuccin-mocha.json"),
        "Mocha",
        "x",
    )
    .unwrap();
    assert_eq!(
        (latte.theme.appearance, mocha.theme.appearance),
        (Appearance::Light, Appearance::Dark)
    );
}

#[test]
fn a_scope_reads_its_most_specific_rule() {
    let rules: Vec<Value> = serde_json::from_str(
        r##"[
            {"scope": "keyword", "settings": {"foreground": "#111111"}},
            {"scope": ["keyword.operator", "x y"], "settings": {"foreground": "#222222"}},
            {"scope": "keyword.operator.word", "settings": {"foreground": "#333333"}}
        ]"##,
    )
    .unwrap();
    let colour = |scope: &str| {
        rule_for(&rules, scope)
            .and_then(|s| s.get("foreground"))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    assert_eq!(colour("keyword.control").as_deref(), Some("#111111"));
    assert_eq!(
        colour("keyword.operator.arithmetic").as_deref(),
        Some("#222222")
    );
    assert_eq!(colour("keyword.operator.word").as_deref(), Some("#333333"));
    assert_eq!(colour("string"), None);
}

#[test]
fn a_colour_that_fails_contrast_moves_toward_the_ink() {
    let page = hex("#FFFFFF").unwrap();
    let ink = hex("#000000").unwrap();
    let pale = hex("#DDDDDD").unwrap();
    let raised = crate::theme::raise(pale, ink, &[page], TEXT_CONTRAST).unwrap();
    assert!(contrast(raised, page) >= TEXT_CONTRAST);
    assert!(
        crate::theme::raise(ink, ink, &[page], TEXT_CONTRAST).is_none(),
        "one that passes stays"
    );
}
