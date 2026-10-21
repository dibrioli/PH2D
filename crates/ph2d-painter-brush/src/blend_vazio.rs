//! **O modo do pincel sobre o VAZIO** (L7-vazio, `docs/Painter/Pinceis/06_LEIS_DAS_LACUNAS.md` §2 e
//! `03_MOTORES.md` §4 L7).
//!
//! [`crate::blend::blend_over`] guarda o alfa do destino nos modos de cor (`Multiply`, `Color`, `Hue`…): numa
//! camada TRANSPARENTE eles não pintam nada. O W3C *Compositing and Blending* §5.8 compõe com a fórmula
//! GERAL — `Cs' = (1 − αb)·Cs + αb·B(Cb, Cs)` e depois source-over a `α` —, que sobre o vazio pousa a cor
//! do pincel em Normal, e o Rebelle faz o mesmo (medido: α no papel = α na tinta ± 0,005, 06 §2).
//!
//! ⚠️ **Sobre a tela OPACA as duas leis são a mesma** (`αb = 1` ⇒ `Cs' = B`, `αo = 1`), e a porta só a
//! troca onde `αb < 1`: um pincel com o campo ligado pinta ao bit o de sempre sobre tinta opaca. Desligado
//! (o de fábrica, a cerca da linha) a porta É a `blend_over` — nenhum carimbo muda.

use crate::blend::{BrushBlend, blend_over, blend_rgb_de};

impl BrushBlend {
    /// O modo compõe por `B(Cb, Cs)` — não é o `Mix` nem um dos dois modos de ALFA.
    #[must_use]
    pub fn e_de_cor(self) -> bool {
        !matches!(
            self,
            BrushBlend::Mix | BrushBlend::EraseAlpha | BrushBlend::AddAlpha
        )
    }
}

/// A fórmula GERAL do W3C (`Cs' = (1 − αb)·Cs + αb·B(Cb, Cs)`, source-over a `a`) para um modo de cor.
#[must_use]
pub fn blend_over_w3c(mode: BrushBlend, dst: [f32; 4], color: [f32; 3], a: f32) -> [f32; 4] {
    let ab = dst[3].clamp(0.0, 1.0);
    let b = blend_rgb_de(mode, [dst[0], dst[1], dst[2]], color);
    let cs = [0, 1, 2].map(|k| (1.0 - ab) * color[k] + ab * b[k]);
    blend_over(BrushBlend::Mix, dst, cs, a)
}

/// Se a fórmula geral age neste texel: o campo ligado, um modo de cor, e o destino não opaco.
#[inline]
fn geral(mode: BrushBlend, dst: [f32; 4], vazio: bool) -> bool {
    vazio && dst[3] < 1.0 && mode.e_de_cor()
}

/// ⭐ **A porta do modo** — `vazio = false` é a [`blend_over`] ao bit.
#[must_use]
pub fn blend_over_modo(
    mode: BrushBlend,
    dst: [f32; 4],
    color: [f32; 3],
    a: f32,
    vazio: bool,
) -> [f32; 4] {
    if geral(mode, dst, vazio) {
        blend_over_w3c(mode, dst, color, a)
    } else {
        blend_over(mode, dst, color, a)
    }
}

/// A porta do modo com a mistura de pigmento — `vazio = false` é a [`crate::blend::blend_over_pigmento`] ao bit. Onde a
/// fórmula geral age o destino não é tinta opaca, e a mistura K–M (lei de tinta SOBRE tinta) não entra.
#[must_use]
#[inline]
pub fn blend_over_pigment_modo(
    mode: BrushBlend,
    dst: [f32; 4],
    color: [f32; 3],
    a: f32,
    pigmento: impl Into<crate::blend::Pigmento>,
    vazio: bool,
) -> [f32; 4] {
    if geral(mode, dst, vazio) {
        blend_over_w3c(mode, dst, color, a)
    } else {
        crate::blend::blend_over_pigmento(mode, dst, color, a, pigmento.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sobre o vazio a cor pousa em Normal a `α`; sobre a tela opaca é a `blend_over` de sempre (a ≤ 1 ulp).
    #[test]
    fn sobre_o_vazio_pousa_a_cor_e_sobre_a_tinta_e_o_de_sempre() {
        let azul = [0.09, 0.4, 0.7];
        for m in BrushBlend::ALL.into_iter().filter(|m| m.e_de_cor()) {
            let o = blend_over_modo(m, [0.0; 4], azul, 0.3, true);
            assert!((o[3] - 0.3).abs() < 1e-6, "{m:?}: α {}", o[3]);
            for k in 0..3 {
                assert!((o[k] - azul[k]).abs() < 1e-5, "{m:?}: cor {o:?}");
            }
            let base = [0.8, 0.2, 0.5, 1.0];
            let hoje = blend_over(m, base, azul, 0.3);
            assert_eq!(
                blend_over_modo(m, base, azul, 0.3, true),
                hoje,
                "{m:?} opaco"
            );
            assert_eq!(
                blend_over_modo(m, [0.0; 4], azul, 0.3, false)[3],
                0.0,
                "{m:?} desligado"
            );
        }
    }
}
