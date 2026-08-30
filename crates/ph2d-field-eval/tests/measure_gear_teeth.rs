//! ⭐ **O PREÇO DE UM DENTE** — a sonda que decide o teto de [`ph2d_field::MAX_GEAR_TEETH`].
//!
//! ⚠️ **O teto de uma contagem não se escolhe, mede-se** (`CLAUDE.md` §0.0). O da estrela saiu de
//! uma tabela que compara o preço dela com o do prisma, que por sua vez shipa a `3,80×` o cilindro;
//! esta faz a mesma pergunta para a engrenagem, cuja construção é a mesma (um corpo, mais uma peça
//! convexa por dente, unidas com filete).
//!
//! ⚠️ **A coluna que manda é a CONTAGEM DE NÓS**, e não um relógio: ela é determinística, e um
//! relógio nesta workstation não vale nada acima de `load ~5` (§5.0). O relógio entra ao lado, como
//! confirmação, com a mediana de várias corridas.
//!
//! ```text
//! cargo test -p ph2d-field-eval --test measure_gear_teeth -- --ignored --nocapture
//! ```

use ph2d_field_eval::{Field, ops, ops_plates};

/// Quantos nós a árvore tem — o `Field` compila-a, e é o que o traçador paga por amostra.
fn nodes(t: &fidget::context::Tree) -> usize {
    let mut ctx = fidget::context::Context::new();
    let _root = ctx.import(t);
    ctx.len()
}

fn ns_por_ponto(t: &fidget::context::Tree, amostras: usize) -> f64 {
    let f = Field::from_tree(t);
    // Aquece, depois mede a MEDIANA de cinco corridas — uma média deixa um pico de carga passar.
    let mut medidas = Vec::new();
    for _ in 0..5 {
        let t0 = std::time::Instant::now();
        let mut acc = 0.0_f64;
        for i in 0..amostras {
            let a = i as f64 / amostras as f64 * 2.0 - 1.0;
            acc += f.at(a, a * 0.7, a * 0.3);
        }
        std::hint::black_box(acc);
        medidas.push(t0.elapsed().as_secs_f64() * 1.0e9 / amostras as f64);
    }
    medidas.sort_by(f64::total_cmp);
    medidas[2]
}

#[test]
#[ignore = "sonda: imprime o preco por contagem de dentes"]
fn measure_gear_teeth() {
    const N: usize = 20_000;
    let cilindro = ops::sd_cylinder(0.8, 0.2, 0.0);
    let (nc, tc) = (nodes(&cilindro), ns_por_ponto(&cilindro, N));
    println!("\ncilindro: {nc} nos, {tc:.2} ns/ponto  (a referencia que o prisma usa)\n");
    // ⭐⭐⭐ **AS DUAS BARRAS QUE A CASA JÁ ACEITOU** — e é contra elas que este teto se mede, não
    // contra um número novo. O prisma shipa a `MAX_PRISM_SIDES` e a estrela a `MAX_STAR_POINTS`.
    let prisma = ops::sd_prism(ph2d_field::MAX_PRISM_SIDES, 0.8, 0.8, 0.2, 0.0);
    let estrela = ops::sd_star(ph2d_field::MAX_STAR_POINTS, 0.9, 0.5, 0.2, 0.0);
    // ⚠️ **Medidas na MESMA corrida que a engrenagem** — comparar com um número colhido noutra
    // corrida somaria os dois ruídos (a lição do A/B desta casa).
    let (np, tp) = (nodes(&prisma), ns_por_ponto(&prisma, N));
    let (ne, te) = (nodes(&estrela), ns_por_ponto(&estrela, N));
    println!(
        "prisma no tecto ({} lados): {np} nos, {tp:.0} ns/ponto = {:.2}x o cilindro",
        ph2d_field::MAX_PRISM_SIDES,
        tp / tc
    );
    println!(
        "estrela no tecto ({} pontas): {ne} nos, {te:.0} ns/ponto = {:.2}x o cilindro\n",
        ph2d_field::MAX_STAR_POINTS,
        te / tc
    );
    println!("(a barra e' a ESTRELA: e' a forma mais cara que esta casa ja' shipa)\n");
    println!(
        "{:>7} {:>8} {:>12} {:>14}",
        "dentes", "nos", "ns/ponto", "x o cilindro"
    );
    println!("{:>46}", "^ e x a ESTRELA no tecto dela");
    for n in [6_u32, 8, 12, 16, 24, 32, 48, 64] {
        let g = ops_plates::sd_gear(n, 0.6, 0.9, 0.5, 0.2, 0.0);
        let (nn, tt) = (nodes(&g), ns_por_ponto(&g, N));
        println!(
            "{n:>7} {nn:>8} {tt:>12.0} {:>13.2}x {:>12.2}x",
            tt / tc,
            tt / te
        );
    }
    println!();
}
