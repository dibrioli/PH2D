//! Testes do `neighbors.rs` — módulo FILHO por `#[path]` (o pai cruzou o teto de 700 LOC
//! com a partição por passagem). `use super::*` alcança os privados como sempre.

use super::*;

fn seg(a: u32, b: u32, pa: (f32, f32), pb: (f32, f32), r: f32) -> Seg {
    Seg {
        a,
        b,
        pa: Vec2::new(pa.0, pa.1),
        pb: Vec2::new(pb.0, pb.1),
        radius: r,
    }
}

#[test]
fn a_straight_stroke_has_no_geometric_neighbors() {
    // Segmentos consecutivos SÃO adjacentes (a janela de sequência os cobre) e
    // os distantes não se aproximam: lista vazia = custo zero no fragment.
    let segs = [
        seg(0, 1, (0.0, 0.0), (20.0, 0.0), 2.0),
        seg(1, 2, (20.0, 0.0), (40.0, 0.0), 2.0),
        seg(2, 3, (40.0, 0.0), (60.0, 0.0), 2.0),
        seg(3, 4, (60.0, 0.0), (80.0, 0.0), 2.0),
    ];
    let extras = extras_for_stroke(&segs);
    assert!(
        extras.iter().all(|e| e.list.is_empty()),
        "reta não tem vizinho geométrico: {extras:?}"
    );
}

#[test]
fn a_zigzag_that_folds_back_sees_the_far_segment() {
    // O 3º segmento passa RENTE ao 1º (2 px), com raio 5 — é a "mordida" de
    // longo alcance: os quads se sobrepõem mas os índices não são adjacentes.
    let segs = [
        seg(0, 1, (0.0, 0.0), (40.0, 0.0), 5.0),
        seg(1, 2, (40.0, 0.0), (10.0, 12.0), 5.0),
        seg(2, 3, (10.0, 12.0), (50.0, 3.0), 5.0),
    ];
    let extras = extras_for_stroke(&segs);
    assert_eq!(
        extras[0].list,
        vec![2],
        "o segmento 0 tem de VER o segmento 2"
    );
    assert_eq!(
        extras[2].list,
        vec![0],
        "e vice-versa (o teste é simétrico aqui)"
    );
    assert!(extras[1].list.is_empty(), "o do meio é adjacente aos dois");
}

#[test]
fn a_far_parallel_segment_is_not_a_neighbor() {
    // Mesmo traço voltando, mas LONGE (30 px com raio 2): fora do alcance de
    // qualquer pixel do quad → não entra na lista (o critério é conservador,
    // não paranoico).
    let segs = [
        seg(0, 1, (0.0, 0.0), (40.0, 0.0), 2.0),
        seg(1, 2, (40.0, 0.0), (40.0, 30.0), 2.0),
        seg(2, 3, (40.0, 30.0), (0.0, 30.0), 2.0),
    ];
    let extras = extras_for_stroke(&segs);
    assert!(
        extras[0].list.is_empty(),
        "30 px é longe demais: {extras:?}"
    );
    assert!(extras[2].list.is_empty());
}

#[test]
fn crossing_segments_are_neighbors() {
    // Um X: os segmentos 0 e 2 se CRUZAM (distância 0).
    let segs = [
        seg(0, 1, (0.0, 0.0), (40.0, 40.0), 4.0),
        seg(1, 2, (40.0, 40.0), (40.0, 0.0), 4.0),
        seg(2, 3, (40.0, 0.0), (0.0, 40.0), 4.0),
    ];
    let extras = extras_for_stroke(&segs);
    assert_eq!(extras[0].list, vec![2]);
    assert_eq!(extras[2].list, vec![0]);
}

/// A referência ingênua `O(n²)` — o oráculo do grid.
/// O oráculo `O(n²)` do broadphase. ⚠️ **A fita local vem pela MESMA porta do
/// produto** (`push_ribbon_local`) em vez de ser re-derivada aqui: o que este teste
/// existe para provar é que o GRID não perde um CRUZAMENTO, e uma 2ª cópia da regra
/// de fita transformaria uma divergência de partição num falso positivo (foi o que
/// aconteceu quando a partição nasceu: o grid achava o vizinho `i±2` por arco e o
/// oráculo o descartava por ranking de distância — o oráculo é que estava velho).
fn brute_force(segs: &[Seg]) -> Vec<Vec<u32>> {
    let n = segs.len();
    let mut out = vec![Vec::new(); n];
    let closed = segs[n - 1].b == segs[0].a;
    let max_radius = segs.iter().fold(0.0f32, |m, s| m.max(s.radius));
    let mut stamp = vec![0u32; n];
    let mut walked = vec![0u32; n];
    for i in 0..n {
        let visit = i as u32 + 1;
        push_ribbon_local(
            segs,
            i,
            closed,
            max_radius,
            &mut stamp,
            &mut walked,
            visit,
            &mut out[i],
        );
        let mut cand: Vec<(f32, u32)> = Vec::new();
        for j in 0..n {
            let (si, sj) = (segs[i], segs[j]);
            if i == j || is_adjacent(&si, &sj) || stamp[j] == visit {
                continue;
            }
            let d2 = seg_seg_distance_sq(si.pa, si.pb, sj.pa, sj.pb);
            let reach = 2.0 * si.radius + sj.radius;
            if d2 < reach * reach {
                cand.push((d2, j as u32));
            }
        }
        cand.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        cand.truncate(MAX_EXTRAS_PER_SEGMENT);
        out[i].extend(cand.iter().map(|&(_, j)| j));
    }
    out
}

