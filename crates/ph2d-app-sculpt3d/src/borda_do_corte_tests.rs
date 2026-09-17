//! SONDA — **a topologia da BORDA do corte, e o que o `Smooth` lhe faz.**
//!
//! Report do dono (2026-09-17): *«o que não fica legal é a topologia das bordas
//! do corte, pois com smooth não se consegue alisar»*.
//!
//! # ⭐⭐⭐ A lei, medida
//!
//! A borda de um corte é feita de **duas** espécies de ponto: os que o motor
//! põe **na curva desenhada** e os **vértices da própria peça** que ele aproveita
//! quando estão perto. Os segundos não estão na curva ⇒ a borda **serrilha**, e
//! a amplitude é a da malha:
//!
//! | peça | aresta | desvio da curva (mundo) p50 · p90 · MAX |
//! |---|---|---|
//! | `12 k` T | `0,0498` | `0,0277` · `0,0557` · `0,0665` |
//! | `50 k` T (a `=46` até 17/09) | `0,0242` | `0,0103` · `0,0201` · `0,0298` |
//! | `80 k` T | `0,0191` | `0,00007` · `0,0137` · `0,0209` |
//! | **`196 k` T (a peça do MÓDULO)** | `0,0124` | **`0,00000` · `0,00007` · `0,00007`** |
//!
//! ⇒ **na peça com que o artista trabalha a borda cai na curva**; a cena `=46`
//! é que abria com uma bola `4×` mais grossa, escolhida por um cabeçalho que
//! nomeava o **relógio** do corte (`58 ms` contra `375`) e era **cego à borda**.
//!
//! # ⛔⛔ E nenhum pincel a pode curar, porque o que falta são CÉLULAS
//!
//! * o `Smooth` **funciona** — ele amacia a quina (p90 `125,9° → 41,1°` numa
//!   passagem) e **não encrespa** a casca à volta (`p50 1,14°` antes e depois);
//! * mas ele não põe a borda na curva, porque *mover vértices não muda de que
//!   vértices o anel é feito*;
//! * e nem o movimento **IDEAL** o faz: pondo cada ponto da borda exactamente na
//!   curva, o desvio vai a `0` e o pior triângulo da casca salta de `25,7` para
//!   **`1 166`**.
//!
//! *É a mesma frase que a linha do quad remesh pagou duas vezes: **o que falta
//! não são POSIÇÕES, são CÉLULAS**.*

use ph2d_mesh::{Mesh, Ray, shapes};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// A esfera do `=46` cortada por um círculo — o caminho da `ph2d-trim` mais o
/// motor, que é o que o botão corre.
fn corta(centro_x: f32) -> (Mesh, Mesh, f32) {
    corta_com(centro_x, 50_000)
}

fn corta_com(centro_x: f32, alvo_tris: usize) -> (Mesh, Mesh, f32) {
    let bola = shapes::sphere_with_triangles(alvo_tris, 1.0);
    let tris: usize = bola
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris as f32);
    let n = 200usize;
    let anel: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            [centro_x + 0.6 * t.cos(), 0.6 * t.sin()]
        })
        .collect();
    let raios: Vec<Ray> = anel
        .iter()
        .map(|p| Ray::new([p[0], p[1], 10.0], [0.0, 0.0, -1.0]))
        .collect();
    let lamina = ph2d_trim::prisma(
        &anel,
        &raios,
        &ph2d_trim::Plano {
            origem: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        },
        &bola,
        ph2d_trim::Profundidade::DaPeca,
        ph2d_trim::Paredes::Fixas,
        ph2d_trim::Resolucao::Ate(alvo),
    )
    .expect("o prisma");
    let out = ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("o corte");
    (bola, out, alvo)
}

fn triangulos(m: &Mesh) -> Vec<[u32; 3]> {
    let mut t = Vec::new();
    for f in m.faces() {
        f.triangles(&mut t);
    }
    t
}

fn d(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// A BORDA: as arestas onde uma face da casca encontra uma que não é da casca.
/// Devolve `(arestas_da_borda, vértices_da_borda)`.
fn borda(m: &Mesh) -> (Vec<(u32, u32)>, std::collections::BTreeSet<u32>) {
    let p = m.positions();
    let t = triangulos(m);
    let na_casca = |i: u32| {
        let q = p[i as usize];
        ((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - 1.0).abs() < 1e-4
    };
    let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<bool>> =
        std::collections::BTreeMap::new();
    for x in &t {
        let casca = x.iter().all(|&i| na_casca(i));
        for (a, b) in [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])] {
            por_aresta
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(casca);
        }
    }
    let mut arestas = Vec::new();
    let mut verts = std::collections::BTreeSet::new();
    for (k, v) in &por_aresta {
        if v.len() == 2 && v[0] != v[1] {
            arestas.push(*k);
            verts.insert(k.0);
            verts.insert(k.1);
        }
    }
    (arestas, verts)
}

