//! ⭐ **O PREÇO DE UMA BOSSA** — a sonda que decide o teto de [`ph2d_field::MAX_CLOUD_LOBES`].
//!
//! ⚠️ **O teto de uma contagem não se escolhe, mede-se** (`CLAUDE.md` §0.0), e ela usa a **mesma
//! régua** da engrenagem e da estrela: o preço contra o cilindro exato, com a estrela no tecto dela
//! ao lado como a forma mais cara que esta casa já shipa.
//!
//! ⚠️ **A coluna que manda é a CONTAGEM DE NÓS** — ela é determinística; um relógio nesta
//! workstation não vale nada acima de `load ~5` (§5.0), e entra só como confirmação.
//!
//! ⭐ **Uma bossa é UM disco**, contra os quatro semiplanos de uma ponta de estrela — e é por isso
//! que o teto dela não tinha de ser o da estrela.
//!
//! ```text
//! cargo test -p ph2d-field-eval --test measure_cloud_lobes -- --ignored --nocapture
//! ```

use ph2d_field_eval::{Field, ops, ops_balloons};

fn nodes(t: &fidget::context::Tree) -> usize {
    let mut ctx = fidget::context::Context::new();
    let _root = ctx.import(t);
    ctx.len()
}

fn ns_por_ponto(t: &fidget::context::Tree, amostras: usize) -> f64 {
    let f = Field::from_tree(t);
    let mut medidas = Vec::new();
    for _ in 0..5 {
        let t0 = std::time::Instant::now();
        let mut acc = 0.0_f64;
        for i in 0..amostras {
            #[allow(clippy::cast_precision_loss)]
            let a = i as f64 / amostras as f64 * 2.0 - 1.0;
            acc += f.at(a, a * 0.7, a * 0.3);
        }
        std::hint::black_box(acc);
        #[allow(clippy::cast_precision_loss)]
        medidas.push(t0.elapsed().as_secs_f64() * 1.0e9 / amostras as f64);
    }
    medidas.sort_by(f64::total_cmp);
    medidas[2]
}

#[test]
#[ignore = "sonda: imprime o preco por contagem de bossas"]
fn measure_cloud_lobes() {
    const N: usize = 20_000;
    let cilindro = ops::sd_cylinder(0.8, 0.2, 0.0, 0.0);
    let (nc, tc) = (nodes(&cilindro), ns_por_ponto(&cilindro, N));
    println!("\ncilindro: {nc} nos, {tc:.2} ns/ponto  (a referencia)\n");
    let estrela = ops::sd_star(ph2d_field::MAX_STAR_POINTS, 0.9, 0.5, 0.2, 0.0, 0.0);
    let (ne, te) = (nodes(&estrela), ns_por_ponto(&estrela, N));
    println!(
        "estrela no tecto ({} pontas): {ne} nos, {te:.0} ns/ponto = {:.2}x o cilindro\n",
        ph2d_field::MAX_STAR_POINTS,
        te / tc
    );
    println!(
        "{:>7} {:>8} {:>12} {:>14} {:>14}",
        "bossas", "nos", "ns/ponto", "x o cilindro", "x a estrela"
    );
    for n in [3_u32, 5, 8, 12, 16, 24, 32] {
        let c = ops_balloons::sd_cloud(n, 0.9, 0.45, 0.0, 0.2, 0.0, 0.0);
        let (nn, tt) = (nodes(&c), ns_por_ponto(&c, N));
        println!(
            "{n:>7} {nn:>8} {tt:>12.0} {:>13.2}x {:>13.2}x",
            tt / tc,
            tt / te
        );
    }
    println!();
}

