//! ⭐⭐ **QUANTO CUSTA UMA PONTA A MAIS NUMA ESTRELA, E QUANTO CUSTA UM ELIPSÓIDE ACHATADO**
//! (W103) — as duas sondas que os números de [`ph2d_field::MAX_STAR_POINTS`] e da nota do
//! [`ph2d_field_eval::ops::sd_ellipsoid`] citam.
//!
//! ⚠️ **A régua é a MESMA do prisma** ([`measure_prism_sides`](measure_prism_sides.rs)): o cilindro
//! exato vale `1,00×`, porque é a forma redonda mais barata que este módulo tem e é contra ela que
//! o teto de lados foi calibrado. Comparar duas famílias com réguas diferentes não escolheria nada.
//!
//! ```text
//! cargo test -p ph2d-field-eval --release --test measure_star_points -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};

/// `(nós da árvore, ns por ponto)` — a mediana de 7 corridas sobre `2^18` pontos, com uma corrida a
/// frio antes de medir.
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

fn cost(p: Primitive) -> (usize, f64) {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça");
    cost_of(ph2d_field_eval::compile(&doc))
}

#[test]
#[ignore]
fn measure_star_points() {
    let cilindro = cost(Primitive::Cylinder {
        radius: 0.45,
        half_height: 0.3,
        round: 0.05,
        chamfer: 0.0,
    });
    println!("  forma                  | semiplanos |  nós |  ns/ponto | × o cilindro");
    println!(
        "{:>22} | {:>10} | {:4} | {:6.2} ns | {:11.2}×",
        "cilindro (exacto)", "-", cilindro.0, cilindro.1, 1.0
    );
    for n in [3_u32, 4, 5, 6, 8, 10, 12, 16, 24, 32] {
        // ⚠️ Acima do teto o documento recusa (é a cerca do produto), então a sonda monta a árvore
        // **directamente** — a mesma razão que a sonda do prisma registou: *uma sonda que só alcança
        // o lado de dentro do limite não pode dizer que o limite está no sítio certo*.
        let (nos, ns) = if n <= ph2d_field::MAX_STAR_POINTS {
            cost(Primitive::Star {
                points: n,
                outer: 0.45,
                inner: 0.18,
                half_height: 0.3,
                round: 0.02,
                chamfer: 0.0,
            })
        } else {
            cost_of(ph2d_field_eval::ops::sd_star(n, 0.45, 0.18, 0.3, 0.02, 0.0))
        };
        let cerca = if n > ph2d_field::MAX_STAR_POINTS {
            " (fora da cerca)"
        } else {
            ""
        };
        println!(
            "{:>22} | {:>10} | {nos:4} | {ns:6.2} ns | {:11.2}×{cerca}",
            format!("estrela de {n} pontas"),
            4 * n,
            ns / cilindro.1
        );
    }

    // ⭐ E as duas formas **sem contagem**, para o teto delas não ser um palpite tampouco.
    for (nome, p) in [
        (
            "gaiola de caixa",
            Primitive::BoxFrame {
                half: [0.45, 0.45, 0.45],
                thickness: 0.13,
                round: 0.02,
                chamfer: 0.0,
            },
        ),
        (
            "elipsóide 1:1:1",
            Primitive::Ellipsoid {
                radii: [0.45, 0.45, 0.45],
            },
        ),
        (
            "elipsóide 1:4",
            Primitive::Ellipsoid {
                radii: [0.45, 0.1125, 0.45],
            },
        ),
        (
            "elipsóide 1:16",
            Primitive::Ellipsoid {
                radii: [0.45, 0.028_125, 0.45],
            },
        ),
    ] {
        let (nos, ns) = cost(p);
        println!(
            "{nome:>22} | {:>10} | {nos:4} | {ns:6.2} ns | {:11.2}×",
            "-",
            ns / cilindro.1
        );
    }
}

/// ⭐⭐⭐ **O QUE UM ELIPSÓIDE ACHATADO CUSTA À MARCHA** — e este é o recurso de que o limite dele
/// seria feito, se ele tivesse um.
///
/// A [`ph2d_field_eval::ops::sd_ellipsoid`] é um **subestimador** por um fator que é exatamente
/// `min(s)/max(s)` na pior direção. Subestimar não erra: custa **passos**. Esta sonda conta os
/// passos de uma marcha de esferas até à superfície, a partir de fora, na direção em que o
/// subestimador é pior — e é isso que diz se um elipsóide de `1:16` é utilizável ou uma armadilha.
#[test]
#[ignore]
fn measure_ellipsoid_march() {
    use fidget::shape::EzShape;
    let passos = |radii: [f32; 3]| -> (usize, f64) {
        let doc = FieldDoc::new(
            vec![Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Ellipsoid { radii }),
            )],
            NodeId(0),
        )
        .expect("a peça");
        let shape = ph2d_field_eval::Engine::from(ph2d_field_eval::compile(&doc));
        let tape = shape.ez_float_slice_tape();
        let mut eval = ph2d_field_eval::Engine::new_float_slice_eval();
        // ⚠️ **Ao longo do eixo MAIOR** — é ali que o campo é escalado pelo menor semi-eixo, logo é
        // onde o subestimador é pior. Uma marcha pelo eixo menor mediria o caso fácil.
        let mut x = 1.5_f32;
        let mut n = 0;
        while n < 4096 {
            let d = eval.eval(&tape, &[x], &[0.0], &[0.0]).expect("avalia")[0];
            if d < 1.0e-4 {
                break;
            }
            x -= d;
            n += 1;
        }
        (n, f64::from(x))
    };
    println!("  elipsóide  | passos até à superfície | x final (o raio é 0,45)");
    for k in [
        1.0_f32,
        0.5,
        0.25,
        0.125,
        0.0625,
        0.031_25,
        0.015_625,
        0.007_812_5,
    ] {
        let (n, x) = passos([0.45, 0.45 * k, 0.45 * k]);
        println!("  1:{:>5.0}    | {n:>23} | {x:.6}", 1.0 / k);
    }
}