fn pc(v: &[f32], q: usize) -> f32 {
    if v.is_empty() {
        return f32::NAN;
    }
    v[(v.len() - 1) * q / 100]
}

fn aspecto(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let (l0, l1, l2) = (d(a, b), d(b, c), d(c, a));
    let s = (l0 + l1 + l2) * 0.5;
    let area = (s * (s - l0) * (s - l1) * (s - l2)).max(0.0).sqrt();
    if area > 1e-14 {
        l0.max(l1).max(l2) * s / (2.0 * area)
    } else {
        f32::INFINITY
    }
}

/// O ângulo entre as duas faces de cada aresta da BORDA, em graus — a «quina».
fn quina(m: &Mesh, arestas: &[(u32, u32)]) -> Vec<f32> {
    let p = m.positions();
    let t = triangulos(m);
    let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, x) in t.iter().enumerate() {
        for (a, b) in [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])] {
            por_aresta.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let nrm = |x: &[u32; 3]| {
        let (a, b, c) = (p[x[0] as usize], p[x[1] as usize], p[x[2] as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ]
    };
    let mut ang = Vec::new();
    for k in arestas {
        let Some(l) = por_aresta.get(k) else { continue };
        if l.len() != 2 {
            continue;
        }
        let (n0, n1) = (nrm(&t[l[0]]), nrm(&t[l[1]]));
        let (a0, a1) = (
            (n0[0] * n0[0] + n0[1] * n0[1] + n0[2] * n0[2]).sqrt(),
            (n1[0] * n1[0] + n1[1] * n1[1] + n1[2] * n1[2]).sqrt(),
        );
        if a0 <= 0.0 || a1 <= 0.0 {
            continue;
        }
        let c = ((n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]) / (a0 * a1)).clamp(-1.0, 1.0);
        ang.push(c.acos().to_degrees());
    }
    ang.sort_by(f32::total_cmp);
    ang
}

