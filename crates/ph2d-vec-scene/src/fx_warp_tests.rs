//! Gates do Pucker & Bloat. Um botão só, então o que há a provar é que ele faz o que diz — e
//! sobretudo que **não é uma escala**, que foi o defeito da 1ª versão.

use super::*;
use crate::VecPath;

/// Um quadrado de lado 40 na origem — de propósito NÃO centrado no mundo, para os gates
/// distinguirem "o centro da forma" de "a origem".
fn square() -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

fn ctx() -> FxCtx {
    FxCtx::of(&square())
}

/// **O neutro é no-op byte-idêntico** — o que a pilha exige de todo efeito.
#[test]
fn the_neutral_point_is_a_byte_identical_no_op() {
    let p = square();
    assert_eq!(
        bloat_contour(&p.verts, true, &BloatSpec::default(), &ctx()).0,
        p.verts
    );
}

/// **Âncoras e alças movem-se em sentidos OPOSTOS** — é isso, e só isso, que o separa de uma
/// escala.
///
/// ⚠️ A 1ª versão escalava âncoras e alças pelo MESMO fator, e o gate media exatamente isso:
/// verde sobre uma escala uniforme, que é o gizmo e não um efeito (Enio, 2026-07-18:
/// *"só aumenta e reduz a escala do objeto"*). O oráculo tem de comparar os DOIS.
#[test]
fn the_bloat_moves_anchors_and_handles_in_opposite_directions() {
    let c = ctx();
    // Um vértice com alça DESTACADA da âncora, senão não há como ver os dois fatores.
    let v = VecVertex {
        anchor: [c.center[0] + 10.0, c.center[1]],
        in_handle: [c.center[0] + 10.0, c.center[1] - 4.0],
        out_handle: [c.center[0] + 10.0, c.center[1] + 4.0],
        kind: crate::VertexKind::Smooth,
        corner_radius: 0.0,
    };
    let r = |p: [f64; 2]| (p[0] - c.center[0]).hypot(p[1] - c.center[1]);
    let (a0, h0) = (r(v.anchor), r(v.out_handle));

    let out = bloat_contour(
        std::slice::from_ref(&v),
        false,
        &BloatSpec { amount: 50.0 },
        &c,
    )
    .0;
    let (a1, h1) = (r(out[0].anchor), r(out[0].out_handle));
    assert!(
        a1 < a0 && h1 > h0,
        "bloat: a âncora tem de ir para DENTRO ({a0} -> {a1}) e a alça para FORA ({h0} -> {h1}). \
         Os dois no mesmo sentido são uma escala."
    );

    let out = bloat_contour(&[v], false, &BloatSpec { amount: -50.0 }, &c).0;
    let (a2, h2) = (r(out[0].anchor), r(out[0].out_handle));
    assert!(
        a2 > a0 && h2 < h0,
        "pucker: o inverso — âncora para fora ({a0} -> {a2}), alça para dentro ({h0} -> {h2})"
    );
}

/// **O raio de quina segue a ÂNCORA**, não a alça: ele é um comprimento ancorado nela, e os dois
/// fatores divergem.
#[test]
fn the_bloat_scales_the_corner_radius_with_the_anchor() {
    let c = ctx();
    let mut v = VecVertex::corner([c.center[0] + 20.0, c.center[1]]);
    v.corner_radius = 6.0;
    let out = bloat_contour(&[v], false, &BloatSpec { amount: -100.0 }, &c).0;
    assert!(
        (out[0].corner_radius - 12.0).abs() < 1e-9,
        "com as âncoras a duplicar, o raio devia duplicar; deu {}",
        out[0].corner_radius
    );
}

/// **Deformar não abre nem fecha uma forma** — o `closed` atravessa intacto.
#[test]
fn the_contour_keeps_its_closedness() {
    let p = square();
    let c = ctx();
    assert!(bloat_contour(&p.verts, true, &BloatSpec { amount: 40.0 }, &c).1);
    assert!(!bloat_contour(&p.verts, false, &BloatSpec { amount: 40.0 }, &c).1);
}
