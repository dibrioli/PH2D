//! Testes de `bubbles.rs` — arquivo irmao (teto de 700 LOC por arquivo, HR-18).
//!
//! Ligado por `#[path]` no modulo pai, entao `use super::*` continua valendo.

use super::*;
use crate::bubbles_ring::{cubic_at, extrema};
use crate::{VecVertex, VertexKind};

const A: [f64; 2] = [-2.0, -1.5];
const B: [f64; 2] = [2.0, 1.5];

/// Os defaults do catálogo (espelham `ShapeKind::defaults`).
fn all() -> Vec<(&'static str, VecPath)> {
    vec![
        (
            "speech_rect",
            speech_rect(A, B, 0.12, 0.30, 0.97, 0.26, 0.30),
        ),
        ("speech_oval", speech_oval(A, B, 0.30, 0.97, 13.0, 0.26)),
        ("thought", thought(A, B, 0.28, 0.95, 3.0, 11.0)),
        ("cloud", cloud(A, B, 11.0, 0.62, 0.0, 1.0)),
        ("burst", burst(A, B, 12.0, 0.565, 0.28, 0.55)),
        ("brace", brace(A, B, 1.0 / 12.0, 0.5, 0.5)),
    ]
}

/// A bbox VERDADEIRA (âncoras + extremos internos das cúbicas) de um path em MUNDO.
fn world_bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let mut eat = |vs: &[VecVertex], is_closed: bool| {
        let n = vs.len();
        for v in vs {
            for k in 0..2 {
                lo[k] = lo[k].min(v.anchor[k]);
                hi[k] = hi[k].max(v.anchor[k]);
            }
        }
        let segs = if is_closed { n } else { n.saturating_sub(1) };
        let mut roots = Vec::new();
        for i in 0..segs {
            let (x, y) = (&vs[i], &vs[(i + 1) % n]);
            for k in 0..2 {
                let (p0, p1, p2, p3) = (x.anchor[k], x.out_handle[k], y.in_handle[k], y.anchor[k]);
                roots.clear();
                extrema(p0, p1, p2, p3, &mut roots);
                for &t in &roots {
                    if t > 0.0 && t < 1.0 {
                        let val = cubic_at(p0, p1, p2, p3, t);
                        lo[k] = lo[k].min(val);
                        hi[k] = hi[k].max(val);
                    }
                }
            }
        }
    };
    eat(&p.verts, p.closed);
    for c in &p.subpaths {
        eat(&c.verts, c.closed);
    }
    (lo, hi)
}

/// Amostra o contorno numa polilinha (a curva, não as âncoras).
fn flatten(vs: &[VecVertex], is_closed: bool, steps: usize) -> Vec<[f64; 2]> {
    let n = vs.len();
    let segs = if is_closed { n } else { n - 1 };
    let mut out = Vec::with_capacity(segs * steps);
    for i in 0..segs {
        let (x, y) = (&vs[i], &vs[(i + 1) % n]);
        for s in 0..steps {
            let t = s as f64 / steps as f64;
            out.push([
                cubic_at(x.anchor[0], x.out_handle[0], y.in_handle[0], y.anchor[0], t),
                cubic_at(x.anchor[1], x.out_handle[1], y.in_handle[1], y.anchor[1], t),
            ]);
        }
    }
    out
}

/// Cruzamento PRÓPRIO entre dois segmentos não-adjacentes de uma polilinha fechada.
fn self_intersects(poly: &[[f64; 2]]) -> bool {
    const E: f64 = 1e-12;
    let m = poly.len();
    let cross = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let opposite = |x: f64, y: f64| (x > E && y < -E) || (x < -E && y > E);
    for i in 0..m {
        for j in (i + 1)..m {
            if j == i + 1 || (i == 0 && j == m - 1) {
                continue; // adjacentes: compartilham vértice, não contam
            }
            let (p1, p2) = (poly[i], poly[(i + 1) % m]);
            let (p3, p4) = (poly[j], poly[(j + 1) % m]);
            if opposite(cross(p3, p4, p1), cross(p3, p4, p2))
                && opposite(cross(p1, p2, p3), cross(p1, p2, p4))
            {
                return true;
            }
        }
    }
    false
}

