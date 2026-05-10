//! Spacing scale tokens. Source: `docs/design/tokens.json` → `spacing.*`.
//!
//! 8 px base scale with sub-base steps for tight UI density. Section
//! gap canônico é 14 px (não-power-of-2 — design choice, ver tokens.json).

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
    /// `xl2` (`2xl` no JSON) — 24 px (section separation).
    Xl2,
    /// `xl3` (`3xl` no JSON) — 32 px (major panel margin).
    Xl3,
    /// `xl4` (`4xl` no JSON) — 48 px (hero spacing).
    Xl4,
}

impl Spacing {
    pub const fn px(self) -> f32 {
        match self {
            Self::Xxs => 2.0,
            Self::Xs => 4.0,
            Self::Sm => 6.0,
            Self::Md => 8.0,
            Self::Lg => 12.0,
            Self::Xl => 16.0,
            Self::Xl2 => 24.0,
            Self::Xl3 => 32.0,
            Self::Xl4 => 48.0,
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

/// Section gap fixo (não-power-of-2). Per tokens.json `section-gap`.
pub const SECTION_GAP_PX: f32 = 14.0;

/// Row height por densidade. Per tokens.json `row-h` (forge-sdf default).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Density {
    /// 22 px row height — máxima densidade.
    Compact,
    /// 26 px row height — equilíbrio.
    Cozy,
    /// 32 px row height — confortável (default tablet/Pencil).
    #[default]
    Comfortable,
}

impl Density {
    pub const fn row_h_px(self) -> f32 {
        match self {
            Self::Compact => 22.0,
            Self::Cozy => 26.0,
            Self::Comfortable => 32.0,
        }
    }
}

/// Default icon-button square size (forge-sdf default = 36 px).
pub const ICON_BTN_SIZE_PX: f32 = 36.0;

/// Default body row height (matches forge-sdf `row-h` = 28).
pub const ROW_H_PX: f32 = 28.0;

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
