//! Theme enum.
//!
//! 4 themes per `docs/design/tokens.json`:
//! - `forge-sdf` (default) — dark + magenta accent.
//! - `paint-studio` — dark + cyan accent (canvas-first vibe).
//! - `sunstone` — light + warm orange accent.
//! - `blueprint` — light + cool blue accent (CAD vibe), sidebar layout.
//!
//! Theme names come from design tokens (Procreate inspiration was
//! renamed to `paint-studio` to avoid the trademark).

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Theme {
    /// `forge-sdf` — dark + magenta. Default theme.
    #[default]
    ForgeSdf,
    /// `paint-studio` — dark + cyan (Procreate-inspired, trademark-free name).
    PaintStudio,
    /// `sunstone` — light + warm orange.
    Sunstone,
    /// `blueprint` — light + cool blue (sidebar layout).
    Blueprint,
}

/// Panel layout flag — declared per-theme in tokens.json.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelLayout {
    /// Floating panels (ForgeSdf / PaintStudio / Sunstone).
    Floating,
    /// Panels docked in the sidebar (Blueprint, CAD style).
    Sidebar,
}

impl Theme {
    /// Cycle through the 4 themes in tweaks-panel order.
    pub fn next(self) -> Self {
        match self {
            Self::ForgeSdf => Self::PaintStudio,
            Self::PaintStudio => Self::Sunstone,
            Self::Sunstone => Self::Blueprint,
            Self::Blueprint => Self::ForgeSdf,
        }
    }

    /// True when the theme is dark (low background luminance).
    pub fn is_dark(self) -> bool {
        matches!(self, Self::ForgeSdf | Self::PaintStudio)
    }

    /// Panel layout declared by the theme.
    pub fn panel_layout(self) -> PanelLayout {
        match self {
            Self::ForgeSdf | Self::PaintStudio | Self::Sunstone => PanelLayout::Floating,
            Self::Blueprint => PanelLayout::Sidebar,
        }
    }

    /// Stable identifier (matches `tokens.json` keys).
    pub fn id(self) -> &'static str {
        match self {
            Self::ForgeSdf => "forge-sdf",
            Self::PaintStudio => "paint-studio",
            Self::Sunstone => "sunstone",
            Self::Blueprint => "blueprint",
        }
    }

    /// Human-readable display name (used by the topbar theme
    /// cluster + the theme menu items).
    pub fn display_name(self) -> &'static str {
        match self {
            Self::ForgeSdf => "Forge SDF",
            Self::PaintStudio => "Paint Studio",
            Self::Sunstone => "Sunstone",
            Self::Blueprint => "Blueprint",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_forge_sdf() {
        assert_eq!(Theme::default(), Theme::ForgeSdf);
    }

    #[test]
    fn next_cycles_all_four() {
        let mut t = Theme::ForgeSdf;
        let mut visited = vec![t];
        for _ in 0..4 {
            t = t.next();
            visited.push(t);
        }
        assert_eq!(visited.len(), 5);
        assert_eq!(visited[0], visited[4]); // round-trip
        assert!(visited[..4].contains(&Theme::ForgeSdf));
        assert!(visited[..4].contains(&Theme::PaintStudio));
        assert!(visited[..4].contains(&Theme::Sunstone));
        assert!(visited[..4].contains(&Theme::Blueprint));
    }

    #[test]
    fn dark_themes_are_dark() {
        assert!(Theme::ForgeSdf.is_dark());
        assert!(Theme::PaintStudio.is_dark());
        assert!(!Theme::Sunstone.is_dark());
        assert!(!Theme::Blueprint.is_dark());
    }

    #[test]
    fn blueprint_uses_sidebar_layout() {
        assert_eq!(Theme::Blueprint.panel_layout(), PanelLayout::Sidebar);
        assert_eq!(Theme::ForgeSdf.panel_layout(), PanelLayout::Floating);
    }

    #[test]
    fn ids_match_tokens_json() {
        assert_eq!(Theme::ForgeSdf.id(), "forge-sdf");
        assert_eq!(Theme::PaintStudio.id(), "paint-studio");
        assert_eq!(Theme::Sunstone.id(), "sunstone");
        assert_eq!(Theme::Blueprint.id(), "blueprint");
    }
}