/// **O detector é confiável** — senão o gate anti-auto-interseção seria decorativo. Um
/// "8" cruza; um quadrado, não.
#[test]
fn the_crossing_detector_actually_detects_a_crossing() {
    let bowtie = [[0.0, 0.0], [1.0, 1.0], [1.0, 0.0], [0.0, 1.0]];
    assert!(self_intersects(&bowtie), "o laco em 8 TEM de cruzar");
    let square = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    assert!(!self_intersects(&square), "o quadrado nao cruza");
}

/// **O contorno do balão é SIMPLES.** O bug da versão antiga: o rabo era inserido ANTES
/// do primeiro vértice da aresta de baixo, quando a caminhada do round-rect exige DEPOIS
/// — o preenchimento `NonZero` escondia, mas o traço mostrava o zigue-zague cruzado.
/// Aqui a interseção é impossível por construção (o rabo vive todo na faixa reservada,
/// `u ≥ 1−room`, e os flancos saem do bico para lados opostos do eixo) — e o gate
/// varre a grade de parâmetros para provar.
#[test]
fn the_speech_balloon_outline_never_crosses_itself() {
    for &tu in &[0.05, 0.3, 0.5, 0.72, 0.98] {
        for &tv in &[0.02, 0.28, 0.5, 0.8, 0.99] {
            for &room in &[0.05, 0.3, 0.6] {
                let r = speech_rect(A, B, 0.4, tu, tv, 0.26, room);
                assert!(
                    !self_intersects(&flatten(&r.verts, true, 12)),
                    "speech_rect cruza em tip=({tu}, {tv}) room={room}"
                );
                let o = speech_oval(A, B, tu, tv, 13.0, room);
                assert!(
                    !self_intersects(&flatten(&o.verts, true, 12)),
                    "speech_oval cruza em tip=({tu}, {tv}) room={room}"
                );
            }
        }
    }
}

/// **O rabo aponta para quem fala.** Em MUNDO (Y-para-cima), com o bico embaixo, o bico
/// é o ponto MAIS BAIXO do caminho — e sozinho lá embaixo, bem abaixo do corpo. Com o
/// bico em cima, ele é o mais ALTO: prova que o rabo SEGUE o parâmetro, e que o mundo não
/// está espelhado (era exatamente esse o defeito da versão antiga).
#[test]
fn the_speech_tail_tips_toward_the_speaker_never_at_the_sky() {
    let h = B[1] - A[1];
    for (name, low, high) in [
        (
            "speech_rect",
            speech_rect(A, B, 0.12, 0.30, 0.97, 0.26, 0.30),
            speech_rect(A, B, 0.12, 0.30, 0.03, 0.26, 0.30),
        ),
        (
            "speech_oval",
            speech_oval(A, B, 0.30, 0.97, 13.0, 0.26),
            speech_oval(A, B, 0.30, 0.03, 13.0, 0.26),
        ),
    ] {
        let mut ys: Vec<f64> = low.verts_all().map(|v| v.anchor[1]).collect();
        ys.sort_by(f64::total_cmp);
        assert!(
            ys[0] < ys[1] - 0.2 * h,
            "{name}: o bico ({}) tem de ficar SOZINHO abaixo do corpo (o proximo esta em {})",
            ys[0],
            ys[1]
        );
        let mut ys: Vec<f64> = high.verts_all().map(|v| v.anchor[1]).collect();
        ys.sort_by(f64::total_cmp);
        let n = ys.len();
        assert!(
            ys[n - 1] > ys[n - 2] + 0.2 * h,
            "{name}: com o bico para CIMA ele tem de ser o ponto mais alto"
        );
    }
}

