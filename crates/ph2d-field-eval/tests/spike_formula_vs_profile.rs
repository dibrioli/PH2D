//! ⭐⭐⭐ **SPIKE (2026-08-28): uma forma DESENHADA custa mesmo muito mais que uma de FÓRMULA?**
//!
//! Pergunta do Enio: *«há alguns dias você tinha referido que um objeto criado através de desenho
//! vetorial custa muito mais caro que um criado por fórmula. Isso é correto?»*
//!
//! ⚠️ **A pergunta não se responde de memória, e a régua óbvia MENTE.** O
//! [`spike_per_edge_radius`](spike_per_edge_radius.rs) mediu nesta mesma sessão que `7,00×` os nós
//! de árvore são `1,21×` o relógio — a fita corre com JIT em oito faixas de SIMD, e o que custa é o
//! **caminho crítico**, não a contagem. ⇒ *citar «26 nós por aresta» como preço seria repetir o
//! erro que a wave anterior acabou de pagar.*
//!
//! Corre com:
//! `cargo test -p ph2d-field-eval --test spike_formula_vs_profile -- --ignored --nocapture`

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Profile, Xform, profile::FillRule};
use ph2d_field_eval::compile;

/// Um `n`-gono regular inscrito no raio `r`.
fn ngon(n: usize, r: f64) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(r * a.cos()) as f32, (r * a.sin()) as f32]
        })
        .collect()
}

fn extrusion(n: usize) -> FieldDoc {
    let profile = Profile::new(vec![ngon(n, 0.5)], FillRule::NonZero, 1.0e-3).expect("perfil");
    doc(Primitive::Extrude {
        profile,
        half_height: 0.4,
        round: 0.0,
        chamfer: 0.0,
    })
}

fn doc(p: Primitive) -> FieldDoc {
    FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("uma folha")
}

/// `(nós da árvore, ns por ponto)` — a mediana de 7 corridas sobre `2^18` pontos.
///
/// ⚠️ **Mediana e não média**: uma corrida presa atrás do escalonador arrasta a média e não move a
/// mediana. E uma corrida **a frio** antes de medir, porque a primeira paga o `mmap` da fita.
fn cost(d: &FieldDoc) -> (usize, f64) {
    use fidget::shape::EzShape;
    const N: usize = 1 << 18;
    let coord =
        |i: usize, k: usize| -0.9 + 1.8 * (((i * 7919 + k * 104_729) % 1024) as f32) / 1024.0;
    let xs: Vec<f32> = (0..N).map(|i| coord(i, 0)).collect();
    let ys: Vec<f32> = (0..N).map(|i| coord(i, 1)).collect();
    let zs: Vec<f32> = (0..N).map(|i| coord(i, 2)).collect();

    let tree = compile(d);
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

/// ⭐⭐⭐ **O PREÇO DE UMA FORMA DESENHADA CONTRA UMA DE FÓRMULA.**
///
/// ⚠️ `#[ignore]`: é medição, não afirmação — e nenhuma leitura de relógio desta workstation vale
/// acima de `load ~5`. O que se lê é a **razão**, medida no mesmo processo, back-to-back.
#[test]
#[ignore]
fn measure_formula_against_drawn() {
    println!("  forma                       |  nós |  ns/ponto | × a esfera");
    let base = cost(&doc(Primitive::Sphere { radius: 0.5 })).1;
    let linha = |nome: &str, (nos, ns): (usize, f64)| {
        println!("{nome:>29} | {nos:4} | {ns:6.2} ns | {:6.2}×", ns / base);
    };
    linha(
        "esfera (fórmula)",
        cost(&doc(Primitive::Sphere { radius: 0.5 })),
    );
    linha(
        "caixa (fórmula)",
        cost(&doc(Primitive::Box {
            half: [0.4; 3],
            round: 0.0,
            chamfer: 0.0,
        })),
    );
    linha(
        "caixa arredondada (fórmula)",
        cost(&doc(Primitive::Box {
            half: [0.4; 3],
            round: 0.08,
            chamfer: 0.0,
        })),
    );
    linha(
        "cilindro (fórmula)",
        cost(&doc(Primitive::Cylinder {
            radius: 0.4,
            half_height: 0.5,
            round: 0.0,
            chamfer: 0.0,
        })),
    );
    linha(
        "toro (fórmula)",
        cost(&doc(Primitive::Torus {
            major: 0.4,
            minor: 0.12,
        })),
    );
    for n in [6_usize, 12, 24, 48, 96, 192] {
        linha(
            &format!("extrusão DESENHADA, {n} lados"),
            cost(&extrusion(n)),
        );
    }
}

/// ⭐⭐ **E o CONTROLO que a pergunta pede: a MESMA forma pelos dois caminhos.**
///
/// Um cilindro é exprimível por fórmula **e** por um contorno desenhado de `n` lados. Comparar as
/// duas é a única leitura honesta de *«desenhar custa mais»* — comparar uma esfera com uma extrusão
/// de 96 lados mede a **complexidade da forma**, não o **caminho**.
#[test]
#[ignore]
fn measure_the_same_shape_by_both_routes() {
    let formula = cost(&doc(Primitive::Cylinder {
        radius: 0.5,
        half_height: 0.4,
        round: 0.0,
        chamfer: 0.0,
    }));
    println!("  o MESMO cilindro pelos dois caminhos");
    println!("  caminho                     |  nós |  ns/ponto | × a fórmula");
    println!(
        "{:>29} | {:4} | {:6.2} ns | {:6.2}×",
        "fórmula", formula.0, formula.1, 1.0
    );
    for n in [6_usize, 12, 24, 48, 96, 192] {
        let (nos, ns) = cost(&extrusion(n));
        println!(
            "{:>29} | {nos:4} | {ns:6.2} ns | {:6.2}×",
            format!("desenhado, {n} lados"),
            ns / formula.1
        );
    }
}
