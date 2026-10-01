# atelier-ui

A design system for [GPUI](https://www.gpui.rs) apps, in Rust. It is the look and the motion of the
[Atelier](https://github.com/flazouh/atelier) code editor: tokens, themes, springs, and more than a hundred
components, from buttons and menus to diff views, a code editor, and agent panels.

Tokens and motion follow [beui.dev](https://beui.dev). The type is Geist and Geist Mono, both embedded.

## Use it

```toml
[dependencies]
atelier-ui = { git = "https://github.com/flazouh/atelier-ui" }

[patch.crates-io]
# The two patched copies in this repo's vendor folder. Cargo applies patches only from the top manifest.
gpui-base = { git = "https://github.com/flazouh/atelier-ui" }
gpui-component = { git = "https://github.com/flazouh/atelier-ui" }
```

```rust
use atelier_ui::{Button, ButtonVariant};

fn init(cx: &mut gpui_kit::App) {
    gpui_kit::init(cx);
    atelier_ui::init(cx);
}
```

Components take plain data. None of them knows about an agent, a code host, or a file system: the app reads
those and hands the result over.

## What is in it

| Part | Modules |
| --- | --- |
| Tokens | `theme`, `typography`, `motion`, `themes` (atelier, GitHub, Catppuccin, Cursor, and VS Code theme import) |
| Controls | `button`, `button_group`, `select`, `multi_select`, `combobox`, `checkbox`, `range_slider`, `text_input`, `menu`, `popover`, `tabs`, `breadcrumb` |
| Code | `code_editor`, `code_block`, `file_diff`, `inline_review`, `changed_files`, `file_tree`, `file_icon`, `syntax` |
| Agent panels | `agent_text`, `tool_call`, `tool_approval`, `todo_list`, `thinking`, `prompt_input`, `subagent_card` |
| Work | `task_board`, `task_picker`, `pr`, `pr_chip`, `merge_button`, `checks_panel`, `review_bar` |

See [docs/themes.md](docs/themes.md) for the themes and how a VS Code theme maps to the tokens.

## Checks

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```

## License

atelier-ui is dual licensed:

- [GPL-3.0](LICENSE) for free use, in apps that are GPL-3.0 compatible.
- A [commercial license](LICENSE-COMMERCIAL.md) for apps that do not want the GPL's terms.

Third-party parts keep their own licenses: see [NOTICE](NOTICE).
