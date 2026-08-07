//! **Onde a curva do SSS assenta, e quanto ela de fato vaza** — a sonda que dá
//! os dois números que eu havia chutado.
//!
//! ```text
//! cargo test -p ph2d-mesh-render --release --test measure_sss_curve -- --ignored --nocapture
//! ```
//!
//! Duas constantes de `sss.rs` só podem sair daqui (`CLAUDE.md` §0):
//!
//! 1. **`T_MAX`** — o teto do eixo `t = scatter·|κ|`. Ele tem de cair onde a
//!    curva já assentou: mais alto gasta resolução onde não há informação, mais
//!    baixo corta antes de a resposta parar de mudar. ⚠️ O meu palpite era **2**,
//!    e o gate `the_table_saturates_within_its_own_range` nasceu vermelho nele.
//! 2. **A barra do vazamento no terminador** — eu escrevi `> 0,1` sem medir, e a
//!    resposta em `t = 1` é 0,070.

use ph2d_mesh_render::sss::integrate;

#[test]
#[ignore = "sonda de medição; roda sob demanda"]
fn measure_where_the_sss_curve_settles() {
    println!("\n=== O VAZAMENTO NO TERMINADOR (N·L = 0) por t = scatter·|kappa| ===");
    println!("{:>8} | {:>8} | {:>8} | {:>8}", "t", "R", "G", "B");
    let mut prev = [0.0f32; 3];
    for i in 0..=32 {
        let t = i as f32 * 0.5;
        let d = integrate(0.0, t);
        let delta = (0..3)
            .map(|c| (d[c] - prev[c]).abs())
            .fold(0.0f32, f32::max);
        println!(
            "{t:>8.2} | {:>8.4} | {:>8.4} | {:>8.4}   (mudou {delta:.4} desde a linha acima)",
            d[0], d[1], d[2]
        );
        prev = d;
    }

    println!("\n=== ONDE ELA ASSENTA: o t a partir do qual dobrar nao muda 1/255 ===");
    let step = 1.0 / 255.0;
    for t in [1.0f32, 2.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 32.0] {
        let here = integrate(0.0, t);
        let twice = integrate(0.0, t * 2.0);
        let worst = (0..3)
            .map(|c| (here[c] - twice[c]).abs())
            .fold(0.0f32, f32::max);
        let verdict = if worst <= step {
            "ASSENTOU"
        } else {
            "ainda anda"
        };
        println!("  t {t:>5.1} -> 2t: pior canal move {worst:.5}  ({verdict})");
    }

    println!(
        "\n=== O PERFIL AO LONGO DO TERMINADOR (t fixo, N·L varrendo) ===\n\
         O que se ve num rosto: onde o lambert JA' e' zero, o vermelho ainda sobra."
    );
    for t in [0.5f32, 1.0, 2.0, 4.0] {
        print!("  t {t:>4.1}: ");
        for i in 0..7 {
            let n_dot_l = -0.3 + i as f32 * 0.1;
            let d = integrate(n_dot_l, t);
            print!("[{n_dot_l:+.1} R{:.3} L{:.3}] ", d[0], n_dot_l.max(0.0));
        }
        println!();
    }
}

/// **QUE `scatter` TORNA O CANAL VISÍVEL NUMA PEÇA REAL** — a sonda que decide o
/// `SCATTER_FRACTION`, que eu havia chutado em 0,02.
///
/// ⚠️ O eixo é `t = scatter·|κ|`, e `|κ|` de uma escultura **não é o da peça
/// inteira**: é o das FEATURES (um vinco, uma ponta, uma dobra), que são muito
/// mais curvas que o corpo. Escolher a fração pelo tamanho da peça sem olhar a
/// distribuição de curvatura é escolher pelo raio errado.
#[test]
#[ignore = "sonda de medição; roda sob demanda"]
fn measure_which_scatter_makes_the_channel_visible() {
    use ph2d_mesh::{QueryScratch, RegionScratch, shapes};

    let mut m = shapes::uv_sphere(48, 72, 1.0);
    m.triangulate();
    let mut q = QueryScratch::default();
    let mut scratch = RegionScratch::default();
    let mut moved = Vec::new();
    for i in 0..7usize {
        let seed = (i * 7919) % m.vert_count();
        let center = m.positions()[seed];
        let radius = 0.10 + 0.06 * (i % 3) as f32;
        let push = if i % 2 == 0 { 0.06 } else { -0.05 };
        m.verts_in_sphere(center, radius, &mut q, &mut moved);
        let hits: Vec<u32> = moved.clone();
        for &v in &hits {
            let n = m.normals()[v as usize];
            let p = m.positions()[v as usize];
            let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
            let t = 1.0 - (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() / radius;
            let w = t.max(0.0).powi(2);
            let pm = &mut m.positions_mut()[v as usize];
            pm[0] += n[0] * push * w;
            pm[1] += n[1] * push * w;
            pm[2] += n[2] * push * w;
        }
        m.refresh_region(&hits, &mut scratch);
    }

    let mut k: Vec<f32> = m.curv_world().iter().map(|x| x.abs()).collect();
    k.sort_by(f32::total_cmp);
    let pct = |p: f32| k[((k.len() as f32 - 1.0) * p) as usize];
    let b = m.bounds();
    let longest = (b.max[0] - b.min[0])
        .max(b.max[1] - b.min[1])
        .max(b.max[2] - b.min[2]);
    println!("\n=== |kappa| de uma esfera de raio 1 com sete tracos (maior lado {longest:.2}) ===");
    println!(
        "  mediana {:.3}   p75 {:.3}   p90 {:.3}   p99 {:.3}   max {:.3}",
        pct(0.5),
        pct(0.75),
        pct(0.90),
        pct(0.99),
        k[k.len() - 1]
    );

    println!("\n=== O QUE CADA FRACAO PRODUZ (t = fracao * maior_lado * |kappa|) ===");
    println!(
        "{:>10} | {:>10} | {:>10} | {:>10} | {:>28}",
        "fracao", "t mediano", "t p90", "t p99", "vaza no terminador (mediana)"
    );
    for frac in [0.02f32, 0.05, 0.10, 0.20, 0.40] {
        let s = frac * longest;
        let tm = s * pct(0.5);
        let bleed = integrate(0.0, tm)[0];
        println!(
            "{frac:>10.2} | {tm:>10.3} | {:>10.3} | {:>10.3} | {bleed:>28.4}",
            s * pct(0.90),
            s * pct(0.99)
        );
    }
    println!(
        "\nLEITURA: a fracao certa poe o `t` da SUPERFICIE TIPICA na faixa em que a\n\
         curva de fato anda (0,5 a 2). Abaixo disso o canal existe e nao se ve.\n"
    );
}
