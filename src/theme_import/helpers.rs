use std::sync::Arc;

use gpui_kit::{Hsla, component::highlighter::HighlightTheme};
use serde_json::{Map, Value, json};

use crate::{
    theme::{Appearance, TEXT_CONTRAST, contrast, mix, raise, raise_marks},
    theme_file::{Tokens, appearance, hex},
};
use super::structs::Imported;
use super::types::{SYNTAX_SCOPES, UI_KEYS};

/// The theme a VS Code theme file describes, called `name` in `family`.
pub fn import(json: &str, name: &str, family: &str) -> Result<Imported, String> {
    let file: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let empty = Map::new();
    let colors = file.get("colors").and_then(Value::as_object).unwrap_or(&empty);
    let key = |keys: &[&str], take: &dyn Fn(Hsla) -> bool| {
        keys.iter().find_map(|k| colors.get(*k).and_then(Value::as_str).and_then(hex).filter(|c| take(*c)))
    };
    let keys_of = |token: &str| UI_KEYS.iter().find(|(t, _)| *t == token).map_or(&[][..], |(_, keys)| *keys);
    let ui = |token: &str| key(keys_of(token), &|_| true);
    let kind = file.get("type").and_then(Value::as_str).unwrap_or_default();
    let dark_type = kind == "dark" || kind == "hc-black";
    // The page is opaque; every other UI colour is laid over it, as VS Code paints a translucent one.
    let page = ui("page").map(|p| Hsla { a: 1., ..p });
    let appearance = match (kind, page) {
        ("light" | "dark" | "hc-black" | "hc-light", _) => appearance(if dark_type { "dark" } else { "light" })?,
        (_, Some(page)) => if page.l < 0.5 { Appearance::Dark } else { Appearance::Light },
        _ => return Err("no editor.background and no type".into()),
    };
    let over = |c: Option<Hsla>| c.map(|c| match page {
        Some(p) if c.a < 1. => mix(p, Hsla { a: 1., ..c }, c.a),
        _ => Hsla { a: 1., ..c },
    });
    // A card the same as the page does not part from it: leave it to be derived.
    let apart = |c: Option<Hsla>| over(c).filter(|c| page.is_none_or(|p| contrast(*c, p) > 1.02));
    let ink = over(ui("ink"));
    // Muted text must be quieter than the ink: a theme whose description colour is its text colour
    // gives the next key its turn.
    let quieter = |c: Hsla| {
        let c = over(Some(c)).unwrap_or(c);
        ink.is_none_or(|ink| contrast(c, ink) > 1.3)
    };
    let mut tokens = Tokens {
        page,
        ink,
        card: apart(ui("card")),
        card_strong: None,
        muted: over(key(keys_of("muted"), &quieter)),
        divider: None,
        accent: over(ui("accent")),
        info: over(ui("info")),
        danger: over(ui("danger")),
        success: over(ui("success")),
        warning: over(ui("warning")),
        warning_fill: None,
        selection: ui("selection"),
        popover: apart(ui("popover")),
        shadow: ui("shadow"),
        diff_added: ui("diff_added"),
        diff_removed: ui("diff_removed"),
        chip_rest: None,
        chip_hover: None,
        chip_arrow: None,
        status: None,
        chart: None,
    };
    tokens.card_strong = apart(ui("card_strong")).filter(|c| tokens.card.is_none_or(|card| contrast(*c, card) > 1.02));
    let syntax = syntax(&file, name, appearance, &tokens)?;
    let (mut theme, derived) = tokens.resolve(name.to_string().into(), family.to_string().into(), appearance, Arc::new(syntax));

    // Text must read on the page and on a card; a status mark must show on the page.
    let mut raised = Vec::new();
    let surfaces = [theme.background, theme.card, theme.card_strong];
    let ink = theme.foreground;
    for (label, color, least) in [
        ("ink", &mut theme.foreground, TEXT_CONTRAST),
        ("muted", &mut theme.muted_foreground, TEXT_CONTRAST),
        ("info", &mut theme.info, TEXT_CONTRAST),
        ("danger", &mut theme.danger, TEXT_CONTRAST),
        ("success", &mut theme.success, TEXT_CONTRAST),
        ("warning", &mut theme.warning, TEXT_CONTRAST),
    ] {
        if let Some(moved) = raise(*color, ink, &surfaces, least) {
            *color = moved;
            raised.push(label);
        }
    }
    theme.primary = theme.foreground;
    theme.status = [theme.info, theme.success, theme.danger, theme.warning, theme.muted_foreground];
    raise_marks(&mut theme);
    Ok(Imported { theme, derived, raised })
}