/// **As bolhas do pensamento DESCEM.** A bolha 0 (a menor) nasce no bico, e a corrente
/// sobe de volta para a nuvem: em MUNDO, o `y` do centro cresce com o índice, e todas
/// ficam abaixo do centro do corpo.
#[test]
fn the_thought_bubbles_descend_away_from_the_cloud() {
    let p = thought(A, B, 0.28, 0.95, 4.0, 11.0);
    assert_eq!(p.subpaths.len(), 4, "quatro bolhas");
    let mid = |vs: &[VecVertex]| vs.iter().map(|v| v.anchor[1]).sum::<f64>() / vs.len() as f64;
    let body = mid(&p.verts);
    let mut prev = f64::MIN;
    for (i, c) in p.subpaths.iter().enumerate() {
        let y = mid(&c.verts);
        assert!(
            y < body,
            "bolha {i} (y = {y}) tem de ficar abaixo do corpo (y = {body})"
        );
        assert!(
            y > prev,
            "a corrente tem de SUBIR de volta ao corpo: bolha {i} em {y} nao esta acima de {prev}"
        );
        prev = y;
    }
    // …e a primeira é a MENOR (é ela que toca o pensador).
    let span = |vs: &[VecVertex]| {
        let (lo, hi) = vs.iter().fold((f64::MAX, f64::MIN), |(l, h), v| {
            (l.min(v.anchor[0]), h.max(v.anchor[0]))
        });
        hi - lo
    };
    assert!(
        span(&p.subpaths[0].verts) < span(&p.subpaths[3].verts),
        "a bolha do bico e a menor da corrente"
    );
}

/// **A forma É a caixa do gesto.** Medido na CURVA (âncoras *e* extremos internos das
/// Béziers — o casco de controle mentiria: ele estufa para fora da curva de verdade).
/// Sem isto o gizmo desalinha do desenho.
#[test]
fn every_bubble_fills_its_gesture_box_exactly() {
    for (name, p) in all() {
        let (lo, hi) = world_bbox(&p);
        for k in 0..2 {
            let (want_lo, want_hi) = (A[k].min(B[k]), A[k].max(B[k]));
            assert!(
                (lo[k] - want_lo).abs() < 1e-6 && (hi[k] - want_hi).abs() < 1e-6,
                "{name}: eixo {k} vai de {} a {} — a caixa e [{want_lo}, {want_hi}]",
                lo[k],
                hi[k]
            );
        }
    }
}

/// **A nuvem é G1 onde o algoritmo promete.** Com `valley > 0` todo join é TANGENTE: o
/// ponto de tangência é colinear com os dois centros, então a bolha e o vale
/// compartilham a reta tangente — não sobra NENHUMA cúspide no anel. Com `valley = 0` a
/// trilateração degenera na interseção círculo-círculo e sobra exatamente uma cúspide
/// por bolha (a escalopada clássica do OOXML).
#[test]
fn the_cloud_arc_joins_are_tangent_when_the_valley_is_open() {
    for &bumps in &[5.0, 8.0, 11.0, 17.0, 24.0] {
        let cusped = cloud(A, B, bumps, 0.62, 0.0, 1.0);
        let cusps = cusped
            .verts
            .iter()
            .filter(|v| v.kind == VertexKind::Corner)
            .count();
        assert_eq!(
            cusps, bumps as usize,
            "com valley = 0 sobra UMA cuspide por bolha (o join = intersecao circulo-circulo)"
        );

        let smooth = cloud(A, B, bumps, 0.62, 0.03, 1.0);
        assert_eq!(
            smooth
                .verts
                .iter()
                .filter(|v| v.kind == VertexKind::Corner)
                .count(),
            0,
            "com valley > 0 NENHUM join e cuspide — a nuvem inteira e G1"
        );
        for v in &smooth.verts {
            let i = [v.anchor[0] - v.in_handle[0], v.anchor[1] - v.in_handle[1]];
            let o = [v.out_handle[0] - v.anchor[0], v.out_handle[1] - v.anchor[1]];
            let (li, lo) = (i[0].hypot(i[1]), o[0].hypot(o[1]));
            assert!(li > 1e-9 && lo > 1e-9, "handle degenerado no anel da nuvem");
            let cross = (i[0] * o[1] - i[1] * o[0]) / (li * lo);
            let dot = (i[0] * o[0] + i[1] * o[1]) / (li * lo);
            assert!(
                cross.abs() < 1e-9 && dot > 0.0,
                "join nao-tangente em {:?}: cross = {cross}, dot = {dot}",
                v.anchor
            );
        }
    }
}

