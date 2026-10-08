use super::structs::Bundled;

pub(super) const VSCODE: &[Bundled] = &[
    Bundled {
        name: "Cursor Dark",
        family: "Cursor",
        json: include_str!("../../assets/themes/vscode/cursor-dark.json"),
    },
    Bundled {
        name: "Cursor Light",
        family: "Cursor",
        json: include_str!("../../assets/themes/vscode/cursor-light.json"),
    },
    Bundled {
        name: "GitHub Light",
        family: "GitHub",
        json: include_str!("../../assets/themes/vscode/github-light.json"),
    },
    Bundled {
        name: "GitHub Dark",
        family: "GitHub",
        json: include_str!("../../assets/themes/vscode/github-dark.json"),
    },
    Bundled {
        name: "Catppuccin Latte",
        family: "Catppuccin",
        json: include_str!("../../assets/themes/vscode/catppuccin-latte.json"),
    },
    Bundled {
        name: "Catppuccin Frappé",
        family: "Catppuccin",
        json: include_str!("../../assets/themes/vscode/catppuccin-frappe.json"),
    },
    Bundled {
        name: "Catppuccin Macchiato",
        family: "Catppuccin",
        json: include_str!("../../assets/themes/vscode/catppuccin-macchiato.json"),
    },
    Bundled {
        name: "Catppuccin Mocha",
        family: "Catppuccin",
        json: include_str!("../../assets/themes/vscode/catppuccin-mocha.json"),
    },
];

pub(super) const ATELIER: [&str; 2] = [
    include_str!("../../assets/themes/atelier-light.json"),
    include_str!("../../assets/themes/atelier-dark.json"),
];