/// A TextMate rule's colour and style for `scope`: the rule whose selector is the longest prefix of
/// it, scope by scope. Selectors with a space (descendant rules) are left out.
pub(super) fn rule_for<'a>(rules: &'a [Value], scope: &str) -> Option<&'a Map<String, Value>> {
    let mut best: Option<(usize, &Map<String, Value>)> = None;
    for rule in rules {
        let Some(settings) = rule.get("settings").and_then(Value::as_object) else { continue };
        let selectors: Vec<&str> = match rule.get("scope") {
            Some(Value::String(s)) => s.split(',').map(str::trim).collect(),
            Some(Value::Array(list)) => list.iter().filter_map(Value::as_str).collect(),
            _ => continue,
        };
        for selector in selectors {
            let matches = !selector.contains(' ') && (scope == selector || scope.starts_with(&format!("{selector}.")));
            if matches && best.is_none_or(|(len, _)| selector.len() > len) {
                best = Some((selector.len(), settings));
            }
        }
    }
    best.map(|(_, settings)| settings)
}

pub(super) fn syntax(file: &Value, name: &str, appearance: Appearance, tokens: &Tokens) -> Result<HighlightTheme, String> {
    let rules = file.get("tokenColors").and_then(Value::as_array).cloned().unwrap_or_default();
    let colors = file.get("colors").and_then(Value::as_object).cloned().unwrap_or_default();
    let color = |k: &str| colors.get(k).and_then(Value::as_str).map(str::to_string);
    let ink = color("editor.foreground").unwrap_or_else(|| "#808080".into());
    let mut syntax = Map::new();
    for (name, scopes) in SYNTAX_SCOPES {
        let found = scopes.iter().find_map(|scope| rule_for(&rules, scope));
        let mut style = Map::new();
        let foreground = found.and_then(|s| s.get("foreground")).and_then(Value::as_str).map(str::to_string);
        style.insert("color".into(), json!(foreground.unwrap_or_else(|| ink.clone())));
        let font = found.and_then(|s| s.get("fontStyle")).and_then(Value::as_str).unwrap_or_default();
        if font.contains("italic") {
            style.insert("font_style".into(), json!("italic"));
        }
        if font.contains("bold") {
            style.insert("font_weight".into(), json!(700));
        }
        syntax.insert((*name).into(), Value::Object(style));
    }
    syntax.insert("emphasis".into(), json!({ "font_style": "italic" }));
    syntax.insert("emphasis.strong".into(), json!({ "font_weight": 600 }));
    let comment = syntax["comment"]["color"].clone();
    syntax.insert("predictive".into(), json!({ "color": comment }));
    syntax.insert("hint".into(), json!({ "color": comment }));
    let hex_of = |c: Option<Hsla>| c.map(|c| {
        let c = c.to_rgb();
        let byte = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
        format!("#{:02X}{:02X}{:02X}{:02X}", byte(c.r), byte(c.g), byte(c.b), byte(c.a))
    });
    let mut style = Map::new();
    let mut put = |k: &str, v: Option<String>| {
        if let Some(v) = v {
            style.insert(k.into(), json!(v));
        }
    };
    put("editor.foreground", Some(ink.clone()));
    put("editor.background", color("editor.background"));
    put("editor.active_line.background", color("editor.lineHighlightBackground"));
    put("editor.line_number", color("editorLineNumber.foreground"));
    put("editor.active_line_number", color("editorLineNumber.activeForeground").or(Some(ink.clone())));
    put("editor.invisible", color("editorWhitespace.foreground"));
    put("error.border", hex_of(tokens.danger));
    put("warning.border", hex_of(tokens.warning));
    put("info.border", hex_of(tokens.info));
    put("created", hex_of(tokens.success));
    put("created.background", hex_of(tokens.success));
    put("deleted.background", hex_of(tokens.danger));
    put("modified", hex_of(tokens.warning));
    style.insert("syntax".into(), Value::Object(syntax));
    let theme = json!({
        "name": name,
        "appearance": if appearance == Appearance::Dark { "dark" } else { "light" },
        "style": Value::Object(style),
    });
    serde_json::from_value(theme).map_err(|e| format!("syntax: {e}"))
}
