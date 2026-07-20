//! SONDA (`--ignored`): varre o Offset da rosquinha num `d` FINO à procura do panic
//! que o Enio vê arrastando o slider — o app visita um contínuo de `d`, e a fase em
//! que o contorno interno tangencia o externo é onde um sweep pode degenerar.
//!
//! Não é um gate: é a ferramenta que decide ONDE o gate deve nascer.

use ph2d_vec_boolean::offset_path;
use ph2d_vec_scene::{Contour, FillRule, LineJoin, OffsetSide, VecPath, VecVertex};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex::corner([x, y])
}

/// A rosquinha DO SMOKE (`build_smoke_expand`): retângulo 2.8..5.2 × -1.2..1.2 com
/// furo quadrado de lado 1.4 centrado em (4, 0) — parede 0.5. É a forma que o Enio
/// arrasta; a sonda tem de conter o fenômeno.
fn smoke_donut() -> VecPath {
    let hole = 0.7;
    VecPath {
        verts: vec![v(2.8, -1.2), v(5.2, -1.2), v(5.2, 1.2), v(2.8, 1.2)],
        closed: true,
        subpaths: vec![Contour::new_closed(vec![
            v(4.0 - hole, -hole),
            v(4.0 + hole, -hole),
            v(4.0 + hole, hole),
            v(4.0 - hole, hole),
        ])],
        fill_rule: FillRule::EvenOdd,
        ..VecPath::default()
    }
}

#[test]
#[ignore = "sonda de diagnóstico, não gate"]
fn fine_sweep_the_smoke_donut_for_panics() {
    let joins = [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel];
    let sides = [OffsetSide::Both, OffsetSide::Inner, OffsetSide::Outer];
    let mut panics = 0u32;
    let mut first: Option<(f64, LineJoin, OffsetSide)> = None;
    // -1.5..1.5 em passos de 1e-3 = 3000 valores × 9 combinações.
    for k in -1500..=1500i32 {
        let d = f64::from(k) * 1e-3;
        for join in joins {
            for side in sides {
                let src = smoke_donut();
                let r = std::panic::catch_unwind(move || {
                    let out = offset_path(&src, d, join, side);
                    // O que a cena consome: toda coordenada do resultado tem de ser finita.
                    for p in &out {
                        for vt in p.verts_all() {
                            let a = vt.anchor;
                            assert!(
                                a[0].is_finite() && a[1].is_finite(),
                                "âncora não-finita a d={d} {join:?} {side:?}: {a:?}"
                            );
                        }
                    }
                });
                if r.is_err() {
                    panics += 1;
                    if first.is_none() {
                        first = Some((d, join, side));
                    }
                }
            }
        }
    }
    assert_eq!(panics, 0, "{panics} panics; primeiro em {first:?}");
}
