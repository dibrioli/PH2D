//! ⭐⭐ **QUANTO CUSTA UM VÉRTICE A MAIS NUM POLÍGONO** (W132) — a sonda que o teto de
//! [`ph2d_field::MAX_POLYGON_VERTICES`] cita.
//!
//! # ⚠️ Ela mede DUAS coisas, porque o polígono tem dois recursos e nenhum é óbvio
//!
//! 1. **O relógio.** A distância a um polígono é `min` sobre os segmentos com o sinal do
//!    enrolamento, e as duas metades crescem com `N`. ⚠️ **A lição do prisma não transfere de
//!    graça**: lá a nota dizia *«a contagem de nós não é o relógio»* e a medição refutou-a, porque
//!    uma cadeia de `max` faz o **caminho crítico** crescer linearmente. Aqui a forma da árvore é
//!    outra (um `min` sobre distâncias, não uma cadeia de semiplanos), então a pergunta reabre.
//! 2. **A linha do painel.** Cada vértice são **duas** linhas, e o painel do módulo tem uma família
//!    de ids de tamanho fixo (`MAX_ROWS`). *Um recurso que não é tempo continua a ser um recurso*
//!    (`CLAUDE.md` §0), e este é o que costuma morder primeiro.
//!
//! ```text
//! cargo test -p ph2d-field-eval --release --test measure_polygon_vertices -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};

/// Um polígono de `n` vértices, **irregular de propósito** — um regular seria o prisma, cuja árvore
/// é outra (semiplanos), e mediria a forma errada.
fn poligono(n: u32) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let t = 2.0 * std::f32::consts::PI * (i as f32) / (n as f32);
            let r = 0.30 + 0.10 * (3.0 * t + 0.7).sin() + 0.05 * (5.0 * t + 1.9).cos();
            [r * t.cos(), r * t.sin()]
        })
        .collect()
}

/// `(nós da árvore, ns por ponto)` — a mediana de 7 corridas sobre `2^18` pontos, com uma corrida a
/// frio antes de medir. É a **mesma** régua do `measure_prism_sides`, de propósito: os dois números
/// só se comparam se a sonda for a mesma.
fn cost(p: Primitive) -> (usize, f64) {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça");
    cost_of(ph2d_field_eval::compile(&doc))
}

fn cost_of(tree: fidget::context::Tree) -> (usize, f64) {
    use fidget::shape::EzShape;
    const N: usize = 1 << 18;
    let coord =
        |i: usize, k: usize| -0.9 + 1.8 * (((i * 7919 + k * 104_729) % 1024) as f32) / 1024.0;
    let xs: Vec<f32> = (0..N).map(|i| coord(i, 0)).collect();
    let ys: Vec<f32> = (0..N).map(|i| coord(i, 1)).collect();
    let zs: Vec<f32> = (0..N).map(|i| coord(i, 2)).collect();

    let mut ctx = fidget::context::Context::new();
    let _ = ctx.import(&tree);
    let nos = ctx.len();

    let shape = ph2d_field_eval::Engine::from(tree);
    let tape = shape.ez_float_slice_tape();
    let mut eval = ph2d_field_eval::Engine::new_float_slice_eval();
    let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
    let mut a: Vec<f64> = (0..7)
        .map(|_| {
            let t0 = std::time::Instant::now();
            let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
            t0.elapsed().as_secs_f64() * 1.0e9 / N as f64
        })
        .collect();
    a.sort_by(f64::total_cmp);
    (nos, a[3])
}

#[test]
#[ignore]
fn measure_polygon_vertices() {
    println!(
        "  load: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let cilindro = cost(Primitive::Cylinder {
        radius: 0.45,
        half_height: 0.3,
        round: 0.05,
        chamfer: 0.0,
    });
    println!("  vértices |  nós |  ns/ponto | × o cilindro | linhas de painel (2N+4, de 64)");
    println!(
        "{:>10} | {:4} | {:6.2} ns | {:11.2}× | {:>10}",
        "cilindro", cilindro.0, cilindro.1, 1.0, "-"
    );
    for n in [3_u32, 4, 5, 6, 8, 12, 16, 24, 32, 48, 64] {
        // ⚠️ Acima do teto o documento **recusa** (é a cerca que esta sonda existe para justificar),
        // então ela monta a árvore directamente — a mesma saída que o `primitive_tree_vertices` usa.
        let profile = ph2d_field::polygon_profile(poligono(n)).expect("o contorno");
        let (nos, ns) = cost_of(ph2d_field_eval::profile::sd_extrude(
            &profile, 0.3, 0.05, 0.0,
        ));
        println!(
            "{n:>10} | {nos:4} | {ns:6.2} ns | {:11.2}× | {:>10}",
            ns / cilindro.1,
            2 * n + 4
        );
    }
}
