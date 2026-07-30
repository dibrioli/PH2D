//! **Sonda do ALCANCE DO NÓ** (plano 25 §6) — os dois números que decidem as barras dos gates.
//!
//! Rodar: `cargo test -p ph2d-vec-scene --test measure_node_reach -- --nocapture --ignored`

use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};

/// Um arco de 3 vértices suaves: `(0,0) → (1,1) → (2,0)`, subindo a 45° e descendo a 45°.
///
/// ⚠️ **As tangentes das PONTAS não podem ser paralelas**, e a 1ª fixture desta sonda era: com
/// handles horizontais nas duas pontas, TODO ponto de controle da cúbica que sobra tem `y = 0` —
/// nenhuma cúbica com aquelas tangentes alcança o ápice, e o refit degrada para a reta (medido:
/// desvio `1,0000`, igual ao da remoção crua). O fit estava certo; a fixture é que não continha o
/// fenômeno.
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

/// Amostra o caminho inteiro (todos os segmentos) em `n` pontos por segmento.
fn sample(p: &VecPath, n: usize) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    let segs = p.verts.len().saturating_sub(1);
    for s in 0..segs {
        for k in 0..=n {
            let t = k as f64 / n as f64;
            if let Some(q) = ph2d_vec_scene::point_on_segment(p, s, t) {
                out.push(q);
            }
        }
    }
    out
}

/// A maior distância de um ponto de `a` ao ponto mais próximo de `b` — o desvio de FORMA.
fn max_dev(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    a.iter().fold(0.0_f64, |m, p| {
        let d = b.iter().fold(f64::INFINITY, |acc, q| {
            acc.min((p[0] - q[0]).hypot(p[1] - q[1]))
        });
        m.max(d)
    })
}

#[test]
#[ignore = "sonda"]
fn what_deleting_the_middle_node_costs() {
    let before = arc3();
    let s0 = sample(&before, 64);

    let mut kept = before.clone();
    ph2d_vec_scene::dissolve_vertex(&mut kept.verts, 1, false);
    let mut naive = before.clone();
    naive.verts.remove(1);

    println!("\n  extensao do arco: x ∈ [0, 2], altura ~0,75");
    for (name, p) in [("PRESERVA (dissolve)", &kept), ("CRU (remove)", &naive)] {
        let s = sample(p, 64);
        println!(
            "  {name:<20} verts {} -> desvio maximo da forma original = {:.4}",
            p.verts.len(),
            max_dev(&s0, &s)
        );
    }
    println!();
}

#[test]
#[ignore = "sonda"]
fn what_reshaping_a_segment_does() {
    let mut p = arc3();
    let n0 = p.verts.len();
    let anchors0: Vec<[f64; 2]> = p.verts.iter().map(|v| v.anchor).collect();
    let t = 0.5;
    let at = ph2d_vec_scene::point_on_segment(&p, 0, t).expect("ponto");
    let delta = [0.3, -0.7];
    let ok = ph2d_vec_scene::reshape_segment(&mut p, 0, t, delta);
    let after = ph2d_vec_scene::point_on_segment(&p, 0, t).expect("ponto");
    let want = [at[0] + delta[0], at[1] + delta[1]];
    println!("\n  reshape_segment ok={ok}");
    println!("  ponto em t=0,5: {at:?} -> {after:?}   (queria {want:?})");
    println!(
        "  erro = {:.3e}   verts {} -> {}   ancoras iguais = {}",
        (after[0] - want[0]).hypot(after[1] - want[1]),
        n0,
        p.verts.len(),
        p.verts
            .iter()
            .zip(&anchors0)
            .all(|(v, a)| (v.anchor[0] - a[0]).abs() < 1e-15 && (v.anchor[1] - a[1]).abs() < 1e-15)
    );
    // E em vários `t`, porque a distribuição tem de ser exata em todos.
    let worst = (1..20).fold(0.0_f64, |m, k| {
        let t = f64::from(k) / 20.0;
        let mut q = arc3();
        let a = ph2d_vec_scene::point_on_segment(&q, 0, t).expect("ponto");
        ph2d_vec_scene::reshape_segment(&mut q, 0, t, delta);
        let b = ph2d_vec_scene::point_on_segment(&q, 0, t).expect("ponto");
        m.max((b[0] - a[0] - delta[0]).hypot(b[1] - a[1] - delta[1]))
    });
    println!("  pior erro sobre t ∈ [0.05, 0.95] = {worst:.3e}\n");
}
