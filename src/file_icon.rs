//! File and folder icons: the Material Icon Theme (PKief/vscode-material-icon-theme, MIT), as acepe maps
//! them (`packages/ui/src/lib/file-icon/extension-map.ts`). The one place an icon keeps its own colours.
//!
//! A file's icon comes from its name first (`package.json`, `Cargo.toml`), then its extension (`d.ts`
//! before `ts`), then the plain file. A trailing `:line` or `:line:column` is ignored. A folder is open or
//! closed, and some names have their own (`src`, `tests`, `.github`). Only the icons the map names are
//! embedded: 218 SVGs, 142 KB.
//!
//! [`FileIcon`] draws one with GPUI's `img`, which keeps an SVG's colours, where `svg` would paint it in
//! one ink. An app can draw file and folder icons its own way with [`set_source`].

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::{bytes, strip_location};
pub use helpers::{file_icon, folder_icon, icon_for_extension, icon_for_name, set_source};
pub use structs::FileIcon;
pub use types::{IconFor, Source};

#[cfg(test)]
mod tests;
