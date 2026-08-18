//! Gates for `motion.path` (doc 65).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::Graph;

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == MANIFEST.id).then_some(&MotionPath as &dyn NodeOp)
    }
}

/// Publish `curve` under `name`, cook the node, and hand back its `(positions, rotations)`.
fn walk(curve: &[[f32; 2]], name: &str, params: &[(&str, f32)]) -> (Vec<[f32; 2]>, Vec<f32>) {
    let mut g = Graph::new();
    let n = g.add_node("motion.path");
    g.set_text_param(n, PATH_PARAM, name);
    for (k, v) in params {
        g.set_param(n, *k, *v);
    }
    let mut cook = Cook::new();
    cook.set_external(
        ph2d_nodegraph::external::curve_of(name),
        Stream::new(curve.len()).with("P", Column::Vec2(curve.to_vec())),
    );
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    let s = out[0].as_stream();
    let pos = match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    let rot = match s.get("rot") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    (pos, rot)
}

/// A straight line 10 long, walked by 5 instances: they land at 0, 2, 4, 6, 8 — **even
/// arc-length**, which is the whole promise.
#[test]
fn instances_land_at_even_arc_length_along_the_drawn_curve() {
    let line = [[0.0, 0.0], [10.0, 0.0]];
    let (pos, _) = walk(&line, "Track", &[("count", 5.0), ("align", 0.0)]);
    assert_eq!(pos.len(), 5);
    for (i, p) in pos.iter().enumerate() {
        assert!(
            (p[0] - i as f32 * 2.0).abs() < 1e-3,
            "instance {i} should sit at x = {}, not {}",
            i as f32 * 2.0,
            p[0]
        );
        assert!(p[1].abs() < 1e-4, "…and on the line");
    }
}

/// **Even arc-length, not even parameter.** A polyline whose two segments have very different
/// lengths is where the two disagree: by parameter the instances would bunch on the short leg.
#[test]
fn a_long_leg_gets_more_instances_than_a_short_one() {
    // A 9-long leg, then a 1-long one.
    let bent = [[0.0, 0.0], [9.0, 0.0], [9.0, 1.0]];
    let (pos, _) = walk(&bent, "Bend", &[("count", 10.0), ("align", 0.0)]);
    let on_long = pos.iter().filter(|p| p[1] < 0.001 && p[0] < 9.0).count();
    assert!(
        on_long >= 8,
        "the long leg is 90% of the arc, so it must take ~90% of the instances - got {on_long}/10"
    );
}

/// **The offset WRAPS.** A curve is a thing to walk around, not a line to fall off the end of — so
/// sliding by a whole turn puts everything back where it started.
#[test]
fn the_offset_slides_and_wraps() {
    let line = [[0.0, 0.0], [10.0, 0.0]];
    let base = walk(&line, "T", &[("count", 4.0), ("align", 0.0)]).0;
    let slid = walk(
        &line,
        "T",
        &[("count", 4.0), ("offset", 0.125), ("align", 0.0)],
    )
    .0;
    assert!(
        (slid[0][0] - 1.25).abs() < 1e-3,
        "an eighth of a 10-long curve is 1.25: {}",
        slid[0][0]
    );
    let round = walk(
        &line,
        "T",
        &[("count", 4.0), ("offset", 1.0), ("align", 0.0)],
    )
    .0;
    assert_eq!(round, base, "a whole turn is where you started");
}

/// **Align turns the instance to face the way the curve is going** — a set marching along a path
/// that all point the same way is a set that is not following anything.
#[test]
fn align_turns_the_instances_to_the_tangent() {
    // Right, then up: the first half of the arc points at 0°, the second at 90°.
    let corner = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]];
    let (_, rot) = walk(&corner, "L", &[("count", 4.0), ("align", 1.0)]);
    assert_eq!(rot.len(), 4);
    assert!(rot[0].abs() < 1.0, "the first leg runs east: {}", rot[0]);
    assert!(
        (rot[3] - 90.0).abs() < 1.0,
        "the second runs north: {}",
        rot[3]
    );

    // …and with align off, the node does not write the column at all (it does not silently pin
    // every instance to 0°, which would fight a `motion.rotate` downstream).
    let (_, none) = walk(&corner, "L", &[("count", 4.0), ("align", 0.0)]);
    assert!(none.is_empty(), "no align, no `rot` column");
}