#[test]
fn the_grid_finds_exactly_what_the_pairwise_scan_finds() {
    // Um rabisco denso que se auto-cruza muito (LCG determinístico — HR-5: sem
    // transcendental, sem RNG do sistema). Se o grid perder um vizinho, um pixel
    // do traço volta a ter a mordida — e este teste é a única barreira.
    let mut state: u32 = 0x1234_5678;
    let mut rnd = || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (state >> 8) as f32 / f32::from(u16::MAX) % 1.0
    };
    let mut segs = Vec::new();
    let mut prev = Vec2::new(32.0, 32.0);
    for k in 0..180u32 {
        let next = Vec2::new(rnd() * 60.0 + 2.0, rnd() * 60.0 + 2.0);
        let r = 2.0 + rnd() * 6.0;
        segs.push(seg(k, k + 1, (prev.x, prev.y), (next.x, next.y), r));
        prev = next;
    }
    let by_grid = extras_for_stroke(&segs);
    let by_pairs = brute_force(&segs);
    assert_eq!(
        by_grid.len(),
        by_pairs.len(),
        "mesma quantidade de segmentos"
    );
    for (i, (g, p)) in by_grid.iter().zip(&by_pairs).enumerate() {
        let (mut g, mut p) = (g.list.clone(), p.clone());
        g.sort_unstable();
        p.sort_unstable();
        assert_eq!(g, p, "segmento {i}: o grid divergiu do par-a-par");
    }
    assert!(
        by_pairs.iter().any(|v: &Vec<u32>| !v.is_empty()),
        "o rabisco tem de produzir vizinhos (senão o teste não prova nada)"
    );
}

#[test]
fn the_extras_list_is_capped_and_sorted_by_proximity() {
    // Um "pente" onde o segmento 0 é rente a MUITOS outros: o corte mantém os
    // mais próximos (o fragment nunca itera sem teto).
    let mut segs = vec![seg(0, 1, (0.0, 0.0), (100.0, 0.0), 6.0)];
    for k in 0..40u32 {
        let y = 1.0 + (k as f32) * 0.1; // todos rentes, cada vez mais longe
        let base = 2 + k * 2;
        segs.push(seg(
            base,
            base + 1,
            (0.0, y),
            (100.0, y),
            1.0, // raio pequeno: adjacência só pelo raio do dono (2·6 = 12)
        ));
    }
    let extras = extras_for_stroke(&segs);
    // ⚠️ O teto TOTAL é a soma dos DOIS orçamentos — a fita própria (`MAX_RIBBON_EXTRAS`) e os
    // cruzamentos (`MAX_EXTRAS_PER_SEGMENT`) —, e é isso que o fragment itera. Este teste dizia
    // só `MAX_EXTRAS_PER_SEGMENT`, de quando a lista era uma só; o que ele PROVA continua sendo
    // o que importa: o laço do fragment nunca é ilimitado.
    assert_eq!(
        extras[0].list.len(),
        MAX_RIBBON_EXTRAS + MAX_EXTRAS_PER_SEGMENT,
        "a lista é capada pelos dois orçamentos"
    );
    // Os escolhidos são os mais PRÓXIMOS (os primeiros do pente).
    assert!(
        extras[0].list.contains(&1) && extras[0].list.contains(&2),
        "os mais próximos entram: {:?}",
        extras[0]
    );
}

mod ribbon_budget_measurement {
    use super::*;

    /// Constrói os segmentos de uma polilinha com raio uniforme (índices globais
    /// sequenciais, como o `pack` faz para um traço aberto).
    fn segs_of(pts: &[(f32, f32)], r: f32) -> Vec<Seg> {
        (0..pts.len() - 1)
            .map(|i| Seg {
                a: i as u32,
                b: i as u32 + 1,
                pa: Vec2::new(pts[i].0, pts[i].1),
                pb: Vec2::new(pts[i + 1].0, pts[i + 1].1),
                radius: r,
            })
            .collect()
    }

