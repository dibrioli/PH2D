//! **O RELÓGIO DO ESFREGÃO** — o report do dono (*«o efeito parece OK, mas meio
//! travado, até na hora de rotacionar o canvas dá uma travadinha»*, 2026-09-14).
//!
//! ⚠️ **A pista é a ROTAÇÃO:** rodar o canvas não corre a lei do pincel. ⇒ o
//! custo que ele sente ao rodar **não pode** ser do alvo por-vértice, e a
//! primeira coisa a medir é o TAMANHO da peça que o roteiro manda construir.

use std::time::Instant;

fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// **A ESCADA DA PEÇA** — quantos vértices o roteiro de uma cena de escultura
/// cria a cada `K`, e o que cada passo custa.
#[test]
#[ignore = "medição: o relógio do esfregão e da peça que o roteiro constrói"]
fn mede_a_escada_da_peca_e_a_referencia() {
    let mut m = ph2d_mesh::shapes::sculpt_sphere(1.0);
    println!("K=0  V = {:>9}", m.vert_count());
    for k in 1..=2 {
        let t = Instant::now();
        m = ph2d_mesh::subdivide(&m);
        println!(
            "K={k}  V = {:>9}   subdivide {:>8.1} ms",
            m.vert_count(),
            t.elapsed().as_secs_f64() * 1e3
        );
    }
}

/// ⭐⭐ **O CUSTO DO PEN-DOWN** — a superfície de referência é fotografada a cada
/// toque, e ela é um `subdivide` inteiro mais um ponto-limite por vértice.
#[test]
#[ignore = "medição: o pen-down dos dois pincéis de multirresolução"]
fn mede_a_superficie_de_referencia() {
    for k in 0..=2 {
        let mut baixo = ph2d_mesh::shapes::sculpt_sphere(1.0);
        for _ in 0..k {
            baixo = ph2d_mesh::subdivide(&baixo);
        }
        let amostras: Vec<f64> = (0..3)
            .map(|_| {
                let t = Instant::now();
                let previsto = ph2d_mesh::subdivide(&baixo);
                let r: Vec<[f32; 3]> = (0..previsto.vert_count())
                    .map(|v| match ph2d_mesh::limit_point(&previsto, v) {
                        ph2d_mesh::LimitPoint::At(q) => q,
                        ph2d_mesh::LimitPoint::None => previsto.positions()[v],
                    })
                    .collect();
                let ms = t.elapsed().as_secs_f64() * 1e3;
                std::hint::black_box(r);
                ms
            })
            .collect();
        println!(
            "nivel de baixo V = {:>9}  ->  topo V = {:>9}   referencia {:>8.1} ms",
            baixo.vert_count(),
            baixo.vert_count() * 4 - 6,
            mediana(amostras)
        );
    }
}