/// **A shape that is not there is an EMPTY stream** — not a panic, not a guess. The artist has not
/// drawn it yet, or renamed it, or deleted it; the node emits nothing and the scene is simply
/// empty, which is the truth.
#[test]
fn a_missing_shape_emits_nothing() {
    let mut g = Graph::new();
    let n = g.add_node("motion.path");
    g.set_text_param(n, PATH_PARAM, "NotDrawnYet");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    assert_eq!(out[0].as_stream().count(), 0);

    // A shape with a single point is not a curve either — no arc to walk.
    cook.set_external(
        "NotDrawnYet",
        Stream::new(1).with("P", Column::Vec2(vec![[1.0, 1.0]])),
    );
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    assert_eq!(out[0].as_stream().count(), 0);
}

/// **Editing the curve moves the instances.** The end-to-end claim of the whole external channel:
/// nothing in this node's graph changed, and it still followed.
#[test]
fn dragging_the_shape_moves_the_set() {
    let mut g = Graph::new();
    let n = g.add_node("motion.path");
    g.set_text_param(n, PATH_PARAM, "Track");
    g.set_param(n, "count", 2.0);
    g.set_param(n, "align", 0.0);
    let mut cook = Cook::new();

    cook.set_external(
        ph2d_nodegraph::external::curve_of("Track"),
        Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [10.0, 0.0]])),
    );
    let before = cook.cook(&g, &Ops, n, 0.0).unwrap()[0]
        .as_stream()
        .get("P")
        .cloned();

    // The artist drags the curve up.
    cook.set_external(
        ph2d_nodegraph::external::curve_of("Track"),
        Stream::new(2).with("P", Column::Vec2(vec![[0.0, 5.0], [10.0, 5.0]])),
    );
    let after = cook.cook(&g, &Ops, n, 0.0).unwrap()[0]
        .as_stream()
        .get("P")
        .cloned();

    assert_ne!(
        before, after,
        "the memo must SEE the curve: edit it and the set has to move, or the node hands back the \
         pre-edit shape forever"
    );
    match after {
        Some(Column::Vec2(v)) => assert!(v.iter().all(|p| (p[1] - 5.0).abs() < 1e-4)),
        _ => panic!("P"),
    }
}

// ---------------------------------------------------------------------------
// A CONTAGEM DERIVADA DO ESPAÇAMENTO (folha 06 linha 46)
//
// A célula pedia CINCO controles que o `pattern_along_path` do módulo Vector
// shipa e este nó não tinha. ⚠️ Medido (`measure_path_controls`), QUATRO deles
// já eram alcançáveis — o `Slide` **é** o `offset` deste nó, e Start/End,
// perpendicular e Side saem do irmão `motion.spline_wrap`, que lê a MESMA curva
// desenhada e cuja `frame_at` chama a MESMA `ph2d_arc_length::at`. Sobrou este.
// ---------------------------------------------------------------------------

/// Uma reta de comprimento 10.
const LINE10: [[f32; 2]; 2] = [[0.0, 0.0], [10.0, 0.0]];

/// **O espaçamento decide quantos.** Numa reta de 10, um passo de 2 dá cinco.
///
/// O oráculo é a GEOMETRIA e não a contagem: as cinco pousam de dois em dois,
/// que é o número pedido — uma contagem certa com posições erradas passaria por
/// um `assert_eq!(len, 5)` sozinho.
#[test]
fn the_spacing_decides_how_many_and_they_land_that_far_apart() {
    let (pos, _) = walk(
        &LINE10,
        "Track",
        &[
            ("mode", MODE_SPACING),
            ("spacing", 2.0),
            ("align", 0.0),
            // ⚠️ O `count` fica no default (24) DE PROPÓSITO: se ele ainda
            // mandasse, este gate leria 24 e não 5.
        ],
    );
    assert_eq!(pos.len(), 5, "10 / 2 = 5 cópias");
    for (i, p) in pos.iter().enumerate() {
        let want = i as f32 * 2.0;
        assert!(
            (p[0] - want).abs() < 1e-3,
            "a cópia {i} pousa em x = {want}, não em {}",
            p[0]
        );
    }
}

