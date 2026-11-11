//! A TINTA de um texel com silhueta `w > 0`: o Grain (com o pó), a rampa, o dente e o stencil — a cor, o alfa da
//! textura e o factor de cobertura `g`. Fase 2 de [`super::stamp_band`]; `None` ⇒ o texel não recebe nada.

use super::silhueta::Silhueta;
use super::{DabCtx, Invariantes};

pub(in crate::dab) struct Tinta {
    pub(in crate::dab) color: [f32; 3],
    /// Mode TextureAlpha: the source pixel's own alpha.
    pub(in crate::dab) stamp_alpha: f32,
    pub(in crate::dab) g: f32,
}

#[inline(always)]
pub(in crate::dab) fn em(
    ctx: &DabCtx,
    inv: &Invariantes,
    s: Silhueta,
    px: i64,
    py: i64,
) -> Option<Tinta> {
    let w = s.w;
    // `w` = dab PROFILE (falloff × silhouette, the build factor); `g` = texture COVERAGE factor
    // (Grain / Strength-ramp) kept SEPARATE so it CAPS the pixel — a 0.3 grain texel tops at
    // 0.3·Strength under re-passing, not climbing to full (the "fills in" bug). `g=1` ⇒ identical.
    // The colour and the Grain's bite come from ONE door ([`crate::ramp_alpha::ramp_texel`]):
    // the ramp is indexed by its OWNER's value (the Grain, or the Shape's layers: its texture
    // value and the coverage `w`), and the Grain bites unless it is the one indexing the ramp.
    let grain = ctx.tex.map(|b| {
        let amostra = || {
            crate::texture::sample(
                &ctx.spec.texture,
                b,
                px,
                py,
                ctx.center,
                ctx.radius,
                ctx.image.as_ref(),
            )
        };
        match ctx.grao.as_deref() {
            Some(c) => c.em(px, py, amostra),
            None => amostra(),
        }
    });
    let lei = &ctx.spec.texture.lei;
    let grain = grain.map(|t| lei.papel_com_po(t, ctx.po.as_ref(), px, py));
    let (depth, gran) = (inv.depth, inv.gran);
    let tinta = match s.cor_cap {
        Some(cc) => {
            let mordida = crate::ramp_alpha::ramp_texel(None, w, None, grain, depth, gran, lei).g;
            cc.fim(mordida)?
        }
        None => {
            let forma = ctx
                .shape_cor
                .map(|sh| crate::dab::shape_value_at(ctx.spec, sh, px, py, ctx.center, ctx.radius));
            crate::ramp_alpha::ramp_texel(ctx.ramp.as_ref(), w, forma, grain, depth, gran, lei)
        }
    };
    let color = tinta.color.unwrap_or(ctx.spec.color);
    let mut g = tinta.g * crate::dente::dente_at(ctx.spec, w, px, py);
    if let Some(b) = ctx.tex {
        g *= crate::texture::stencil_gate(&ctx.spec.texture, b, px, py); // rect mask, ramp-safe
        if w * g <= 0.0 {
            return None;
        }
    }
    Some(Tinta {
        color,
        stamp_alpha: tinta.stamp_alpha,
        g,
    })
}