/// **A QUINA do corte aguenta o `Smooth`?** — o que o dono descreve.
#[test]
#[ignore = "sonda de diagnóstico: corre à mão"]
fn diag_a_quina_contra_o_smooth() {
    let (_bola, out, alvo) = corta(0.0);
    let (arestas, verts) = borda(&out);
    let q0 = quina(&out, &arestas);
    println!(
        "ANTES : quina p10={:>6.1}° p50={:>6.1}° p90={:>6.1}°  (n={})",
        pc(&q0, 10),
        pc(&q0, 50),
        pc(&q0, 90),
        q0.len()
    );
    let anel: Vec<[f32; 3]> = verts.iter().map(|&i| out.positions()[i as usize]).collect();
    for (raio_em_arestas, passagens) in [(3.0f32, 1usize), (3.0, 5), (8.0, 5)] {
        let mut mesh = out.clone();
        let raio = raio_em_arestas * alvo;
        let b = Brush {
            verb: Verb::Smooth,
            radius: raio,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        // Um traço que percorre o anel INTEIRO, `passagens` vezes.
        for _ in 0..passagens {
            for c in &anel {
                let dab = Dab::at(*c, raio, [c[0], c[1], c[2] + 10.0]);
                s.dab(&mut mesh, &b, &dab, Symmetry::default());
            }
        }
        let q = quina(&mesh, &arestas);
        println!(
            "DEPOIS (raio {raio_em_arestas:>3} arestas, {passagens} passagem(ns)): \
             quina p10={:>6.1}° p50={:>6.1}° p90={:>6.1}°  (n={})",
            pc(&q, 10),
            pc(&q, 50),
            pc(&q, 90),
            q.len()
        );
    }
}

/// **O PRÉMIO, antes de prometer a cura:** e se a borda fosse posta EXACTAMENTE
/// na curva que o gesto desenhou?
///
/// A curva é `esfera ∩ cilindro`: `hypot(x, y) = 0,6` sobre `r = 1`, ou seja o
/// círculo em `|z| = 0,8`. Aqui isso é feito **na sonda** (nada de produto) só
/// para medir o que se ganharia — e o que se pagaria.
#[test]
#[ignore = "sonda de diagnóstico: corre à mão"]
fn diag_o_premio_de_por_a_borda_na_curva() {
    let (_bola, out, alvo) = corta(0.0);
    let (arestas, verts) = borda(&out);
    let na_casca = |p: &[[f32; 3]], i: u32| {
        let q = p[i as usize];
        ((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - 1.0).abs() < 1e-4
    };
    let retrato = |m: &Mesh, rot: &str| {
        let p = m.positions();
        let mut dv: Vec<f32> = verts
            .iter()
            .map(|&i| (p[i as usize][0].hypot(p[i as usize][1]) - 0.6).abs() / alvo)
            .collect();
        dv.sort_by(f32::total_cmp);
        let mut asp: Vec<f32> = triangulos(m)
            .iter()
            .filter(|x| x.iter().all(|&i| na_casca(p, i)))
            .map(|x| aspecto(p[x[0] as usize], p[x[1] as usize], p[x[2] as usize]))
            .collect();
        asp.sort_by(f32::total_cmp);
        let q = quina(m, &arestas);
        println!(
            "{rot:>10}: borda p50={:.4} p90={:.4} MAX={:.4} | aspecto da casca p99={:>6.2} MAX={:>8.2} | quina p90={:>6.1}°",
            pc(&dv, 50),
            pc(&dv, 90),
            dv.last().copied().unwrap_or(f32::NAN),
            pc(&asp, 99),
            asp.last().copied().unwrap_or(f32::NAN),
            pc(&q, 90)
        );
    };
    retrato(&out, "HOJE");

    let mut curada = out.clone();
    {
        let p = curada.positions_mut();
        for &i in &verts {
            let q = p[i as usize];
            let h = q[0].hypot(q[1]);
            if h <= 0.0 {
                continue;
            }
            let (k, z) = (0.6 / h, if q[2] >= 0.0 { 0.8 } else { -0.8 });
            p[i as usize] = [q[0] * k, q[1] * k, z];
        }
    }
    retrato(&curada, "NA CURVA");
}

/// **O encrespar, com a régua limpa:** o ângulo entre faces vizinhas de TODA a
/// casca dentro da faixa do pincel, antes e depois do `Smooth`.
#[test]
#[ignore = "sonda de diagnóstico: corre à mão"]
fn diag_o_smooth_encrespa_a_casca() {
    let (_bola, out, alvo) = corta(0.0);
    let (_arestas, verts) = borda(&out);
    let p0 = out.positions().to_vec();
    let na_casca = |p: &[[f32; 3]], i: u32| {
        let q = p[i as usize];
        ((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - 1.0).abs() < 1e-3
    };
    // A faixa: arestas cujas DUAS faces são da casca e que ficam a menos de 6
    // arestas da borda — a quina do corte fica de fora por construção.
    let perto: std::collections::BTreeSet<u32> = (0..p0.len() as u32)
        .filter(|&i| {
            verts
                .iter()
                .any(|&j| d(p0[i as usize], p0[j as usize]) < 6.0 * alvo)
        })
        .collect();
    let faixa = |m: &Mesh| -> Vec<f32> {
        let p = m.positions();
        let t = triangulos(m);
        let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
            std::collections::BTreeMap::new();
        for (i, x) in t.iter().enumerate() {
            if !x.iter().all(|&v| na_casca(p, v)) {
                continue;
            }
            for (a, b) in [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])] {
                por_aresta.entry((a.min(b), a.max(b))).or_default().push(i);
            }
        }
        let pares: Vec<(u32, u32)> = por_aresta
            .iter()
            .filter(|(k, v)| v.len() == 2 && perto.contains(&k.0) && perto.contains(&k.1))
            .map(|(k, _)| *k)
            .collect();
        quina(m, &pares)
    };
    let a = faixa(&out);
    println!(
        "ANTES : faixa da casca junto da borda — p50={:>5.2}° p90={:>5.2}° MAX={:>6.2}°  (n={})",
        pc(&a, 50),
        pc(&a, 90),
        a.last().copied().unwrap_or(f32::NAN),
        a.len()
    );
    let anel: Vec<[f32; 3]> = verts.iter().map(|&i| p0[i as usize]).collect();
    for passagens in [1usize, 5] {
        let mut mesh = out.clone();
        let raio = 3.0 * alvo;
        let b = Brush {
            verb: Verb::Smooth,
            radius: raio,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for _ in 0..passagens {
            for c in &anel {
                let dab = Dab::at(*c, raio, [c[0], c[1], c[2] + 10.0]);
                s.dab(&mut mesh, &b, &dab, Symmetry::default());
            }
        }
        let v = faixa(&mesh);
        println!(
            "DEPOIS ({passagens} passagem(ns)): p50={:>5.2}° p90={:>5.2}° MAX={:>6.2}°",
            pc(&v, 50),
            pc(&v, 90),
            v.last().copied().unwrap_or(f32::NAN)
        );
    }
}

/// **O zigue-zague em unidades de MUNDO contra a densidade da peça** — é o que
/// decide se a cura é adensar a peça no sítio do corte.
#[test]
#[ignore = "sonda de diagnóstico: corre à mão"]
fn diag_o_zigue_zague_contra_a_densidade() {
    for tris in [12_000usize, 30_000, 50_000, 80_000, 120_000, 200_000] {
        let (_bola, out, alvo) = corta_com(0.0, tris);
        let (_a, verts) = borda(&out);
        let p = out.positions();
        let mut v: Vec<f32> = verts
            .iter()
            .map(|&i| (p[i as usize][0].hypot(p[i as usize][1]) - 0.6).abs())
            .collect();
        v.sort_by(f32::total_cmp);
        println!(
            "peça ~{tris:>7} T (aresta {alvo:.5}): borda n={:>4}  desvio em MUNDO \
             p50={:.5} p90={:.5} MAX={:.5}   (= {:.3} · {:.3} · {:.3} arestas)",
            v.len(),
            pc(&v, 50),
            pc(&v, 90),
            v.last().copied().unwrap_or(f32::NAN),
            pc(&v, 50) / alvo,
            pc(&v, 90) / alvo,
            v.last().copied().unwrap_or(f32::NAN) / alvo
        );
    }
}

/// **E na peça com que o módulo ABRE?** — a `sculpt_sphere`, que é o que o
/// artista tem na mão fora da cena `=46`.
#[test]
#[ignore = "sonda de diagnóstico: corre à mão"]
fn diag_a_borda_na_peca_de_fabrica() {
    let bola = shapes::sculpt_sphere(1.0);
    let mut t = Vec::new();
    for f in bola.faces() {
        f.triangles(&mut t);
    }
    let b = bola.bounds();
    let raio_da_peca = (b.max[0] - b.min[0]) * 0.5;
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), t.len() as f32);
    // O anel do gesto: 60 % do raio DA PEÇA, como na cena.
    let rc = 0.6 * raio_da_peca;
    let n = 200usize;
    let anel: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let th = i as f32 / n as f32 * std::f32::consts::TAU;
            [rc * th.cos(), rc * th.sin()]
        })
        .collect();
    let raios: Vec<Ray> = anel
        .iter()
        .map(|p| Ray::new([p[0], p[1], 10.0], [0.0, 0.0, -1.0]))
        .collect();
    let lamina = ph2d_trim::prisma(
        &anel,
        &raios,
        &ph2d_trim::Plano {
            origem: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        },
        &bola,
        ph2d_trim::Profundidade::DaPeca,
        ph2d_trim::Paredes::Fixas,
        ph2d_trim::Resolucao::Ate(alvo),
    )
    .expect("o prisma");
    let out = ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("o corte");
    let (_a, verts) = borda_com(&out, raio_da_peca);
    let p = out.positions();
    let mut v: Vec<f32> = verts
        .iter()
        .map(|&i| (p[i as usize][0].hypot(p[i as usize][1]) - rc).abs())
        .collect();
    v.sort_by(f32::total_cmp);
    println!(
        "PEÇA DE FÁBRICA: {} T, aresta {alvo:.5}, raio {raio_da_peca:.4}\n  \
         borda n={:>4}  desvio em MUNDO p50={:.5} p90={:.5} MAX={:.5}  \
         (= {:.3} · {:.3} · {:.3} arestas)",
        t.len(),
        v.len(),
        pc(&v, 50),
        pc(&v, 90),
        v.last().copied().unwrap_or(f32::NAN),
        pc(&v, 50) / alvo,
        pc(&v, 90) / alvo,
        v.last().copied().unwrap_or(f32::NAN) / alvo
    );
}

