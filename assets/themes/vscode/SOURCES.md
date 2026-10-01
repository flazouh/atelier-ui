# VS Code themes, as published

Each file is the theme exactly as its package ships it; beui imports it at start
(`src/theme_import.rs`, mapping in `docs/themes.md`).

| File | Package | Version | Licence |
| --- | --- | --- | --- |
| `github-light.json` | `GitHub.github-vscode-theme` (primer/github-vscode-theme), `themes/light-default.json` | 6.3.5, from open-vsx.org | MIT, `LICENSE-github` |
| `github-dark.json` | the same, `themes/dark-default.json` | 6.3.5 | MIT, `LICENSE-github` |
| `catppuccin-latte.json`, `-frappe`, `-macchiato`, `-mocha` | `@catppuccin/vscode`, `themes/<flavour>.json` | 3.18.1, from npm | MIT, `LICENSE-catppuccin` |
| `cursor-dark.json`, `cursor-light.json` | `cursor-themes` by Ryo Lu (Anysphere), bundled in the Cursor app as `extensions/theme-cursor/themes/cursor-{dark,light}-color-theme.json`; its package names github.com/ryokun6/cursor-themes | 0.0.2, copied unchanged from the app | none found; Alex chose to ship them on 2026-09-29 |
