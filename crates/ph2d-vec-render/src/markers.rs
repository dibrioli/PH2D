//! **As pontas de traço, desenhadas** — módulo irmão de [`crate::lib`].
//!
//! A geometria mora em `ph2d_vec_scene::marker` (pura, testável, sem render). Aqui está só
//! o que é de RENDER:
//!
//! 1. a linha é **encurtada** pelos recuos das duas pontas antes de ser traçada;
//! 2. cada ponta é construída na tangente do extremo e emitida — **preenchida** (a seta
//!    cheia, o losango, a bolinha) ou **traçada** com a mesma caneta da linha (a aberta, a
//!    barra, os vazados), que é o que as faz parecerem do mesmo desenho.
//!
//! Nada disto roda quando o traço não tem ponta, que é o caso da esmagadora maioria dos
//! paths — o `has_markers()` corta antes.

use ph2d_vec_scene::{Marker, StrokeSpec, VecPath, end_tangent, trim_path};
use ph2d_vector::{Affine, Brush, Fill, Stroke, VectorScene};

use crate::{build_bezpath, color};

/// A linha JÁ ENCURTADA para caber as pontas. `None` = o caminho é mais curto que os recuos
/// somados (uma linha de 2 px com uma seta gorda) — aí não há linha, só as pontas.
#[must_use]
pub(crate) fn stroked_line(path: &VecPath, s: &StrokeSpec) -> Option<VecPath> {
    trim_path(
        path,
        s.marker_start.inset() * s.width,
        s.marker_end.inset() * s.width,
    )
}

/// Emite as duas pontas do traço.
pub(crate) fn draw(target: &mut VectorScene, path: &VecPath, s: &StrokeSpec, xf: Affine) {
    // Contorno fechado não tem pontas (não há extremo onde pô-las).
    if path.closed {
        return;
    }
    for (marker, at_start) in [(s.marker_start, true), (s.marker_end, false)] {
        if marker == Marker::None {
            continue;
        }
        let Some((tip, dir)) = end_tangent(path, at_start) else {
            continue;
        };
        let Some(geo) = marker.build(tip, dir, s.width) else {
            continue;
        };
        let bp = build_bezpath(&geo);
        let brush = Brush::Solid(color(s.color));
        if marker.is_filled() {
            target
                .inner_mut()
                .fill(Fill::NonZero, xf, &brush, None, &bp);
        } else {
            // As vazadas usam a caneta da linha na LARGURA, mas **sempre sólida**: uma linha
            // tracejada com um losango vazado desenharia o losango pontilhado, o que é
            // ruído. O tracejado é da LINHA; a ponta é um símbolo.
            target
                .inner_mut()
                .stroke(&Stroke::new(s.width), xf, &brush, None, &bp);
        }
    }
}
