//! Spacing scale tokens. Source: `docs/design/tokens.json` → `spacing.*`.
//!
//! 8 px base scale with sub-base steps for tight UI density. The
//! canonical section gap is 14 px (non-power-of-2 — design choice;
//! see tokens.json).
//!
//! Wave 4 stage A: values now come from `crate::generated::SPACING_*`,
//! `DENSITY_*` and `CHROME_*` const tables (codegen'd by build.rs from
//! `docs/design/tokens.json`). Designer edits the JSON; Rust replicates.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Spacing {
    /// `xxs` — 2 px (sub-pixel divider gaps).
    Xxs,
    /// `xs` — 4 px (icon ↔ label tight inline).
    Xs,
    /// `sm` — 6 px (compact rows).
    Sm,
    /// `md` — 8 px (default inline padding).
    Md,
    /// `lg` — 12 px (default vertical rhythm, panel padding).
    Lg,
    /// `xl` — 16 px (comfortable padding).
    Xl,
    /// `xl2` (`2xl` in JSON) — 24 px (section separation).
    Xl2,
    /// `xl3` (`3xl` in JSON) — 32 px (major panel margin).
    Xl3,
    /// `xl4` (`4xl` in JSON) — 48 px (hero spacing).
    Xl4,
}

impl Spacing {
    pub const fn px(self) -> f32 {
        match self {
            Self::Xxs => crate::generated::SPACING_XXS,
            Self::Xs => crate::generated::SPACING_XS,
            Self::Sm => crate::generated::SPACING_SM,
            Self::Md => crate::generated::SPACING_MD,
            Self::Lg => crate::generated::SPACING_LG,
            Self::Xl => crate::generated::SPACING_XL,
            Self::Xl2 => crate::generated::SPACING_XL2,
            Self::Xl3 => crate::generated::SPACING_XL3,
            Self::Xl4 => crate::generated::SPACING_XL4,
        }
    }

    /// Token id (matches JSON key).
    pub const fn id(self) -> &'static str {
        match self {
            Self::Xxs => "xxs",
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xl2 => "2xl",
            Self::Xl3 => "3xl",
            Self::Xl4 => "4xl",
        }
    }
}

/// Fixed section gap (non-power-of-2). Per tokens.json `chrome.section-gap`.
pub const SECTION_GAP_PX: f32 = crate::generated::CHROME_SECTION_GAP;

/// Row height by density. Per tokens.json `density.*`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Density {
    /// `compact` row height — max density.
    Compact,
    /// `cozy` row height — balanced.
    Cozy,
    /// `comfortable` row height — comfortable (default for tablet/Pencil).
    #[default]
    Comfortable,
}

impl Density {
    pub const fn row_h_px(self) -> f32 {
        match self {
            Self::Compact => crate::generated::DENSITY_COMPACT,
            Self::Cozy => crate::generated::DENSITY_COZY,
            Self::Comfortable => crate::generated::DENSITY_COMFORTABLE,
        }
    }
}

/// Default icon-button square size. Per tokens.json `chrome.icon-btn-size`.
pub const ICON_BTN_SIZE_PX: f32 = crate::generated::CHROME_ICON_BTN_SIZE;

/// Default body row height. Per tokens.json `chrome.row-h`.
pub const ROW_H_PX: f32 = crate::generated::CHROME_ROW_H;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_strictly_increasing() {
        let scale = [
            Spacing::Xxs,
            Spacing::Xs,
            Spacing::Sm,
            Spacing::Md,
            Spacing::Lg,
            Spacing::Xl,
            Spacing::Xl2,
            Spacing::Xl3,
            Spacing::Xl4,
        ];
        for w in scale.windows(2) {
            assert!(
                w[0].px() < w[1].px(),
                "scale broken at {:?} → {:?}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn ids_match_tokens_json() {
        assert_eq!(Spacing::Xxs.id(), "xxs");
        assert_eq!(Spacing::Xl2.id(), "2xl");
        assert_eq!(Spacing::Xl4.id(), "4xl");
    }

    #[test]
    fn density_row_height_strictly_increasing() {
        assert!(Density::Compact.row_h_px() < Density::Cozy.row_h_px());
        assert!(Density::Cozy.row_h_px() < Density::Comfortable.row_h_px());
    }

    #[test]
    fn comfortable_is_default() {
        assert_eq!(Density::default(), Density::Comfortable);
    }
}
