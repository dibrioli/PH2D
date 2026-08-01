//! **A sonda que decide o K1** (`docs/3D/03.5`): *um dab numa malha de 5 M
//! triângulos passa de 8 ms na CPU?* Se passar, o kernel migra para a GPU atrás
//! da porta única e a CPU vira o oráculo de paridade. Se não passar, a hipótese
//! "a CPU basta" deixa de ser hipótese.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_brush_kernel -- --nocapture
//! ```
//!
//! ⚠️ **`--release` não é preferência.** O kernel é aritmética por-vértice; em
//! debug ele mede o `opt-level=0`, não o produto — o mesmo motivo pelo qual o
//! smoke do AI Denoise exige release.
//!
//! # Como esta sonda mede
//!
//! **Ablação pelas PORTAS DO PRODUTO, nunca por instrumentação.** A decomposição
//! sai de chamar `verts_in_sphere` e `refresh_region` — que são portas públicas
//! que o traço usa — em vez de cronometrar de dentro dele. Uma sonda que
//! re-implementa o laço fica CEGA à porta e segue reportando o custo antigo
//! depois de o produto parar de pagá-lo (a lição que a `line/Painter` pagou em
//! 2026-07-26).
//!
//! E ela **anda o pincel**, em vez de bater no mesmo lugar: um dab repetido no
//! mesmo ponto mede o cache quente daquela vizinhança, que não é o que a mão faz.
//!
//! ⚠️ **Ela re-começa o traço a cada dab, e isso é o LIMITE SUPERIOR de propósito.**
//! Dentro de um traço vivo, um dab que reincide sobre vértices já capturados
//! custa MENOS (a captura não repete, e o envelope descarta quem não subiu) — é
//! a lei do traço trabalhando. O número que decide o K1 tem de ser o do dab que
//! faz trabalho cheio, senão a sobreposição da fixture o maquia para baixo.
//!
//! ⚠️ **Os números da W1 NÃO foram herdados.** Aquela medição era do `apply_dab`,
//! que o traço subsumiu; manter os dois seria a segunda porta para *"aplicar um
//! dab"*. O caminho de hoje faz estritamente mais trabalho (captura + envelope +
//! alvo), e a tabela abaixo é a re-medição.

use std::time::Instant;

use ph2d_mesh::{Mesh, QueryScratch, RegionScratch, shapes};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// O teto que o `docs/3D/03.5` declara para um dab. Não é inventado: é o mesmo
/// orçamento que o Painter usa para um *move* de pincel, porque é o mesmo gesto
/// humano com a mesma expectativa.
const K1_BUDGET_MS: f64 = 8.0;

