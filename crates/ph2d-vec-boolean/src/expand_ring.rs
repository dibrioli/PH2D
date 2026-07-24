//! **O offset DIRETO** — módulo irmão de [`super`] (teto de LOC), o caminho barato do Contour VIVO.
//!
//! A borda do traço sem a booleana: ~668× mais barato que [`super::offset_path`] e **nunca passa
//! pelo `linesweeper`** (não panica). Cobre o caso comum do preview — contorno único, `Outer`,
//! CRESCER — que é a raiz do FPS e do pisca-pisca que o Enio reportou.

use kurbo::{BezPath, Cap, PathEl, Shape, Stroke, StrokeOpts};
use ph2d_vec_scene::{FillRule, LineJoin, OffsetSide, VecPath};

use super::{MIN_OFFSET, MIN_TOL, join_of, tolerance, weld_seams};
use crate::{Closing, to_bez_with, verts_from_bez};

/// **Offset DIRETO — a borda do traço, sem a booleana.** ~668× mais barato que
/// [`super::offset_path`] e **nunca passa pelo `linesweeper`** (não panica).
///
/// # A ideia, que é a mesma que o Pattern/Blend usam
///
/// Um anel de contour é a **dilatação de Minkowski** da forma por um disco de raio `d` — a forma
/// crescida. O contorno EXTERNO do `kurbo::stroke(forma, 2·d)` **é** essa fronteira; preenchê-lo
/// resolve a dilatação. O caro (a booleana) só existia para *limpar a geometria*; para um
/// preview que o **rasterizador** preenche com `NonZero`, a geometria não precisa ser limpa — o
/// Vello resolve a auto-interseção. Medido (`the_ring_matches_the_booleana`): a área **NonZero**
/// do loop externo é IDÊNTICA à da booleana, convexo E côncavo — e **correta onde a booleana
/// panica**.
///
/// Por isso o resultado sai com **`fill_rule = NonZero`** e pode ser auto-cruzado: é um preview,
/// não geometria editável. O [`super::offset_path`] (booleana) fica para o **Expand**, que
/// materializa formas limpas num clique único — não por-frame.
///
/// # O domínio, e por que devolve `None` fora dele
///
/// Só o **contorno único** (sem furos), lado **Outer**, e **`d > 0` (CRESCER)**. `None` diz ao
/// chamador *"use a booleana"*, e ele o diz para três casos, cada um por um motivo:
///
/// - **compound / Inner/Both** — topologia (furo que encolhe, dois lados) que a booleana resolve;
/// - **`d ≤ 0` (ENCOLHER)** — ⚠️ o `kurbo::stroke` **DERRUBA o laço de erosão** quando `|d|` fica
///   grande relativo à forma (MEDIDO, `probe_square_loops`: a erosão de um quadrado unitário some
///   a `|d| = 0,3`, e `[0,3;0,7]²` é VÁLIDA, não anihilada). Cobrir só o alcance em que ele
///   responde deixaria anéis do MESMO contour cozidos por métodos diferentes — direto para `|d|`
///   pequeno, booleana para grande — e a divergência medida entre os dois (4,2% num hexágono a
///   `|d|=0,3`) viraria um **salto no crossover** durante o arrasto. A erosão inteira pela booleana
///   é CONSISTENTE (sem crossover). O caro é pago só ao encolher, que é o caso raro; o comum
///   (crescer) fica no caminho barato. `Some(vec![])` é diferente de `None`: *"cresci e a forma
///   degenerou"* (entrada patológica) — não acontece na prática.
///
/// A dilatação de Minkowski é robusta no `kurbo::stroke` (o contorno EXTERNO sempre existe), então
/// o crescer não tem esse problema.
#[must_use]
pub fn offset_ring(
    path: &VecPath,
    d: f64,
    join: LineJoin,
    side: OffsetSide,
) -> Option<Vec<VecPath>> {
    // `d < MIN_OFFSET` recusa o encolher (`d ≤ 0`) E o offset ~nulo de uma vez: o direto só CRESCE.
    if !d.is_finite() || d < MIN_OFFSET || !path.subpaths.is_empty() || side != OffsetSide::Outer {
        return None;
    }
    let bez = to_bez_with(path, Closing::Always);
    let tol = tolerance(&bez);
    let pen = Stroke::new(2.0 * d)
        .with_join(join_of(join))
        .with_caps(Cap::Butt);
    let traced = weld_seams(&kurbo::stroke(&bez, &pen, &StrokeOpts::default(), tol), tol);
    // O traço de um laço fechado tem DOIS contornos: a **dilatação** (a borda + `d`, o que
    // queremos) e a **erosão** (a borda − `d`). O kurbo emite a erosão como um FURO — winding
    // INVERTIDO em relação à fonte, a convenção do donut —, então a dilatação tem o MESMO sinal de
    // área que a fonte.
    //
    // ⚠️ **O discriminador é o SINAL da área, não o bbox nem a |área|.** Numa forma CÔNCAVA a Miter
    // crava spikes de auto-interseção que devolvem o bbox da erosão ao tamanho da fonte, e o
    // shoelace-abs conta a sobreposição com multiplicidade — os dois podem confundir os dois laços
    // (MEDIDO, `probe_ring_loops`). O sinal é imune à auto-interseção (é o winding dominante), a
    // mesma lição do gate de paridade — shoelace mente sobre winding.
    let src_positive = bez.area() >= 0.0;
    let chosen = split_loops(&traced)
        .into_iter()
        .map(|l| (l.area(), l))
        .filter(|(a, _)| a.abs() > MIN_TOL) // laço degenerado (ponto) não conta
        // A dilatação tem o MESMO sinal da fonte (a erosão o oposto).
        .filter(|(a, _)| (*a >= 0.0) == src_positive)
        // Se houver mais de um laço do lado certo (slivers), fica o de maior |área| — o principal.
        .max_by(|x, y| {
            x.0.abs()
                .partial_cmp(&y.0.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    let Some((_, loop_bez)) = chosen else {
        return Some(Vec::new()); // cresci e não sobrou dilatação: entrada degenerada
    };
    let Some(verts) = verts_from_bez(&loop_bez) else {
        return Some(Vec::new());
    };
    Some(vec![VecPath {
        verts,
        closed: true,
        fill: path.fill.clone(),
        stroke: path.stroke,
        // ⚠️ **NonZero, e é a espinha:** o loop pode ser auto-cruzado (côncavo + `d` grande), e é
        // o winding NonZero do rasterizador que preenche a auto-interseção — o mesmo que a
        // booleana faria, sem o custo dela.
        fill_rule: FillRule::NonZero,
        ..VecPath::default()
    }])
}

/// Quebra um `BezPath` de vários sub-contornos num `BezPath` por contorno (corte em cada `MoveTo`).
fn split_loops(bez: &BezPath) -> Vec<BezPath> {
    let mut out: Vec<BezPath> = Vec::new();
    let mut cur = BezPath::new();
    for el in bez.elements() {
        if matches!(el, PathEl::MoveTo(_)) && !cur.elements().is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(*el);
    }
    if !cur.elements().is_empty() {
        out.push(cur);
    }
    out
}
