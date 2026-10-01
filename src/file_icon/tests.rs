use super::*;

// acepe's `extension-map.test.ts`, ported.

#[test]
fn resolves_language_specific_icons_from_the_file_type_pack() {
    assert_eq!(file_icon("main.ts"), "file-icons/typescript.svg");
    assert_eq!(file_icon("App.tsx"), "file-icons/react_ts.svg");
    assert_eq!(file_icon("Widget.svelte"), "file-icons/svelte.svg");
    assert_eq!(file_icon("lib.rs"), "file-icons/rust.svg");
    assert_eq!(icon_for_extension("ts"), "typescript");
    assert_eq!(icon_for_extension("svelte"), "svelte");
}

#[test]
fn matches_special_filenames_from_a_full_path_basename() {
    assert_eq!(icon_for_name("packages/ui/package.json"), Some("npm"));
    assert_eq!(file_icon("packages/ui/package.json"), "file-icons/npm.svg");
}

#[test]
fn keeps_the_fallback_on_the_same_pack() {
    assert_eq!(file_icon("unknown.zzzz"), "file-icons/file.svg");
    assert_eq!(file_icon("Makefile.nothing-here"), "file-icons/file.svg");
}

#[test]
fn strips_line_and_column_suffixes_before_resolving() {
    assert_eq!(file_icon("tests.rs:914"), "file-icons/rust.svg");
    assert_eq!(file_icon("vitest.config.ts:10"), "file-icons/vitest.svg");
    assert_eq!(file_icon("src/app.ts:12:4"), "file-icons/typescript.svg");
    assert_eq!(file_icon("packages/ui/package.json:3"), "file-icons/npm.svg");
}

// atelier's own.

#[test]
fn a_declaration_file_is_not_plain_typescript() {
    assert_eq!(file_icon("types/index.d.ts"), "file-icons/typescript-def.svg");
}

#[test]
fn extensions_and_names_match_without_regard_to_case() {
    assert_eq!(file_icon("MAIN.RS"), "file-icons/rust.svg");
    assert_eq!(file_icon("Package.JSON"), "file-icons/npm.svg");
}

#[test]
fn folders_are_open_or_closed_and_some_have_their_own() {
    assert_eq!(folder_icon("crates", false), "file-icons/folder.svg");
    assert_eq!(folder_icon("crates", true), "file-icons/folder-open.svg");
    assert_eq!(folder_icon("src", false), "file-icons/folder-src.svg");
    assert_eq!(folder_icon("Tests", true), "file-icons/folder-test-open.svg");
    // A merged folder row ("crates/beui/src") takes its last folder's icon.
    assert_eq!(folder_icon("crates/beui/src", false), "file-icons/folder-src.svg");
}

#[test]
fn every_icon_the_map_names_is_embedded() {
    for path in [file_icon("a.rs"), file_icon("a.zzz"), folder_icon("src", true), folder_icon("x", false), file_icon("package.json")] {
        assert!(bytes(&path).is_some(), "{path}");
    }
    assert!(bytes("file-icons/no-such.svg").is_none());
}

mod source {
    use gpui_kit::{Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use crate::file_icon::{FileIcon, IconFor, set_source};

    struct Host;

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(FileIcon::file("src/lib.rs")).child(FileIcon::folder("src", true))
        }
    }

    fn mark(name: &'static str) -> gpui_kit::AnyElement {
        div().debug_selector(move || name.into()).size(px(1.)).into_any_element()
    }

    #[gpui_kit::test]
    fn an_app_can_draw_file_icons_its_own_way_and_keep_the_folders(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_source(
                |icon, _, _| match icon {
                    IconFor::File(path) if *path == "src/lib.rs" => Some(mark("own-file-icon")),
                    _ => None,
                },
                cx,
            );
        });
        let (_host, cx) = cx.add_window_view(|_, _| Host);
        cx.simulate_resize(size(px(100.), px(100.)));
        cx.run_until_parked();
        let own = cx.debug_bounds("own-file-icon").expect("the app's icon draws for the file");
        assert_eq!(f32::from(own.size.width), 1.);
        assert!(cx.debug_bounds("file-icon").is_some(), "the folder keeps the built-in icon");
    }
}