/// A borda, para uma peça cujo «na casca» é um raio qualquer.
fn borda_com(m: &Mesh, raio: f32) -> (Vec<(u32, u32)>, std::collections::BTreeSet<u32>) {
    let p = m.positions();
    let t = triangulos(m);
    let na_casca = |i: u32| {
        let q = p[i as usize];
        ((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - raio).abs() < 0.04 * raio
    };
    let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<bool>> =
        std::collections::BTreeMap::new();
    for x in &t {
        let casca = x.iter().all(|&i| na_casca(i));
        for (a, b) in [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])] {
            por_aresta
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(casca);
        }
    }
    let (mut arestas, mut verts) = (Vec::new(), std::collections::BTreeSet::new());
    for (k, v) in &por_aresta {
        if v.len() == 2 && v[0] != v[1] {
            arestas.push(*k);
            verts.insert(k.0);
            verts.insert(k.1);
        }
    }
    (arestas, verts)
}

/// **O RELÓGIO do corte contra a densidade da peça** — o número que a cena `=46`
/// trocou pela qualidade da borda. ⚠️ Corra em `--release`.
#[test]
#[ignore = "sonda de diagnóstico: corre à mão (--release)"]
fn diag_o_relogio_do_corte() {
    for (nome, bola) in [
        ("=46 (50 k T)", shapes::sphere_with_triangles(50_000, 1.0)),
        ("fábrica (196 k T)", shapes::sculpt_sphere(1.0)),
    ] {
        let mut t = Vec::new();
        for f in bola.faces() {
            f.triangles(&mut t);
        }
        let b = bola.bounds();
        let raio_da_peca = (b.max[0] - b.min[0]) * 0.5;
        let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), t.len() as f32);
        let rc = 0.6 * raio_da_peca;
        let n = 200usize;
        let anel: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let th = i as f32 / n as f32 * std::f32::consts::TAU;
                [rc * th.cos(), rc * th.sin()]
            })
            .collect();
        let raios: Vec<Ray> = anel
            .iter()
            .map(|p| Ray::new([p[0], p[1], 10.0], [0.0, 0.0, -1.0]))
            .collect();
        let lamina = ph2d_trim::prisma(
            &anel,
            &raios,
            &ph2d_trim::Plano {
                origem: [0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            &bola,
            ph2d_trim::Profundidade::DaPeca,
            ph2d_trim::Paredes::Fixas,
            ph2d_trim::Resolucao::Ate(alvo),
        )
        .expect("o prisma");
        let mut melhor = f64::INFINITY;
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            let _ = ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair)
                .expect("o corte");
            melhor = melhor.min(t0.elapsed().as_secs_f64() * 1000.0);
        }
        println!("{nome:>20}: {} T, corte = {melhor:>8.1} ms", t.len());
    }
}