/// Medianas de `n` amostras, descartando a primeira (ela paga o *first-touch*
/// dos buffers e é estruturalmente diferente das outras — o redutor é parte da
/// fixture).
fn median(mut v: Vec<f64>) -> f64 {
    if v.len() > 1 {
        v.remove(0);
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Um caminho de dabs sobre a superfície, como uma pincelada de verdade.
fn stroke_centers(mesh: &Mesh, count: usize) -> Vec<[f32; 3]> {
    let n = mesh.vert_count();
    // Amostra vértices espalhados por um passo primo, para o traço atravessar a
    // malha em vez de ficar num anel.
    (0..count)
        .map(|i| mesh.positions()[(i * 7919) % n])
        .collect()
}

struct Row {
    tris: usize,
    radius_frac: f32,
    affected: usize,
    dab_ms: f64,
    query_ms: f64,
    normals_ms: f64,
}

/// Força minúscula: a sonda mede CUSTO, e deformar a malha ao longo da
/// varredura mudaria a vizinhança de um dab para o seguinte.
fn probe_brush(verb: Verb, radius: f32) -> Brush {
    Brush {
        verb,
        radius,
        strength: 1e-4,
        ..Brush::default()
    }
}

fn measure(mesh: &mut Mesh, radius_frac: f32, dabs: usize) -> Row {
    let radius = mesh.bounds().longest_edge() * radius_frac;
    let centers = stroke_centers(mesh, dabs);
    let brush = probe_brush(Verb::Draw, radius);
    let mut stroke = SculptStroke::default();

    // (a) o dab COMPLETO, pela porta do produto.
    let mut full = Vec::with_capacity(dabs);
    let mut affected = 0usize;
    for c in &centers {
        stroke.begin(mesh);
        let dab = Dab::at(*c, radius, eye_towards(*c));
        let t = Instant::now();
        let moved = stroke.dab(mesh, &brush, &dab, Symmetry::default());
        full.push(t.elapsed().as_secs_f64() * 1e3);
        affected = affected.max(moved);
    }

    // (b) só a CONSULTA, pela porta do produto.
    let mut q = QueryScratch::default();
    let mut hits = Vec::new();
    let mut query = Vec::with_capacity(dabs);
    for c in &centers {
        let t = Instant::now();
        mesh.verts_in_sphere(*c, radius, &mut q, &mut hits);
        query.push(t.elapsed().as_secs_f64() * 1e3);
    }

    // (c) só as NORMAIS da região — o K2 —, sobre o mesmo conjunto afetado.
    let mut region = RegionScratch::default();
    let mut normals = Vec::with_capacity(dabs);
    for c in &centers {
        mesh.verts_in_sphere(*c, radius, &mut q, &mut hits);
        let t = Instant::now();
        mesh.refresh_region(&hits, &mut region);
        normals.push(t.elapsed().as_secs_f64() * 1e3);
    }

    Row {
        tris: mesh.triangle_count(),
        radius_frac,
        affected,
        dab_ms: median(full),
        query_ms: median(query),
        normals_ms: median(normals),
    }
}

#[test]
fn measure_brush_kernel() {
    println!("\n=== ph2d-sculpt3d :: custo de UM dab (CPU, serial) ===\n");
    println!(
        "{:>10} {:>8} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "triangulos", "raio", "vertices", "dab ms", "consulta", "normais", "resto"
    );

    let mut rows = Vec::new();
    for target in [100_000usize, 1_000_000, 5_000_000] {
        let build = Instant::now();
        let mut mesh = shapes::sphere_with_triangles(target, 1.0);
        let build_ms = build.elapsed().as_secs_f64() * 1e3;
        for frac in [0.02f32, 0.10, 0.30] {
            let r = measure(&mut mesh, frac, 12);
            println!(
                "{:>10} {:>7.0}% {:>10} {:>10.3} {:>10.3} {:>10.3} {:>10.3}",
                r.tris,
                r.radius_frac * 100.0,
                r.affected,
                r.dab_ms,
                r.query_ms,
                r.normals_ms,
                (r.dab_ms - r.query_ms - r.normals_ms).max(0.0)
            );
            rows.push(r);
        }
        println!("           (construir esta malha: {build_ms:.0} ms — user-paced, fora do dab)");
    }

    let worst = rows
        .iter()
        .fold(&rows[0], |a, b| if b.dab_ms > a.dab_ms { b } else { a });
    println!(
        "\nPIOR dab: {:.3} ms ({} triangulos, raio {:.0}%, {} vertices) contra o K1 de {K1_BUDGET_MS} ms",
        worst.dab_ms,
        worst.tris,
        worst.radius_frac * 100.0,
        worst.affected
    );

    if worst.dab_ms >= K1_BUDGET_MS {
        println!(
            "  >> O K1 DISPARA neste regime (pincel de {:.0}% do modelo = {} vertices, \
             {:.0}% da malha). Decisão de arquitetura do Enio (docs/3D/03.5).",
            worst.radius_frac * 100.0,
            worst.affected,
            100.0 * worst.affected as f64 / (worst.tris as f64 / 2.0)
        );
    }

    // ⚠️ **O gate é a FORMA, não o relógio.** O teto de 8 ms é uma decisão de
    // produto em aberto — que pincel o artista de fato usa em que densidade —, e
    // travá-lo aqui deixaria um número que eu escrevi sem dado governar o build.
    // O que **não** é negociável é a forma: um dab é limitado pela PEGADA.
    //
    // ⚠️ E a fixture teve de ser refeita para perguntar isso. Comparar linhas de
    // mesma FRAÇÃO de raio compara footprints DIFERENTES (a malha mais densa tem
    // mais vértices sob o mesmo pincel), então aquela razão media densidade, não
    // varredura. O experimento certo FIXA a pegada.
    let mut fixed = Vec::new();
    for target in [500_000usize, 5_000_000] {
        let mut mesh = shapes::sphere_with_triangles(target, 1.0);
        let mut best = (f32::MAX, 0.02f32);
        let mut stroke = SculptStroke::default();
        let probe_at = mesh.positions()[0];
        let span = mesh.bounds().longest_edge();
        for k in 1..=60 {
            let frac = k as f32 * 0.005;
            let r = span * frac;
            stroke.begin(&mesh);
            let n = stroke.dab(
                &mut mesh,
                &probe_brush(Verb::Draw, r),
                &Dab::at(probe_at, r, eye_towards(probe_at)),
                Symmetry::default(),
            );
            let err = (n as f32 - 30_000.0).abs();
            if err < best.0 {
                best = (err, frac);
            }
        }
        let r = measure(&mut mesh, best.1, 12);
        println!(
            "pegada fixa: {:>8} triangulos · raio {:.1}% · {} vertices · {:.3} ms",
            r.tris,
            r.radius_frac * 100.0,
            r.affected,
            r.dab_ms
        );
        fixed.push(r.dab_ms);
    }
    let growth = fixed[1] / fixed[0];
    println!("10x a malha, MESMA pegada: {growth:.2}x de custo\n");
    assert!(
        growth < 3.0,
        "10x a malha custou {growth:.2}x com a MESMA pegada — isso é assinatura \
         de varredura de malha, e é o que mata a malha grande"
    );
}

/// O olho de uma sonda: de fora, olhando direto para o dab — o mesmo caso que a
/// suíte de gates usa, para o conjunto frontal não mudar o que se está medindo.
fn eye_towards(c: [f32; 3]) -> [f32; 3] {
    let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    if l > 1e-6 {
        [-c[0] / l, -c[1] / l, -c[2] / l]
    } else {
        [0.0, 0.0, -1.0]
    }
}
