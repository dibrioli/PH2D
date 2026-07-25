//! Gates do [`super::knot_path`] — o entrelace celta.
//!
//! O corte por arco é o mesmo do Trim (com gates próprios); aqui prova-se o que é do Knot: neutro
//! byte-idêntico, caminho sem travessia fica inteiro, EXATAMENTE um vão por travessia (medido no
//! comprimento de arco removido), as fitas saem abertas, e o Swap move os vãos. A prova de que o
//! entrelace ALTERNA (parece tecido) é a folha de contacto visual (`tests/fx_look.rs`).

use super::{KnotSpec, knot_path};
use crate::arclen::arclen;
use crate::corner_live::segment;
use crate::effect::{FxCtx, FxEntry, PathEffect, run_stack};
use crate::{VecPath, VecVertex};

/// Um pentagrama `{5/2}` — 5 pontos ligados de dois em dois, 5 auto-interseções. O nó celta canônico.
fn pentagram() -> VecPath {
    const R: f64 = 72.0;
    VecPath {
        verts: (0..5)
            .map(|k| {
                let a = (90.0 + f64::from(k) * 144.0).to_radians();
                VecVertex::corner([R * a.cos(), R * a.sin()])
            })
            .collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// Um círculo em 4 cúbicas — NÃO se cruza (o controle "sem travessia").
fn circle() -> VecPath {
    const K: f64 = 0.552_284_749_830_793_4;
    const R: f64 = 60.0;
    let p = [[R, 0.0], [0.0, R], [-R, 0.0], [0.0, -R]];
    let t = [[0.0, K * R], [-K * R, 0.0], [0.0, -K * R], [K * R, 0.0]];
    VecPath {
        verts: (0..4)
            .map(|i| VecVertex {
                anchor: p[i],
                in_handle: [p[i][0] - t[i][0], p[i][1] - t[i][1]],
                out_handle: [p[i][0] + t[i][0], p[i][1] + t[i][1]],
                kind: crate::VertexKind::Smooth,
                corner_radius: 0.0,
            })
            .collect(),
        closed: true,
        ..VecPath::default()
    }
}

fn contour_arclen(verts: &[VecVertex], closed: bool) -> f64 {
    let n = verts.len();
    if n < 2 {
        return 0.0;
    }
    let sc = if closed { n } else { n - 1 };
    (0..sc).map(|i| arclen(&segment(verts, i, n))).sum()
}

fn path_arclen(p: &VecPath) -> f64 {
    let mut s = contour_arclen(&p.verts, p.closed);
    for c in &p.subpaths {
        s += contour_arclen(&c.verts, c.closed);
    }
    s
}

/// **Vão zero é um no-op** — a pilha SALTA o Knot neutro (Cow::Borrowed sobrevive).
#[test]
fn a_neutral_knot_is_skipped_by_the_stack() {
    let mut p = pentagram();
    p.effects = vec![FxEntry::new(PathEffect::Knot(KnotSpec {
        gap: 0.0,
        swap: false,
    }))];
    assert!(
        run_stack(&p, &p.effects).is_none(),
        "um Knot com vão 0 tem de ser saltado pela pilha"
    );
}

/// **Um caminho SEM travessia fica inteiro** — um círculo não se cruza, então o Knot não tem o que
/// tecer: sai UM contorno fechado, igual à entrada. A mutação que cortasse vãos sem travessia sangra.
#[test]
fn a_path_with_no_crossings_is_left_whole() {
    let circ = circle();
    let ctx = FxCtx::of(&circ);
    let out = knot_path(
        &circ,
        &KnotSpec {
            gap: 8.0,
            swap: false,
        },
        &ctx,
    );
    assert!(out.closed, "o círculo tem de sair fechado");
    assert!(
        out.subpaths.is_empty(),
        "sem travessia, não há fitas a separar"
    );
    assert_eq!(out.verts, circ.verts, "sem travessia, a geometria não muda");
}

/// **Exatamente UM vão por travessia** — o pentagrama tem 5 auto-interseções, e o comprimento de
/// arco removido tem de ser ~= 5 vãos. Menos = travessias perdidas; mais = fitas cortadas duas
/// vezes. A mutação que não corta vão nenhum deixa o comprimento intacto e sangra.
#[test]
fn a_knot_cuts_one_gap_per_crossing() {
    let star = pentagram();
    let ctx = FxCtx::of(&star);
    let gap_pct = 8.0;
    let out = knot_path(
        &star,
        &KnotSpec {
            gap: gap_pct,
            swap: false,
        },
        &ctx,
    );

    let before = path_arclen(&star);
    let after = path_arclen(&out);
    let removed = before - after;
    let gap_len = ctx.ref_size * (gap_pct / 100.0);
    let expected = 5.0 * gap_len; // 5 travessias, um vão cada
    assert!(
        (removed - expected).abs() < expected * 0.2,
        "removido {removed:.2}, esperado ~{expected:.2} (5 vãos de {gap_len:.2})"
    );
    // Todas as fitas saem ABERTAS: os vãos quebraram o laço.
    assert!(!out.closed, "a fita primária tem de sair aberta");
    assert!(
        out.subpaths.iter().all(|c| !c.closed),
        "toda fita tem de sair aberta"
    );
}

/// **O Swap move os vãos** — inverter quem passa por cima em todo cruzamento muda a geometria (os
/// vãos caem nas outras passagens). A mutação que ignora o `swap` deixa os dois iguais e sangra.
#[test]
fn swap_moves_the_gaps() {
    let star = pentagram();
    let ctx = FxCtx::of(&star);
    let a = knot_path(
        &star,
        &KnotSpec {
            gap: 8.0,
            swap: false,
        },
        &ctx,
    );
    let b = knot_path(
        &star,
        &KnotSpec {
            gap: 8.0,
            swap: true,
        },
        &ctx,
    );
    // Mesmo comprimento removido (5 vãos nos dois), mas em SÍTIOS diferentes ⇒ geometria diferente.
    assert!(
        a.verts != b.verts || a.subpaths.len() != b.subpaths.len() || {
            a.subpaths
                .iter()
                .zip(&b.subpaths)
                .any(|(x, y)| x.verts != y.verts)
        },
        "o Swap não mudou nada — os vãos não se moveram"
    );
}