/// **Determinismo.** Cozinhar duas vezes tem de dar os MESMOS bytes: a nuvem e a explosão
/// tiram a irregularidade de tabelas fixas indexadas pelo vértice, não de RNG. Uma forma
/// que muda de silhueta a cada re-cook (cada tecla no painel, cada undo, cada reload) é
/// inutilizável.
#[test]
fn cooking_twice_produces_the_very_same_bytes() {
    for (name, p) in all() {
        let q = all()
            .into_iter()
            .find(|(n, _)| *n == name)
            .expect("a mesma forma")
            .1;
        let (x, y) = (
            postcard::to_allocvec(&p).expect("serializa"),
            postcard::to_allocvec(&q).expect("serializa"),
        );
        assert_eq!(x, y, "{name}: dois cozimentos, bytes diferentes");
        for v in p.verts_all() {
            assert!(
                v.anchor[0].is_finite()
                    && v.anchor[1].is_finite()
                    && v.in_handle[0].is_finite()
                    && v.out_handle[1].is_finite(),
                "{name}: NaN/inf na geometria"
            );
        }
    }
}

/// A explosão é um polígono ESTRELADO em torno do centro: o desvio angular máximo
/// (0,9 × 0,33 = 0,297 do passo) é menor que meio passo, então os ângulos nunca trocam de
/// ordem — e um raio monótono em θ não pode se auto-intersectar.
#[test]
fn the_burst_never_folds_over_itself() {
    for &spikes in &[5.0, 12.0, 19.0, 30.0] {
        let p = burst(A, B, spikes, 0.565, 0.6, 0.9);
        assert_eq!(p.verts.len(), 2 * spikes as usize, "duas ancoras por ponta");
        assert!(
            !self_intersects(&flatten(&p.verts, true, 1)),
            "a explosao de {spikes} pontas dobrou sobre si"
        );
    }
}

/// A chave é um TRAÇO (aberto), e o bico é uma cúspide de 180° de verdade: os dois arcos
/// chegam nele tangentes à horizontal e o sentido de percurso INVERTE. Se fosse suave
/// sairia um `S`, não um `{`.
#[test]
fn the_brace_beak_is_a_real_cusp_not_an_ess() {
    let p = brace(A, B, 1.0 / 12.0, 0.5, 0.5);
    assert!(!p.closed, "a chave e um traco, nao uma regiao");
    let beak = p
        .verts
        .iter()
        .min_by(|x, y| x.anchor[0].total_cmp(&y.anchor[0]))
        .expect("tem vertices");
    let i = [
        beak.anchor[0] - beak.in_handle[0],
        beak.anchor[1] - beak.in_handle[1],
    ];
    let o = [
        beak.out_handle[0] - beak.anchor[0],
        beak.out_handle[1] - beak.anchor[1],
    ];
    let dot = i[0] * o[0] + i[1] * o[1];
    assert!(
        dot < 0.0,
        "o bico tem de reverter o percurso (cuspide de 180 graus): dot = {dot}"
    );
    assert!(
        i[1].abs() < 1e-9 && o[1].abs() < 1e-9,
        "os dois arcos chegam ao bico TANGENTES a horizontal"
    );
}