/// ⭐⭐⭐ **A BORDA DO CORTE CAI NA CURVA DESENHADA — à densidade do MÓDULO.**
///
/// ⚠️⚠️ **O CONTROLO é metade do gate, e é a metade que explica o report:** a um
/// QUARTO da densidade a mesma cadeia serrilha até `1,2` arestas da malha, e era
/// essa a peça com que a cena `=46` abria até 17/09.
///
/// ⚠️ **As duas metades correm sobre a MESMA primitiva, e isso é a lei do
/// método:** a 1.ª redacção comparava a `sculpt_sphere` com uma esfera UV e
/// precisou de **duas** tolerâncias de «isto está na casca» — o raio da primeira
/// varia `3,09 %` por construção. *Com duas réguas, o controlo lia `0,0000` e
/// acusava a fixtura de não conter o fenómeno que ela continha.* ⇒ a variável
/// isolada é a **DENSIDADE**, e a densidade da metade fina é **lida do produto**.
#[test]
fn a_borda_do_corte_cai_na_curva_desenhada() {
    let tris_da_fabrica: usize = crate::scenes::mesh::peca_de_fabrica()
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let desvio = |alvo_tris: usize| -> Vec<f32> {
        let (_bola, out, alvo) = corta_com(0.0, alvo_tris);
        let (_a, verts) = borda(&out);
        let p = out.positions();
        let mut v: Vec<f32> = verts
            .iter()
            .map(|&i| (p[i as usize][0].hypot(p[i as usize][1]) - 0.6).abs() / alvo)
            .collect();
        v.sort_by(f32::total_cmp);
        v
    };

    let fina = desvio(tris_da_fabrica);
    assert!(
        fina.len() > 400,
        "PISO DE POPULAÇÃO: só {} vértices de borda — a régua deixou de a achar",
        fina.len()
    );
    assert!(
        pc(&fina, 90) < 0.05,
        "à densidade do módulo ({tris_da_fabrica} T) a borda saiu da curva \
         desenhada: p90 = {:.4} arestas da malha (medido 0,006) — é o serrilhado \
         que o dono reportou em 17/09",
        pc(&fina, 90)
    );

    // ⭐ O CONTROLO: a um quarto da densidade a MESMA cadeia serrilha.
    let grossa = desvio(tris_da_fabrica / 4);
    assert!(
        pc(&grossa, 90) > 0.3,
        "CONTROLO: a peça grossa deixou de serrilhar (p90 = {:.4}) — a fixtura \
         já não contém o fenómeno e a metade de cima não afirma nada",
        pc(&grossa, 90)
    );
}
