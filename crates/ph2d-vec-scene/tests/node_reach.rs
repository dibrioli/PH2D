//! Gates do **ALCANCE DO NÓ** (plano 25 §6) — as duas operações que compartilham o
//! `merged_segment_fit`: apagar um nó **preservando a forma**, e reformar um segmento **sem mexer
//! na topologia**.
//!
//! Os oráculos são de FORMA e de CONTAGEM, nunca da fórmula: *a curva que sobra passa por onde a
//! outra passava* e *nenhum vértice nasceu nem morreu*. Um gate que recomputasse a distribuição de
//! Bernstein para conferir a distribuição de Bernstein seria o espelho sempre-verde.

use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};

/// Um arco de 3 vértices: `(0,0) → (1,1) → (2,0)`, subindo e descendo a 45°.
///
/// ⚠️ **As tangentes das PONTAS não podem ser paralelas.** Com handles horizontais nas duas, TODO
/// ponto de controle da cúbica que sobra tem `y = 0` — nenhuma cúbica com aquelas tangentes
/// alcança o ápice, e o refit degrada honestamente para a reta. MEDIDO na 1ª versão desta
/// fixture: desvio `1,0000`, **idêntico ao da remoção crua**, e a mutação não sangraria. É o
/// limite real da operação (o Illustrator tem o mesmo), e não um defeito a consertar.
fn arc3() -> VecPath {
    let v = |a: [f64; 2], i: [f64; 2], o: [f64; 2]| VecVertex {
        anchor: a,
        in_handle: i,
        out_handle: o,
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    };
    VecPath {
        verts: vec![
            v([0.0, 0.0], [-0.55, -0.55], [0.55, 0.55]),
            v([1.0, 1.0], [0.6, 1.0], [1.4, 1.0]),
            v([2.0, 0.0], [1.45, 0.55], [2.55, -0.55]),
        ],
        closed: false,
        ..VecPath::default()
    }
}

fn sample(p: &VecPath, n: usize) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for s in 0..p.verts.len().saturating_sub(1) {
        for k in 0..=n {
            if let Some(q) = ph2d_vec_scene::point_on_segment(p, s, k as f64 / n as f64) {
                out.push(q);
            }
        }
    }
    out
}

/// A maior distância de um ponto de `a` ao ponto mais próximo de `b` — o desvio de FORMA, que é
/// o que o olho vê.
fn max_dev(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    a.iter().fold(0.0_f64, |m, p| {
        m.max(b.iter().fold(f64::INFINITY, |acc, q| {
            acc.min((p[0] - q[0]).hypot(p[1] - q[1]))
        }))
    })
}

/// **Apagar um nó PRESERVA a curva.** Era a operação de nó mais usada em qualquer app, e aqui era
/// um `verts.remove` cru: a curva morria com o ponto.
///
/// ⚠️ **A barra sai da MEDIÇÃO, com o fosso ao lado:** o arco mede 2 de largura por ~0,75 de
/// altura; o refit desvia **0,0799** (11% da altura) e a remoção crua **0,5875** (78% — a curva
/// vira quase a corda). A barra de `0,15` fica entre os dois com folga de 2× para cada lado.
#[test]
fn deleting_a_node_keeps_the_curve_where_it_was() {
    let before = arc3();
    let s0 = sample(&before, 64);
    let mut kept = before.clone();
    assert!(ph2d_vec_scene::dissolve_vertex(&mut kept.verts, 1, false));
    assert_eq!(kept.verts.len(), 2, "o no' do meio nao saiu");
    let dev = max_dev(&s0, &sample(&kept, 64));
    assert!(
        dev < 0.15,
        "a curva desviou {dev:.4} ao perder o no' do meio -- a remocao crua desvia 0,5875, e e' \
         exatamente isso que o refit existe para nao fazer"
    );
    // As duas âncoras que sobram não se mexem: apagar um nó não move os vizinhos.
    for (i, a) in [[0.0, 0.0], [2.0, 0.0]].into_iter().enumerate() {
        assert!(
            (kept.verts[i].anchor[0] - a[0]).abs() < 1e-12
                && (kept.verts[i].anchor[1] - a[1]).abs() < 1e-12,
            "a ancora {i} andou: {:?}",
            kept.verts[i].anchor
        );
    }
}

