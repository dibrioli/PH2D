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

/// Ajuste por MÍNIMOS QUADRADOS das duas alças (Schneider `GenerateBezier`): mesmas pontas,
/// mesmas direções de tangente, mas os comprimentos saem de amostras da curva ORIGINAL inteira
/// em vez de um único ponto de passagem.
fn lsq_fit(prev: &VecVertex, mid: &VecVertex, next: &VecVertex) -> ([f64; 2], [f64; 2]) {
    let p0 = prev.anchor;
    let p3 = next.anchor;
    let unit = |v: [f64; 2]| {
        let l = v[0].hypot(v[1]);
        if l > 1e-12 {
            [v[0] / l, v[1] / l]
        } else {
            [0.0, 0.0]
        }
    };
    let t1 = unit([prev.out_handle[0] - p0[0], prev.out_handle[1] - p0[1]]);
    let t2 = unit([next.in_handle[0] - p3[0], next.in_handle[1] - p3[1]]);
    // Amostras da curva original (dois segmentos), com parametrização por comprimento de corda.
    let cub = |c: &[[f64; 2]; 4], t: f64| {
        let m = 1.0 - t;
        [0, 1].map(|k| {
            m * m * m * c[0][k]
                + 3.0 * m * m * t * c[1][k]
                + 3.0 * m * t * t * c[2][k]
                + t * t * t * c[3][k]
        })
    };
    let s1 = [p0, prev.out_handle, mid.in_handle, mid.anchor];
    let s2 = [mid.anchor, mid.out_handle, next.in_handle, p3];
    let n = 32;
    let mut pts: Vec<[f64; 2]> = Vec::new();
    for k in 0..=n {
        pts.push(cub(&s1, f64::from(k) / f64::from(n)));
    }
    for k in 1..=n {
        pts.push(cub(&s2, f64::from(k) / f64::from(n)));
    }
    // Comprimento de corda acumulado -> u em [0,1]
    let mut acc = vec![0.0_f64];
    for w in pts.windows(2) {
        let d = (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
        acc.push(acc.last().unwrap() + d);
    }
    let total = *acc.last().unwrap();
    let us: Vec<f64> = acc
        .iter()
        .map(|a| if total > 0.0 { a / total } else { 0.0 })
        .collect();
    // Normal equations 2x2 sobre (a, b): P(u) = B0 p0 + B1 (p0 + a t1) + B2 (p3 + b t2) + B3 p3
    let (mut c00, mut c01, mut c11, mut x0, mut x1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (p, &u) in pts.iter().zip(&us) {
        let m = 1.0 - u;
        let (b0, b1, b2, b3) = (m * m * m, 3.0 * m * m * u, 3.0 * m * u * u, u * u * u);
        let a1 = [b1 * t1[0], b1 * t1[1]];
        let a2 = [b2 * t2[0], b2 * t2[1]];
        let base = [
            (b0 + b1) * p0[0] + (b2 + b3) * p3[0],
            (b0 + b1) * p0[1] + (b2 + b3) * p3[1],
        ];
        let r = [p[0] - base[0], p[1] - base[1]];
        c00 += a1[0] * a1[0] + a1[1] * a1[1];
        c01 += a1[0] * a2[0] + a1[1] * a2[1];
        c11 += a2[0] * a2[0] + a2[1] * a2[1];
        x0 += a1[0] * r[0] + a1[1] * r[1];
        x1 += a2[0] * r[0] + a2[1] * r[1];
    }
    let det = c00 * c11 - c01 * c01;
    let chord = (p3[0] - p0[0]).hypot(p3[1] - p0[1]);
    let (a, b) = if det.abs() < 1e-12 {
        (chord / 3.0, chord / 3.0)
    } else {
        ((x0 * c11 - c01 * x1) / det, (c00 * x1 - x0 * c01) / det)
    };
    let a = a.clamp(0.0, 3.0 * chord);
    let b = b.clamp(0.0, 3.0 * chord);
    (
        [p0[0] + a * t1[0], p0[1] + a * t1[1]],
        [p3[0] + b * t2[0], p3[1] + b * t2[1]],
    )
}

/// O mesmo LSQ, mas com **reparametrização de Newton** entre as resoluções (Schneider 1990): o
/// `u` de cada amostra é re-encontrado na curva ajustada antes de re-resolver as duas alças.
fn schneider_fit(
    prev: &VecVertex,
    mid: &VecVertex,
    next: &VecVertex,
    iters: usize,
) -> ([f64; 2], [f64; 2]) {
    let p0 = prev.anchor;
    let p3 = next.anchor;
    let unit = |v: [f64; 2]| {
        let l = v[0].hypot(v[1]);
        if l > 1e-12 {
            [v[0] / l, v[1] / l]
        } else {
            [0.0, 0.0]
        }
    };
    let t1 = unit([prev.out_handle[0] - p0[0], prev.out_handle[1] - p0[1]]);
    let t2 = unit([next.in_handle[0] - p3[0], next.in_handle[1] - p3[1]]);
    let cub = |c: &[[f64; 2]; 4], t: f64| {
        let m = 1.0 - t;
        [0, 1].map(|k| {
            m * m * m * c[0][k]
                + 3.0 * m * m * t * c[1][k]
                + 3.0 * m * t * t * c[2][k]
                + t * t * t * c[3][k]
        })
    };
    let s1 = [p0, prev.out_handle, mid.in_handle, mid.anchor];
    let s2 = [mid.anchor, mid.out_handle, next.in_handle, p3];
    let n = 32;
    let mut pts: Vec<[f64; 2]> = Vec::new();
    for k in 0..=n {
        pts.push(cub(&s1, f64::from(k) / f64::from(n)));
    }
    for k in 1..=n {
        pts.push(cub(&s2, f64::from(k) / f64::from(n)));
    }
    let mut acc = vec![0.0_f64];
    for w in pts.windows(2) {
        let d = (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
        acc.push(acc.last().unwrap() + d);
    }
    let total = *acc.last().unwrap();
    let mut us: Vec<f64> = acc
        .iter()
        .map(|a| if total > 0.0 { a / total } else { 0.0 })
        .collect();
    let chord = (p3[0] - p0[0]).hypot(p3[1] - p0[1]);
    let (mut a, mut b) = (chord / 3.0, chord / 3.0);
    for _ in 0..iters.max(1) {
        let (mut c00, mut c01, mut c11, mut x0, mut x1) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for (p, &u) in pts.iter().zip(&us) {
            let m = 1.0 - u;
            let (b0, b1, b2, b3) = (m * m * m, 3.0 * m * m * u, 3.0 * m * u * u, u * u * u);
            let a1 = [b1 * t1[0], b1 * t1[1]];
            let a2 = [b2 * t2[0], b2 * t2[1]];
            let base = [
                (b0 + b1) * p0[0] + (b2 + b3) * p3[0],
                (b0 + b1) * p0[1] + (b2 + b3) * p3[1],
            ];
            let r = [p[0] - base[0], p[1] - base[1]];
            c00 += a1[0] * a1[0] + a1[1] * a1[1];
            c01 += a1[0] * a2[0] + a1[1] * a2[1];
            c11 += a2[0] * a2[0] + a2[1] * a2[1];
            x0 += a1[0] * r[0] + a1[1] * r[1];
            x1 += a2[0] * r[0] + a2[1] * r[1];
        }
        let det = c00 * c11 - c01 * c01;
        if det.abs() > 1e-12 {
            a = ((x0 * c11 - c01 * x1) / det).clamp(0.0, 3.0 * chord);
            b = ((c00 * x1 - x0 * c01) / det).clamp(0.0, 3.0 * chord);
        }
        // Reparametriza: para cada amostra, o `u` do ponto mais próximo na curva ajustada.
        let c = [
            p0,
            [p0[0] + a * t1[0], p0[1] + a * t1[1]],
            [p3[0] + b * t2[0], p3[1] + b * t2[1]],
            p3,
        ];
        for (p, u) in pts.iter().zip(us.iter_mut()) {
            let mut best = (*u, f64::INFINITY);
            for k in 0..=200 {
                let v = f64::from(k) / 200.0;
                let q = cub(&c, v);
                let d = (q[0] - p[0]).hypot(q[1] - p[1]);
                if d < best.1 {
                    best = (v, d);
                }
            }
            *u = best.0;
        }
    }
    (
        [p0[0] + a * t1[0], p0[1] + a * t1[1]],
        [p3[0] + b * t2[0], p3[1] + b * t2[1]],
    )
}

/// O PISO TEÓRICO: uma cúbica entre as duas âncoras com `P1` e `P2` **completamente livres** (4
/// graus de liberdade em vez de 2) — as tangentes das pontas deixam de ser preservadas.
fn free_fit(
    prev: &VecVertex,
    mid: &VecVertex,
    next: &VecVertex,
    iters: usize,
) -> ([f64; 2], [f64; 2]) {
    let p0 = prev.anchor;
    let p3 = next.anchor;
    let cub = |c: &[[f64; 2]; 4], t: f64| {
        let m = 1.0 - t;
        [0, 1].map(|k| {
            m * m * m * c[0][k]
                + 3.0 * m * m * t * c[1][k]
                + 3.0 * m * t * t * c[2][k]
                + t * t * t * c[3][k]
        })
    };
    let s1 = [p0, prev.out_handle, mid.in_handle, mid.anchor];
    let s2 = [mid.anchor, mid.out_handle, next.in_handle, p3];
    let n = 32;
    let mut pts: Vec<[f64; 2]> = Vec::new();
    for k in 0..=n {
        pts.push(cub(&s1, f64::from(k) / f64::from(n)));
    }
    for k in 1..=n {
        pts.push(cub(&s2, f64::from(k) / f64::from(n)));
    }
    let mut acc = vec![0.0_f64];
    for w in pts.windows(2) {
        let d = (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
        acc.push(acc.last().unwrap() + d);
    }
    let total = *acc.last().unwrap();
    let mut us: Vec<f64> = acc.iter().map(|a| a / total).collect();
    let (mut p1, mut p2) = (p0, p3);
    for _ in 0..iters {
        // Normal equations 2x2 (a matriz é a mesma para x e y).
        let (mut c00, mut c01, mut c11) = (0.0, 0.0, 0.0);
        let (mut rx, mut ry) = ([0.0, 0.0], [0.0, 0.0]);
        for (p, &u) in pts.iter().zip(&us) {
            let m = 1.0 - u;
            let (b0, b1, b2, b3) = (m * m * m, 3.0 * m * m * u, 3.0 * m * u * u, u * u * u);
            c00 += b1 * b1;
            c01 += b1 * b2;
            c11 += b2 * b2;
            let base = [b0 * p0[0] + b3 * p3[0], b0 * p0[1] + b3 * p3[1]];
            let r = [p[0] - base[0], p[1] - base[1]];
            rx[0] += b1 * r[0];
            rx[1] += b2 * r[0];
            ry[0] += b1 * r[1];
            ry[1] += b2 * r[1];
        }
        let det = c00 * c11 - c01 * c01;
        if det.abs() > 1e-12 {
            p1 = [
                (rx[0] * c11 - c01 * rx[1]) / det,
                (ry[0] * c11 - c01 * ry[1]) / det,
            ];
            p2 = [
                (c00 * rx[1] - c01 * rx[0]) / det,
                (c00 * ry[1] - c01 * ry[0]) / det,
            ];
        }
        let c = [p0, p1, p2, p3];
        for (p, u) in pts.iter().zip(us.iter_mut()) {
            let mut best = (*u, f64::INFINITY);
            for k in 0..=200 {
                let v = f64::from(k) / 200.0;
                let q = cub(&c, v);
                let d = (q[0] - p[0]).hypot(q[1] - p[1]);
                if d < best.1 {
                    best = (v, d);
                }
            }
            *u = best.0;
        }
    }
    (p1, p2)
}

#[test]
#[ignore = "sonda"]
fn how_much_a_deleted_node_costs_and_what_it_would_cost_to_do_better() {
    let before = arc3();
    let s0 = sample(&before, 128);
    let dev = |p: &VecPath| max_dev(&s0, &sample(p, 128));

    let mut naive = before.clone();
    naive.verts.remove(1);
    let mut cur = before.clone();
    ph2d_vec_scene::dissolve_vertex(&mut cur.verts, 1, false);

    let fit_with = |f: ([f64; 2], [f64; 2])| {
        let mut p = before.clone();
        p.verts[0].out_handle = f.0;
        p.verts[2].in_handle = f.1;
        p.verts.remove(1);
        p
    };
    let sch = fit_with(schneider_fit(
        &before.verts[0],
        &before.verts[1],
        &before.verts[2],
        8,
    ));
    let free_f = free_fit(&before.verts[0], &before.verts[1], &before.verts[2], 8);
    let free_p = fit_with(free_f);
    let ang = |a: [f64; 2], b: [f64; 2]| {
        let c = (a[0] * b[0] + a[1] * b[1]) / (a[0].hypot(a[1]) * b[0].hypot(b[1]));
        c.clamp(-1.0, 1.0).acos().to_degrees()
    };
    let t_old = [
        before.verts[0].out_handle[0] - before.verts[0].anchor[0],
        before.verts[0].out_handle[1] - before.verts[0].anchor[1],
    ];
    let t_new = [
        free_f.0[0] - before.verts[0].anchor[0],
        free_f.0[1] - before.verts[0].anchor[1],
    ];

    println!("\n  arco de 2,0 de largura por 0,75 de altura, no' do meio apagado");
    println!(
        "  remocao CRUA (o de antes)                      desvio {:.4}",
        dev(&naive)
    );
    println!(
        "  ATUAL  passa pela ancora, tangentes fixas      desvio {:.4}",
        dev(&cur)
    );
    println!(
        "  SCHNEIDER  LSQ + reparametrizacao, 8 iters     desvio {:.4}",
        dev(&sch)
    );
    println!(
        "  PISO   tangentes LIVRES                        desvio {:.4}   (a tangente da ponta gira {:.1} graus)",
        dev(&free_p),
        ang(t_old, t_new)
    );
    println!();
}