/// ⭐⭐⭐ **O QUE DE FACTO LIMITA UMA NUVEM É A MARCHA, e não o preço.**
///
/// ⚠️ Numa união n-ária o tecto de `‖∇f‖` é `√(quantas peças estão ACTIVAS)`, e acima de
/// `passo × ‖∇f‖ = 1` **a marcha atravessa a superfície** — a peça sai furada. ⛔ E o raio da
/// mistura **não é a alavanca**: uma varredura de `0,50` a `0,10` moveu o número em `0,05`.
#[test]
#[ignore = "sonda: escolhe o tecto de bossas pela MARCHA"]
fn measure_cloud_blend() {
    use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
    println!("\n{:>8} {:>16}", "bossas", "passo x |grad|");
    for lobes in [3_u32, 4, 5, 6, 7, 8, 9, 10, 12, 16] {
        let mut pior = 0.0f64;
        // ⚠️ **Duas proporções**, e a barra é a PIOR: a da paleta (larga e baixa) e a do censo.
        for (hw, hs) in [(0.45_f64, 0.22_f64), (0.50, 0.25)] {
            let p = Primitive::Cloud {
                lobes,
                half_width: hw as f32,
                half_span: hs as f32,
                tail: 0.18,
                half_height: 0.10,
                round: 0.03,
                chamfer: 0.0,
            };
            let doc = FieldDoc::new(
                vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
                NodeId(0),
            )
            .expect("peça");
            let f = Field::new(&doc);
            let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
            let mut g = 0.0f64;
            let n = 70usize;
            #[allow(clippy::cast_precision_loss)]
            let at = |t: usize| -0.8 + 1.6 * (t as f64 + 0.5) / n as f64;
            for i in 0..n {
                for j in 0..n {
                    for k in 0..n {
                        let (x, y, z) = (at(i), at(j), at(k));
                        if f.at(x, y, z).abs() < 0.05 {
                            let gg = f.gradient_norm(x, y, z, 1.0e-4);
                            if gg.is_finite() {
                                g = g.max(gg);
                            }
                        }
                    }
                }
            }
            pior = pior.max(passo * g);
        }
        println!("{lobes:>8} {pior:>16.4}");
    }
    println!();
}

/// ⭐⭐⭐ **E A ESTRELA FURA NO TETO DELA** — o achado que o gate
/// `every_counted_shape_marches_safely_at_its_own_ceiling` devolveu em 2026-09-05.
///
/// ⚠️ O [`ph2d_field::MAX_STAR_POINTS`] foi escrito a partir do **preço** (*«a estrela chega ao
/// preço do prisma às 16 pontas»*), e ninguém correu a **marcha** lá. Esta sonda diz onde ela deixa
/// de ser segura.
#[test]
#[ignore = "sonda: onde a estrela deixa de marchar"]
fn measure_star_points_against_the_march() {
    use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
    println!("\n{:>7} {:>16} {:>16}", "pontas", "paleta", "censo");
    for pontas in [5_u32, 6, 7, 8, 9, 10, 12, 14, 16] {
        let mut col = Vec::new();
        // ⚠️ **As DUAS proporções que o produto tem**: a que a paleta cria e a do representante.
        for (outer, inner, hh, round) in [
            (0.5_f32, 0.2_f32, 0.175_f32, 0.05_f32),
            (0.45, 0.2, 0.12, 0.03),
        ] {
            let mut p = Primitive::Star {
                points: pontas,
                outer,
                inner,
                half_height: hh,
                round,
                chamfer: 0.0,
            };
            ph2d_field::clamp_round(&mut p);
            let doc = FieldDoc::new(
                vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
                NodeId(0),
            )
            .expect("peça");
            let f = Field::new(&doc);
            let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
            let mut g = 0.0f64;
            let n = 70usize;
            #[allow(clippy::cast_precision_loss)]
            let at = |t: usize| -0.8 + 1.6 * (t as f64 + 0.5) / n as f64;
            for i in 0..n {
                for j in 0..n {
                    for k in 0..n {
                        let (x, y, z) = (at(i), at(j), at(k));
                        if f.at(x, y, z).abs() < 0.05 {
                            let gg = f.gradient_norm(x, y, z, 1.0e-4);
                            if gg.is_finite() {
                                g = g.max(gg);
                            }
                        }
                    }
                }
            }
            col.push(passo * g);
        }
        println!("{pontas:>7} {:>16.4} {:>16.4}", col[0], col[1]);
    }
    println!();
}
