//! **A luz do relevo dos fios** (W3): a altura das camadas visíveis ([`ph2d_wet_paint::grid::Grid::altura`],
//! em px) acesa pela luz do Impasto ([`crate::tool::paint::impasto_light::porta::LuzLivre`]) na linha de
//! pigmento que o render acabou de escrever — antes do `over` sobre a base, então a luz vai com a tinta.

use crate::tool::paint::impasto_light::DEPTH_UNIT_PX;
use crate::tool::paint::impasto_light::porta::LuzLivre;
use ph2d_wet_paint::render::RenderLayer;

/// Acende as células `cx0..=cx1` da linha de células `cy` (`row` = a linha do plano de pigmento, RGBA sRGB8
/// direito, a célula `cx` em `(cx − 1) · 4`). A altura soma-se entre as camadas visíveis (mais tinta é mais
/// alta, como no Impasto) e passa de px de altura por célula de `ratio` px às unidades da luz.
pub(super) fn acende_linha(
    luz: &LuzLivre,
    layers: &[RenderLayer<'_>],
    ratio: u8,
    cy: usize,
    cx0: usize,
    cx1: usize,
    row: &mut [u8],
) {
    let escala = 1.0 / (DEPTH_UNIT_PX * f32::from(ratio.max(1)));
    let altura = |cx: usize, cy: usize| -> f32 {
        layers
            .iter()
            .filter(|l| l.visible)
            .map(|l| ph2d_wet_paint::grid::altura::em(l.grid, cy * l.grid.s + cx))
            .sum::<f32>()
            * escala
    };
    for cx in cx0..=cx1 {
        let o = (cx - 1) * 4;
        let cover = f32::from(row[o + 3]) / 255.0;
        if cover <= 0.0 {
            continue;
        }
        let dhx = (altura(cx + 1, cy) - altura(cx - 1, cy)) * 0.5;
        let dhy = (altura(cx, cy + 1) - altura(cx, cy - 1)) * 0.5;
        luz.acende(dhx, dhy, cover, &mut row[o..o + 4]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **A UNIDADE da altura** (W3): o campo está em px, e uma rampa que sobe `s` px por px de tela acende-se
    /// como o Impasto acende o mesmo declive (`s / DEPTH_UNIT_PX` por texel) — a 1 px por célula e a 2 (a
    /// rampa por CÉLULA é a mesma, o declive por px é metade). Mutação «a escala ×16» passava os portões de
    /// produto (a luz satura); aqui não.
    #[test]
    fn a_altura_e_em_px_de_tela() {
        let luz = LuzLivre::nova(&ph2d_light::LightRig::default()).expect("a lâmpada de fábrica");
        for ratio in [1u8, 2] {
            let mut g = ph2d_wet_paint::grid::Grid::new(8, 3);
            let s = g.s;
            let campo = ph2d_wet_paint::grid::altura::garante(&mut g);
            for y in 0..g_rows(s, campo.len()) {
                for x in 0..s {
                    campo[y * s + x] = 0.3 * x as f32; // 0,3 px de altura por CÉLULA
                }
            }
            let layers = [RenderLayer {
                grid: &g,
                opacity: 1.0,
                visible: true,
            }];
            let mut row = [120u8, 80, 200, 255].repeat(8);
            acende_linha(&luz, &layers, ratio, 2, 1, 8, &mut row);
            let mut esperado = [120u8, 80, 200, 255];
            luz.acende(
                0.3 / (DEPTH_UNIT_PX * f32::from(ratio)),
                0.0,
                1.0,
                &mut esperado,
            );
            assert_ne!(esperado, [120, 80, 200, 255], "a rampa não acendeu");
            assert_eq!(row[16..20], esperado, "ratio {ratio}");
        }
    }

    fn g_rows(s: usize, n: usize) -> usize {
        n / s
    }
}