/// ⭐ **O CUSTO DE UM DAB** — o esfregão contra o `Draw`, na MESMA peça e com a
/// mesma pegada.
#[test]
#[ignore = "medição: um dab do esfregão contra um do Draw"]
fn mede_um_dab_do_esfregao() {
    for niveis in [0usize, 1, 2] {
        let mut base = ph2d_mesh::shapes::sculpt_sphere(1.0);
        for _ in 0..niveis {
            base = ph2d_mesh::subdivide(&base);
        }
        let referencia: Vec<[f32; 3]> = base.positions().to_vec();
        for verbo in [
            ph2d_sculpt3d::Verb::Draw,
            ph2d_sculpt3d::Verb::Smooth,
            ph2d_sculpt3d::Verb::SmearMultires,
        ] {
            let b = ph2d_sculpt3d::Brush {
                verb: verbo,
                radius: 0.35,
                strength: 1.0,
                ..ph2d_sculpt3d::Brush::default()
            };
            let amostras: Vec<f64> = (0..5)
                .map(|_| {
                    let mut mesh = base.clone();
                    let mut s = ph2d_sculpt3d::SculptStroke::default();
                    s.begin(&mesh);
                    s.reference = referencia.clone();
                    // O 1.º dab arma o caminho; o 2.º é o que se cronometra.
                    let c0 = [0.0, 1.0, 0.0];
                    s.dab(
                        &mut mesh,
                        &b,
                        &ph2d_sculpt3d::Dab::at(c0, b.radius, [0.0, -1.0, 0.0]),
                        ph2d_sculpt3d::Symmetry::default(),
                    );
                    let t = Instant::now();
                    s.dab(
                        &mut mesh,
                        &b,
                        &ph2d_sculpt3d::Dab::at([0.05, 1.0, 0.0], b.radius, [0.0, -1.0, 0.0]),
                        ph2d_sculpt3d::Symmetry::default(),
                    );
                    t.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            println!(
                "V = {:>9}  {:>16}  dab {:>7.3} ms",
                base.vert_count(),
                verbo.label(),
                mediana(amostras)
            );
        }
    }
}

/// ⭐⭐⭐ **QUANTO O RELEVO ANDA POR DAB, e contra QUE ele se compara** — o 2.º
/// report do dono (*«a intensidade parece baixa mesmo no máximo»*, 2026-09-14).
///
/// ⚠️ **A hipótese a medir:** a lei da espec §5.2 é uma média sobre o **ANEL**,
/// logo o que ela transporta por dab é da ordem de UMA ARESTA. Numa peça densa
/// uma aresta é uma fracção minúscula do raio do pincel ⇒ o relevo anda, e anda
/// pouco. *Se for isso, a alavanca não é a força — é a DENSIDADE.*
#[test]
#[ignore = "medição: quanto o relevo anda por dab, contra a aresta e contra o raio"]
fn mede_o_transporte_por_dab() {
    for niveis in [0usize, 1] {
        let mut base = ph2d_mesh::shapes::sculpt_sphere(1.0);
        for _ in 0..niveis {
            base = ph2d_mesh::subdivide(&base);
        }
        let referencia: Vec<[f32; 3]> = base.positions().to_vec();

        // Uma bossa gaussiana no pólo — o relevo que o esfregão transporta.
        let mut mesh = {
            let mut pos = referencia.clone();
            for p in &mut pos {
                let d2 = p[0] * p[0] + (p[1] - 1.0) * (p[1] - 1.0) + p[2] * p[2];
                let h = 0.20 * (-d2 / 0.10).exp();
                let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
                for c in p.iter_mut() {
                    *c += *c / n * h;
                }
            }
            ph2d_mesh::Mesh::from_parts(pos, base.faces().to_vec()).expect("a bossa")
        };

        // A aresta média da peça — a unidade em que a lei do anel trabalha.
        let aresta = {
            let mut soma = 0.0f64;
            let mut n = 0u64;
            for f in mesh.faces() {
                let v = f.verts();
                for i in 0..v.len() {
                    let a = mesh.positions()[v[i] as usize];
                    let b = mesh.positions()[v[(i + 1) % v.len()] as usize];
                    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    soma += f64::from((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
                    n += 1;
                }
            }
            soma / n as f64
        };

        let b = ph2d_sculpt3d::Brush {
            verb: ph2d_sculpt3d::Verb::SmearMultires,
            radius: 0.35,
            strength: 1.0,
            ..ph2d_sculpt3d::Brush::default()
        };
        let mut s = ph2d_sculpt3d::SculptStroke::default();
        s.begin(&mesh);
        s.reference = referencia.clone();

        let antes = mesh.positions().to_vec();
        let passo = b.radius * 0.1;
        let mut por_dab = Vec::new();
        for k in 0..12 {
            let anterior = mesh.positions().to_vec();
            let c = [passo * k as f32, 1.0, 0.0];
            s.dab(
                &mut mesh,
                &b,
                &ph2d_sculpt3d::Dab::at(c, b.radius, [0.0, -1.0, 0.0]),
                ph2d_sculpt3d::Symmetry::default(),
            );
            let d = anterior
                .iter()
                .zip(mesh.positions())
                .map(|(p, q)| {
                    let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
                })
                .fold(0.0f32, f32::max);
            por_dab.push(d);
        }
        let total = antes
            .iter()
            .zip(mesh.positions())
            .map(|(p, q)| {
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            })
            .fold(0.0f32, f32::max);
        println!(
            "V = {:>9}  aresta {:.5}  raio 0,35 = {:.1} arestas  |  1.o dab {:.5}  \
             pior dab {:.5}  TOTAL em 12 dabs {:.5}  (bossa 0,200)",
            base.vert_count(),
            aresta,
            0.35 / aresta,
            por_dab[1],
            por_dab.iter().copied().fold(0.0f32, f32::max),
            total
        );
    }
}
