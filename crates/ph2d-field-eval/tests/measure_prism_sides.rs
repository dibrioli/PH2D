//! ⭐⭐ **QUANTO CUSTA UM LADO A MAIS NUM PRISMA** (W101) — a sonda que o teto de
//! [`ph2d_field::MAX_PRISM_SIDES`] cita.
//!
//! # ⚠️ Ela existe porque eu ia escrever a razão errada
//!
//! A resposta óbvia para *«porquê um teto de lados?»* é *«cada lado é um plano na árvore, logo é
//! custo»*. O `spike_formula_vs_profile` já tinha medido, noutra família, que a contagem de nós
//! **não** é o relógio (`7,00×` os nós dá `1,21×` o tempo — a fita corre com JIT em oito faixas de
//! SIMD, e o que custa é o caminho crítico). Escrever «é custo» sem medir seria repetir o erro que
//! aquela sonda acabou de pagar.
//!
//! ```text
//! cargo test -p ph2d-field-eval --release --test measure_prism_sides -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};

/// `(nós da árvore, ns por ponto)` — a mediana de 7 corridas sobre `2^18` pontos, com uma corrida a
/// frio antes de medir (a primeira paga o `mmap` da fita).
fn cost(p: Primitive) -> (usize, f64) {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça");
    cost_of(ph2d_field_eval::compile(&doc))
}

/// ⭐ **A MESMA sonda, mas a partir da árvore** — e é ela que consegue olhar **para além da cerca**.
///
/// ⚠️ A primeira versão media pelo documento e **entrou em pânico a 48 lados**: o
/// `FieldDoc::new` recusa acima do [`ph2d_field::MAX_PRISM_SIDES`], que é precisamente o número que
/// esta sonda existe para justificar. *Uma sonda que só alcança o lado de dentro do limite não pode
/// dizer que o limite está no sítio certo* — o `ops::sd_prism` não valida, e é por ele que se
/// espreita o outro lado.
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
fn measure_prism_sides() {
    let cilindro = cost(Primitive::Cylinder {
        radius: 0.45,
        half_height: 0.3,
        round: 0.05,
        chamfer: 0.0,
    });
    println!("  forma                  |  nós |  ns/ponto | × o cilindro | desvio da quina");
    println!(
        "{:>22} | {:4} | {:6.2} ns | {:11.2}× | {:>15}",
        "cilindro (exacto)", cilindro.0, cilindro.1, 1.0, "-"
    );
    for n in [3_u32, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96] {
        // ⚠️ Acima do teto o documento recusa (é a cerca do produto), então a sonda passa a montar
        // a árvore **directamente** — ver [`cost_of`].
        let (nos, ns) = if n <= ph2d_field::MAX_PRISM_SIDES {
            cost(Primitive::Prism {
                sides: n,
                bottom: 0.45,
                top: 0.45,
                half_height: 0.3,
                round: 0.05,
                chamfer: 0.0,
            })
        } else {
            cost_of(ph2d_field_eval::ops::sd_prism(
                n, 0.45, 0.45, 0.3, 0.05, 0.0,
            ))
        };
        let desvio = 1.0 - (std::f64::consts::PI / f64::from(n)).cos();
        let cerca = if n > ph2d_field::MAX_PRISM_SIDES {
            " (fora da cerca)"
        } else {
            ""
        };
        println!(
            "{:>22} | {nos:4} | {ns:6.2} ns | {:11.2}× | {:14.2}%{cerca}",
            format!("prisma de {n} lados"),
            ns / cilindro.1,
            desvio * 100.0
        );
    }
}
