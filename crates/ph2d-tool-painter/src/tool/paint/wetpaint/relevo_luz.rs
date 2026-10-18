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