/// **A ponta de um contorno ABERTO sai sem refit**, e é a resposta honesta: não há `prev` e `next`
/// para costurar, e a curva fica genuinamente mais curta. Inventar um refit ali moveria a ponta
/// que o artista acabou de escolher apagar.
#[test]
fn deleting_an_endpoint_just_shortens_the_path() {
    let mut p = arc3();
    let second = p.verts[1].anchor;
    assert!(ph2d_vec_scene::dissolve_vertex(&mut p.verts, 0, false));
    assert_eq!(p.verts.len(), 2);
    assert!(
        (p.verts[0].anchor[0] - second[0]).abs() < 1e-12,
        "a ponta saiu e o caminho nao comeca no 2o vertice"
    );
}

/// **Reformar um segmento leva o ponto agarrado ao dedo, EXATAMENTE** — a distribuição é a solução
/// de norma mínima de `B₁ΔP₁ + B₂ΔP₂ = delta`, e uma cúbica é linear nos pontos de controle, então
/// não há aproximação nenhuma. MEDIDO: pior erro `2,0e-16` sobre `t ∈ [0.05, 0.95]`.
#[test]
fn reshaping_a_segment_lands_the_grabbed_point_on_the_finger() {
    let delta = [0.3, -0.7];
    let worst = (1..20).fold(0.0_f64, |m, k| {
        let t = f64::from(k) / 20.0;
        let mut p = arc3();
        let a = ph2d_vec_scene::point_on_segment(&p, 0, t).expect("ponto");
        assert!(ph2d_vec_scene::reshape_segment(&mut p, 0, t, delta));
        let b = ph2d_vec_scene::point_on_segment(&p, 0, t).expect("ponto");
        m.max((b[0] - a[0] - delta[0]).hypot(b[1] - a[1] - delta[1]))
    });
    assert!(
        worst < 1e-12,
        "o ponto agarrado nao seguiu o dedo: erro {worst:.3e}"
    );
}

/// **A topologia NÃO muda** — é o que separa esta operação da inserção que ela substituiu. Nenhum
/// vértice nasce, nenhum morre, e as duas âncoras do segmento ficam onde estão: quem se move são
/// só os dois handles.
#[test]
fn reshaping_a_segment_never_touches_the_topology() {
    let mut p = arc3();
    let n0 = p.verts.len();
    let anchors: Vec<[f64; 2]> = p.verts.iter().map(|v| v.anchor).collect();
    assert!(ph2d_vec_scene::reshape_segment(&mut p, 0, 0.5, [0.3, -0.7]));
    assert_eq!(p.verts.len(), n0, "a reforma mudou a contagem de vertices");
    for (v, a) in p.verts.iter().zip(&anchors) {
        assert!(
            (v.anchor[0] - a[0]).abs() < 1e-15 && (v.anchor[1] - a[1]).abs() < 1e-15,
            "uma ancora andou: {:?} -> {:?}",
            a,
            v.anchor
        );
    }
    // E o OUTRO segmento não foi tocado: reformar um trecho não reforma o vizinho.
    let mut q = arc3();
    let far = ph2d_vec_scene::point_on_segment(&q, 1, 0.5).expect("ponto");
    ph2d_vec_scene::reshape_segment(&mut q, 0, 0.5, [0.3, -0.7]);
    let far2 = ph2d_vec_scene::point_on_segment(&q, 1, 0.5).expect("ponto");
    assert!(
        (far[0] - far2[0]).hypot(far[1] - far2[1]) < 1e-12,
        "reformar o segmento 0 mexeu no 1"
    );
}
