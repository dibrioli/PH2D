//! Gates da **SOLDA das pontas** — o buraco de quina que a auditoria de 2026-07-21 nomeou.
//!
//! A arte real é feita de traços SEPARADOS que se encontram nas quinas. O que o artista vê é
//! o CORPO pintado (largura 0,26) e ele se sobrepõe folgado; o que o `colorize` rasteriza é o
//! **EIXO** (raio 0, BUGS #14/#15) e entre dois eixos sobra um buraco de 0,02-0,04 doc. A cor
//! escapa por um buraco que **não existe na tela** — e nenhum knob deveria ser preciso para
//! tapar o que o artista já pintou por cima.
//!
//! Irmão do `lib_tests.rs` pelo teto de LOC (700).

use super::{ColorRegion, Scribble, colorize};
use ph2d_core::Vec2;

/// O tremor determinístico do smoke (`flip_colorize_smoke.rs::hand`), reproduzido ao bit —
/// é ele que descola as pontas das quinas, e o fenômeno **não existe** numa reta perfeita.
pub(crate) fn hand(pts: &[Vec2], seed: usize) -> Vec<Vec2> {
    let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    pts.iter()
        .enumerate()
        .map(|(i, p)| Vec2::new(p.x + h(i + seed) * 0.05, p.y + h(i + seed + 91) * 0.05))
        .collect()
}

pub(crate) fn seg(a: Vec2, b: Vec2, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        })
        .collect()
}

/// **A arte do smoke**: a caixa `[-4,4] × [-2.5,2.5]` desenhada como **QUATRO traços
/// separados** + o divisor off-center com o vão deliberado, todos com a meia-largura real
/// (0,13 — o que `boundaries()` entrega). É a fixture que CONTÉM o fenômeno: a fixture
/// `boxed_with_divider` do `lib_tests.rs` usa uma polilinha FECHADA e por isso não tem quina
/// nenhuma para vazar ([[feedback_a_fixture_only_proves_what_it_contains]]).
pub(crate) fn smoke_art() -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5), 0usize, 24usize),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5), 7, 24),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5), 13, 24),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5), 29, 24),
        (Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6), 41, 41),
        (Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5), 53, 53),
    ]
    .into_iter()
    .map(|(a, b, s, n)| {
        let pts = hand(&seg(a, b, n), s);
        let m = pts.len();
        (pts, vec![0.13; m], false)
    })
    .collect()
}

pub(crate) fn smoke_scribbles() -> Vec<Scribble> {
    vec![
        Scribble {
            label: 0,
            points: seg(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8),
            width: 0.15,
        },
        Scribble {
            label: 1,
            points: seg(Vec2::new(2.6, -1.5), Vec2::new(2.6, 1.5), 8),
            width: 0.15,
        },
    ]
}

/// O ponto mais longe da origem em qualquer anel de qualquer região (o "quão fora" da caixa).
fn worst_outside(regions: &[ColorRegion], half_w: f32, half_h: f32) -> (f32, Vec2) {
    let mut worst = 0.0f32;
    let mut at = Vec2::new(0.0, 0.0);
    for p in regions
        .iter()
        .flat_map(|r| std::iter::once(&r.fill.outer).chain(r.fill.holes.iter()))
        .flat_map(|ring| ring.iter())
    {
        let d = (p.x.abs() - half_w).max(p.y.abs() - half_h);
        if d > worst {
            worst = d;
            at = *p;
        }
    }
    (worst, at)
}

/// 🔴 **A cor PARA na caixa, mesmo no Bleed default** (auditoria 2026-07-21, item 5; e o
/// report do Enio *"nada tira a extrapolação"*).
///
/// A quina da caixa é o encontro de DOIS traços cujas pontas **não coincidem** (tremor de
/// mão): 0,023 e 0,040 doc de vão entre os EIXOS. O corpo pintado dos dois tem meia-largura
/// 0,13 cada ⇒ eles se **sobrepõem folgado na tela**, e o artista vê uma quina FECHADA. Como
/// o raster da parede é o eixo (raio 0), sobrava um buraco de 3,7-6,5 px a precisão 160 e a
/// cor escorria para a moldura de fora.
///
/// ⚠️ O selo do `Bleed 0` (trapped-ball) **mascarava** isto — com a bola ligada não escapa. O
/// gate roda no **default** (`trap_px = 0`), que é onde o artista está.
///
/// Oráculo de APARÊNCIA: nenhum vértice de anel algum além da moldura desenhada. O gate mede
/// o próprio vão antes de afirmar (controle positivo — a fixture tem de conter o fenômeno).
///
/// Mutação que sangra: não soldar as pontas (a moldura externa volta a ser pintada).
#[test]
fn the_colour_stays_inside_a_box_whose_corners_are_separate_strokes() {
    let strokes = smoke_art();

    // ── Controle positivo: a fixture CONTÉM o buraco de quina. ──
    // O vão entre a ponta de um traço e a ponta do vizinho, em cada uma das 4 quinas.
    let mut corner_gaps: Vec<f32> = Vec::new();
    for i in 0..4 {
        let end = *strokes[i].0.last().expect("ponta");
        let start = strokes[(i + 1) % 4].0[0];
        corner_gaps.push((end - start).length());
    }
    let (min_gap, max_gap) = corner_gaps
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), &g| (l.min(g), h.max(g)));
    assert!(
        min_gap > 0.0 && max_gap < 0.26,
        "controle positivo: as quinas têm de estar DESCOLADAS (vão {corner_gaps:?}) e ainda \
         assim cobertas pelo corpo pintado (2 x 0,13) — senão a fixture não contém o fenômeno"
    );

    for precision in [80.0f32, 160.0, 320.0] {
        let regions = colorize(&strokes, &smoke_scribbles(), precision, 0.0);
        // Controle positivo #2: houve tinta. "Não vaza" fica verde com a tela vazia.
        assert_eq!(
            regions.len(),
            2,
            "precisão {precision}: as duas cores têm de existir"
        );
        let painted: f32 = regions
            .iter()
            .map(|r| ph2d_flip_fill::signed_area(&r.fill.outer).abs())
            .sum();
        assert!(
            painted > 20.0,
            "precisão {precision}: controle positivo — a caixa tem 40 unidades² e a cor tem \
             de encher a maior parte ({painted:.1})"
        );
        // A moldura desenhada, com a folga do tremor (±0,025) + a sobreposição de ~2 px que
        // a borda faz por cima do eixo (o zíper do 6º smoke).
        let slack = 0.025 + 3.0 / precision;
        let (out, at) = worst_outside(&regions, 4.0 + slack, 2.5 + slack);
        assert!(
            out <= 0.0,
            "precisão {precision}: a cor escapou {out:.3} doc além da moldura (em {at:?}) — \
             o buraco de quina entre traços separados"
        );
    }
}