    /// Reamostra no passo do produto (`0.4 × largura = 0.8 · r`).
    fn densify(pts: &[(f32, f32)], step: f32) -> Vec<(f32, f32)> {
        let mut out = vec![pts[0]];
        for w in pts.windows(2) {
            let (a, b) = (w[0], w[1]);
            let d = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let n = (d / step).floor() as usize;
            for k in 1..=n {
                let t = k as f32 * step / d;
                if t < 1.0 {
                    out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
                }
            }
            out.push(b);
        }
        out
    }

    /// **A MEDIÇÃO que fixa `MAX_RIBBON_EXTRAS`.** Roda com
    /// `cargo test -p ph2d-flip-render ribbon_budget -- --nocapture`.
    ///
    /// A pergunta: quantos vizinhos da PRÓPRIA fita caem dentro do alcance de
    /// influência, na densidade que o produto de fato produz? O teto tem de ficar
    /// acima do pior caso REAL — e o pior caso não é uma reta, é a curvatura máxima
    /// que um pincel consegue desenhar (o raio de curvatura chega ao raio do pincel;
    /// abaixo disso a fita dobra sobre si mesma e vira cruzamento, não fita).
    #[test]
    fn measure_ribbon_budget() {
        const R: f32 = 5.0;
        let step = 0.8 * R;
        // ⚠️ Os arcos são gerados **no passo do produto** (comprimento de ARCO), não por
        // grau: uma varredura por grau num raio grande fica 4,6× mais densa que o que o
        // `resample_smooth` de fato produz, e mediria a fixture em vez do produto.
        let arc = |curv_r: f32, sweep_deg: f32| -> Vec<(f32, f32)> {
            let total = curv_r * sweep_deg.to_radians();
            let n = (total / step).max(2.0) as usize;
            (0..=n)
                .map(|k| {
                    let a = (k as f32 / n as f32) * sweep_deg.to_radians();
                    (curv_r * a.cos(), curv_r * a.sin())
                })
                .collect()
        };
        /// Um caso da sonda: rótulo, o caminho, e se ele deve ficar dentro do orçamento.
        type Case = (&'static str, Vec<(f32, f32)>, bool);
        let cases: Vec<Case> = vec![
            ("reta", vec![(0.0, 0.0), (200.0, 0.0)], true),
            ("arco raio 10·r", arc(10.0 * R, 90.0), true),
            ("arco raio 2·r (curvatura alta)", arc(2.0 * R, 180.0), true),
            ("arco raio 1·r (o limite do pincel)", arc(R, 360.0), true),
            (
                "hachura gap 0.5·r",
                (0..5)
                    .flat_map(|i| {
                        let x = i as f32 * 0.5 * R;
                        if i % 2 == 0 {
                            [(x, 0.0), (x, 80.0)]
                        } else {
                            [(x, 80.0), (x, 0.0)]
                        }
                    })
                    .collect(),
                true,
            ),
            // A MÃO LENTA: o RDP preserva pontos numa curva apertada, então a entrada
            // pode chegar mais densa que o passo da reamostragem (que só ACRESCENTA).
            (
                "curva com entrada 4× densa (mão lenta)",
                (0..=240)
                    .map(|k| {
                        let a = (k as f32 / 240.0) * std::f32::consts::PI;
                        (4.0 * R * a.cos(), 4.0 * R * a.sin())
                    })
                    .collect(),
                false,
            ),
        ];

        println!("\n  cenário                                 | pts | máx | média | densidade");
        println!("  ----------------------------------------|-----|-----|-------|----------");
        let mut worst_product = 0usize;
        for (name, spine, product_density) in cases {
            let pts = densify(&spine, step);
            let segs = segs_of(&pts, R);
            let mut stamp = vec![0u32; segs.len()];
            let mut walked = vec![0u32; segs.len()];
            let (mut max_n, mut sum) = (0usize, 0usize);
            for i in 0..segs.len() {
                let mut out = Vec::new();
                push_ribbon_local(
                    &segs,
                    i,
                    false,
                    R,
                    &mut stamp,
                    &mut walked,
                    i as u32 + 1,
                    &mut out,
                );
                max_n = max_n.max(out.len());
                sum += out.len();
            }
            if product_density {
                worst_product = worst_product.max(max_n);
            }
            println!(
                "  {name:<39} | {:>3} | {max_n:>3} | {:>5.1} | {}",
                segs.len(),
                sum as f32 / segs.len() as f32,
                if product_density {
                    "produto"
                } else {
                    "4x densa"
                }
            );
        }
        println!(
            "\n  pior caso na densidade do PRODUTO: {worst_product}   \
             MAX_RIBBON_EXTRAS = {MAX_RIBBON_EXTRAS}\n"
        );
        assert!(
            MAX_RIBBON_EXTRAS >= worst_product,
            "o teto ({MAX_RIBBON_EXTRAS}) tem de cobrir o pior caso na densidade que o \
             produto de fato produz ({worst_product}) — abaixo dele a fita local volta a \
             perder vizinhos e a mordida ressurge no traço NORMAL"
        );
    }
}