/// ⭐⭐⭐ **A FÓRMULA PUBLICADA DO ELIPSÓIDE, MEDIDA CONTRA A NOSSA** — porque «preferi uma prova»
/// não é motivo, e a alternativa tinha de ser experimentada.
///
/// A referência (IQ, *distance functions*) publica `k0·(k0−1)/k1`, com `k0 = |p/r|` e `k1 = |p/r²|`
/// — bem mais **apertada** que a nossa (a esfera reescalada por `min(r)`), e portanto mais barata de
/// marchar. Esta sonda mede as duas onde importa: o **centro da peça** e o **pior gradiente**.
///
/// ⚠️ `k1` é `0` na origem, e `k0` também: a fórmula é `0/0` **exatamente no centro do sólido**, que
/// é o ponto que toda grade centrada amostra. O piso do `sqrt` desta crate transforma o `NaN` num
/// **zero** — um zero no meio de uma peça é uma superfície fantasma para quem procura troca de
/// sinal.
#[test]
#[ignore]
fn measure_ellipsoid_against_the_published_formula() {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    fn iq(radii: [f64; 3]) -> Tree {
        let over = |t: Tree, r: f64| t * Tree::constant(1.0 / r);
        let sq = |t: Tree| t.square();
        let k0 = (sq(over(Tree::x(), radii[0]))
            + sq(over(Tree::y(), radii[1]))
            + sq(over(Tree::z(), radii[2])))
        .max(1.0e-30)
        .sqrt();
        let k1 = (sq(over(Tree::x(), radii[0] * radii[0]))
            + sq(over(Tree::y(), radii[1] * radii[1]))
            + sq(over(Tree::z(), radii[2] * radii[2])))
        .max(1.0e-30)
        .sqrt();
        k0.clone() * (k0 - Tree::constant(1.0)) / k1
    }
    let sonda = |tree: Tree| -> (f32, f64) {
        let shape = ph2d_field_eval::Engine::from(tree);
        let tape = shape.ez_float_slice_tape();
        let mut eval = ph2d_field_eval::Engine::new_float_slice_eval();
        let centro = eval.eval(&tape, &[0.0], &[0.0], &[0.0]).expect("avalia")[0];
        // O pior gradiente por diferenças centrais, numa nuvem à volta da peça.
        const H: f32 = 1.0e-3;
        let mut pior = 0.0_f64;
        let step = 2.0_f32 / 24.0;
        for i in 0..24_u8 {
            for j in 0..24_u8 {
                for k in 0..24_u8 {
                    let p = [
                        f32::from(i) * step - 1.0,
                        f32::from(j) * step - 1.0,
                        f32::from(k) * step - 1.0,
                    ];
                    // ⚠️ Os seis pontos do estêncil vão numa avaliação só, e cada eixo tem de
                    // aparecer nas TRÊS listas: a fita recebe triplos, não uma coordenada.
                    let xs = [p[0] - H, p[0] + H, p[0], p[0], p[0], p[0]];
                    let ys = [p[1], p[1], p[1] - H, p[1] + H, p[1], p[1]];
                    let zs = [p[2], p[2], p[2], p[2], p[2] - H, p[2] + H];
                    let Ok(o) = eval.eval(&tape, &xs, &ys, &zs) else {
                        continue;
                    };
                    let d = |a: usize| f64::from(o[a + 1] - o[a]);
                    let g = [d(0), d(2), d(4)];
                    let n = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt() / f64::from(2.0 * H);
                    if n.is_finite() && n > pior {
                        pior = n;
                    }
                }
            }
        }
        (centro, pior)
    };
    println!(
        "  elipsóide  |            f(centro) esperado |  a NOSSA f(0) | pior ‖∇f‖ |  a do IQ f(0) | pior ‖∇f‖"
    );
    for k in [1.0_f64, 0.5, 0.25, 0.125] {
        let radii = [0.45, 0.45 * k, 0.45 * k];
        let (c_nosso, g_nosso) = sonda(ph2d_field_eval::ops::sd_ellipsoid(radii));
        let (c_iq, g_iq) = sonda(iq(radii));
        println!(
            "  1:{:>5.0}    | {:29.6} | {c_nosso:13.6} | {g_nosso:9.4} | {c_iq:13.6} | {g_iq:9.4}",
            1.0 / k,
            -(0.45 * k)
        );
    }
}
