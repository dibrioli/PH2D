//! A COR do mixer num espaço perceptual (ciclo 18, doc 127) — cortado do `lib.rs` pelo tecto de LOC.

use super::{MODE_AVG, MODE_BLEND, blend_at};
use ph2d_nodegraph::attr::Column;

/// ⭐ **A cor misturada num espaço perceptual** (ciclo 18): o `Blend` pela mistura da casa
/// ([`ph2d_color::perceptual::mix`]), o `Avg` pela média ponderada em OKLab e volta à gama.
/// `None` (outros modos, ou uma coluna que não é RGBA) ⇒ a mistura de sempre.
pub(super) fn cor_perceptual(
    mode: i64,
    cols: &[Column],
    blend: &[f32],
    weights: &[f32],
    total_w: f32,
    count: usize,
    espaco: ph2d_color::perceptual::MixSpace,
) -> Option<Column> {
    use ph2d_color::perceptual as pc;
    let vs: Vec<&Vec<[f32; 4]>> = cols
        .iter()
        .map(|c| match c {
            Column::Vec4(v) => Some(v),
            _ => None,
        })
        .collect::<Option<_>>()?;
    match mode {
        MODE_BLEND if vs.len() >= 2 => Some(Column::Vec4(
            (0..count)
                .map(|i| pc::mix(vs[0][i], vs[1][i], blend_at(blend, i), espaco))
                .collect(),
        )),
        MODE_BLEND | MODE_AVG if total_w != 0.0 => Some(Column::Vec4(
            (0..count)
                .map(|i| {
                    let (mut lab, mut alfa) = ([0.0_f32; 3], 0.0_f32);
                    for (v, w) in vs.iter().zip(weights) {
                        let o = pc::to_oklab([v[i][0], v[i][1], v[i][2]]);
                        for k in 0..3 {
                            lab[k] += o[k] * *w;
                        }
                        alfa += v[i][3] * *w;
                    }
                    let lab = lab.map(|x| x / total_w);
                    let rgb = pc::gamut_map(pc::oklab_to_oklch(lab));
                    [rgb[0], rgb[1], rgb[2], alfa / total_w]
                })
                .collect(),
        )),
        _ => None,
    }
}
