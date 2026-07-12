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

use ph2d_vec_scene::{Marker, StrokeSpec, VecPath, trim_path};
use ph2d_vector::{Affine, Brush, Fill, Stroke, VectorScene};

use crate::{build_bezpath, color};

/// A linha JÁ ENCURTADA para caber as pontas. `None` = o caminho é mais curto que os recuos
/// somados (uma linha de 2 px com uma seta gorda) — aí não há linha, só as pontas.
///
/// O recuo passa pelo MESMO `marker_scale` que a geometria da ponta (`draw`, abaixo): uma
/// cabeça maior tem de empurrar a linha mais para trás, senão o traço reaparece atravessando
/// a seta. Recuo e cabeça são a mesma medida vista dos dois lados.
#[must_use]
pub(crate) fn stroked_line(path: &VecPath, s: &StrokeSpec) -> Option<VecPath> {
    trim_path(
        path,
        s.marker_start.inset(s.marker_scale) * s.width,
        s.marker_end.inset(s.marker_scale) * s.width,
    )
}

/// A ponta de um extremo do traço, já em MUNDO.
///
/// **A geometria mora em `ph2d_vec_scene::stroke_head`, e não aqui** — porque o hit-test do
/// canvas precisa da MESMA cabeça. Enquanto ela vivia neste módulo, o mouse não a enxergava:
/// clicar no triângulo (a parte gorda da seta, a que o olho mira) não selecionava nada, e a
/// única área clicável era o fio da linha.
///
/// Este wrapper fica porque é aqui que a COSTURA é testada (`markers_tests`): compilar prova
/// que os tipos batem; não prova que o `marker_scale` do painel chegou na cabeça — e um `1.0`
/// esquecido no lugar dele deixaria a linha (que recua pelo `inset(scale)`) parando longe de
/// uma cabeça pequena, com um vão no meio.
#[must_use]
pub(crate) fn head(path: &VecPath, s: &StrokeSpec, at_start: bool) -> Option<(Marker, VecPath)> {
    ph2d_vec_scene::stroke_head(path, s, at_start)
}

/// Emite as duas pontas do traço.
pub(crate) fn draw(target: &mut VectorScene, path: &VecPath, s: &StrokeSpec, xf: Affine) {
    for at_start in [true, false] {
        let Some((marker, geo)) = head(path, s, at_start) else {
            continue;
        };
        let bp = build_bezpath(&geo);
        let brush = Brush::Solid(color(s.color));
        if marker.is_filled() {
            target
                .inner_mut()
                .fill(Fill::NonZero, xf, &brush, None, &bp);
        } else {
            // As vazadas usam a caneta da linha na LARGURA (a `width` CRUA, sem o
            // `marker_scale`: o `scale` é o tamanho da CABEÇA, não a grossura da caneta que a
            // desenha — engrossar o risco junto engordaria um losango vazado até fechá-lo),
            // mas **sempre sólida**: uma linha tracejada com um losango vazado desenharia o
            // losango pontilhado, o que é ruído. O tracejado é da LINHA; a ponta é um símbolo.
            target
                .inner_mut()
                .stroke(&Stroke::new(s.width), xf, &brush, None, &bp);
        }
    }
}

#[cfg(test)]
#[path = "markers_tests.rs"]
mod tests;