/// **O modo `Number` é BYTE-IDÊNTICO ao nó que shipava** — o default reduz.
///
/// Não é uma promessa: as duas rotas são cozidas e comparadas com `to_bits()`.
/// A metade que carrega o peso é o `spacing` estar ARMADO num valor que mudaria
/// tudo se ele fosse lido; sem isso o gate ficaria verde sobre um param inerte.
#[test]
fn the_number_mode_is_byte_identical_to_the_node_that_shipped() {
    let base = walk(&LINE10, "Track", &[("count", 7.0)]);
    let armed = walk(
        &LINE10,
        "Track",
        &[("count", 7.0), ("mode", MODE_COUNT), ("spacing", 0.37)],
    );
    assert_eq!(base.0.len(), 7);
    for (a, b) in base.0.iter().zip(&armed.0) {
        assert_eq!(a[0].to_bits(), b[0].to_bits(), "x ao bit");
        assert_eq!(a[1].to_bits(), b[1].to_bits(), "y ao bit");
    }
    for (a, b) in base.1.iter().zip(&armed.1) {
        assert_eq!(a.to_bits(), b.to_bits(), "rot ao bit");
    }
}

/// **Um espaçamento maior que a curva devolve ZERO, não uma cópia.**
///
/// É o veredito do irmão (`if k_hi < k_lo { return Vec::new() }`) e é o honesto:
/// nada cabe. Arredondar para uma cópia seria o nó a inventar um encaixe.
#[test]
fn a_spacing_longer_than_the_curve_fits_nothing() {
    let (pos, _) = walk(
        &LINE10,
        "Track",
        &[("mode", MODE_SPACING), ("spacing", 25.0)],
    );
    assert!(pos.is_empty(), "nada cabe, e o nó emite vazio: {pos:?}");
}

/// **FLOOR: o entregue nunca é mais APERTADO que o pedido.**
///
/// ⚠️ É a metade que um `round` quebraria, e o caso que a separa é o RESTO: numa
/// reta de 10 com passo 3 cabem 3 (resto 1), e o vão real vira `10/3 = 3,33`.
/// Um `round` daria 3 aqui também — o discriminante é o passo 2,2 (`10/2,2 =
/// 4,54`): FLOOR dá 4 e vão 2,50 (≥ 2,2 ✔), ROUND daria 5 e vão 2,00 (< 2,2 ✘).
#[test]
fn the_gap_delivered_is_never_tighter_than_the_gap_asked() {
    for spacing in [2.2f32, 3.0, 0.7, 1.3] {
        let (pos, _) = walk(
            &LINE10,
            "Track",
            &[("mode", MODE_SPACING), ("spacing", spacing), ("align", 0.0)],
        );
        assert!(!pos.is_empty(), "spacing {spacing} deveria caber");
        let gap = 10.0 / pos.len() as f32;
        assert!(
            gap >= spacing - 1e-4,
            "pedido {spacing}, entregue {gap} com {} cópias — mais apertado que o pedido",
            pos.len()
        );
    }
}

/// **O piso do espaçamento é load-bearing**, e o que ele guarda é o clamp que
/// mentiria: sem ele `spacing = 0` pede uma contagem infinita e o teto devolve
/// `RECOMMENDED_MAX_ELEMENTS` em silêncio, com o slider a dizer zero.
#[test]
fn the_spacing_floor_keeps_a_zero_from_asking_for_everything() {
    assert_eq!(copies_that_fit(10.0, MIN_SPACING), 1000);
    // E o piso é aplicado na LEI, não só no slider: um documento pode carregar
    // um `spacing` menor (o param é f32 e um fio pode dirigi-lo, doc 58).
    let (pos, _) = walk(&LINE10, "Track", &[("mode", MODE_SPACING), ("spacing", 0.0)]);
    assert_eq!(
        pos.len(),
        1000,
        "o piso responde, e a resposta é finita e nomeada"
    );
}
