//! Border-radius tokens. Source: `docs/design/tokens.json` → `radius.*`.
//!
//! Tier-based ("soft" default across all themes); values here are the
//! soft tier. `full` (999) yields a perfect circle (avatars, pills).
//!
//! Wave 4 stage A: values now come from `crate::generated::RADIUS_*`
//! consts (codegen'd by build.rs).

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Radius {
    /// `xs` — 4 px (chips, micro-tags).
    Xs,
    /// `sm` — 6 px (compact buttons, dense inputs).
    Sm,
    /// `md` — **4 px** (default cards, inner panels).
    ///
    /// ⚠️ Este doc dizia **8 px** e o `tokens.json` diz `4` desde que alguém o baixou — *um
    /// comentário velho mente com a mesma cara de um novo*, e este mentia no sítio caro: quem
    /// escolhesse `Md` a ler «8» ficava com metade do raio que pediu. Corrigido 2026-09-02, ao
    /// escolher o token do raio padrão dos sliders. ⛔ O valor é DERIVADO (`RADIUS_MD`); se voltar
    /// a divergir, é este doc que está errado.
    Md,
    /// `lg` — 12 px (floating panels, modals).
    Lg,
    /// `xl` — 16 px (hero cards, splash).
    Xl,
    /// `xl2` (`2xl` in JSON) — 20 px (large surfaces).
    Xl2,
    /// `full` — 999 px (perfect circle).
    Full,
}

impl Radius {
    /// O valor de **FÁBRICA** — irmão exacto do [`crate::Spacing::factory_px`], e o porquê da
    /// separação está escrito lá.
    pub const fn factory_px(self) -> f32 {
        match self {
            Self::Xs => crate::generated::RADIUS_XS,
            Self::Sm => crate::generated::RADIUS_SM,
            Self::Md => crate::generated::RADIUS_MD,
            Self::Lg => crate::generated::RADIUS_LG,
            Self::Xl => crate::generated::RADIUS_XL,
            Self::Xl2 => crate::generated::RADIUS_XL2,
            Self::Full => crate::generated::RADIUS_FULL,
        }
    }

    /// O valor **VIVO** — o que o artista autorou neste modo, ou a fábrica.
    #[must_use]
    pub fn px(self) -> f32 {
        crate::num_runtime::live(crate::num::NumToken::Radius(self)).unwrap_or(self.factory_px())
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xl2 => "2xl",
            Self::Full => "full",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_monotonic_until_full() {
        // `Md` was halved 8 → 4 px on 2026-05-24 to soften the default
        // card/inner-panel corners. That made Md (4) < Sm (6), which is
        // an intentional inversion — Md is "default cards" not "between
        // Sm and Lg by size." The remaining tiers (Xs, Sm, Lg, Xl, Xl2)
        // are still strictly increasing.
        let scale_excluding_md = [Radius::Xs, Radius::Sm, Radius::Lg, Radius::Xl, Radius::Xl2];
        for w in scale_excluding_md.windows(2) {
            assert!(w[0].px() < w[1].px(), "{:?} → {:?}", w[0], w[1]);
        }
        assert!(Radius::Full.px() > Radius::Xl2.px());
        // Sanity: Md should sit at the floor of the scale (≤ Xs is OK).
        assert!(Radius::Md.px() <= Radius::Sm.px());
    }
}
