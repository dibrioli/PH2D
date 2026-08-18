//! **O QUE FALTA AO SHARPEN, E QUANTO ELE CUSTA** — a sonda que decide o
//! `INTENSIFY` e o `SHARPEN_MAX`, pela porta que **não passa pelo clamp**.
//!
//! ⚠️ **A tabela anterior mediu-se a si mesma.** O `filter_sharpen` clampa a
//! entrada pelo próprio teto ANTES de qualquer aritmética, então toda leitura
//! acima dele via o CLAMP e não a lei — a "saturação em 4,0" que o doc afirmava
//! era o teto a devolver o mesmo número para qualquer entrada maior. Aqui tudo
//! passa pela [`sharpen_total_for_measurement`].
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --test measure_sharpen_intensify --release -- --ignored --nocapture`

use ph2d_mesh::{Mesh, shapes};
use ph2d_sculpt3d::{Brush, SculptStroke, Verb, sharpen_total_for_measurement};

fn norm(d: [f32; 3]) -> f32 {
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// A malha do PRODUTO com uma crista — valência 4, o regime da referência.
fn ridged() -> Mesh {
    let mut m = shapes::sculpt_sphere(1.0);
    for p in m.positions_mut() {
        let r = norm(*p);
        if r <= f32::EPSILON {
            continue;
        }
        let t = p[1] / r;
        let bump = (-(t * t) / (2.0 * 0.15 * 0.15)).exp() * 0.12;
        let s = (r + bump) / r;
        *p = [p[0] * s, p[1] * s, p[2] * s];
    }
    m
}

fn brush() -> Brush {
    Brush {
        verb: Verb::Smooth,
        ..Brush::default()
    }
}

/// **O DEGRAU** — o maior salto radial entre vizinhos. Afiar é aumentá-lo.
fn max_step(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let mut worst = 0.0f32;
    for v in 0..pos.len() {
        let r = norm(pos[v]);
        for &nb in adj.vert_verts.neighbours(v) {
            worst = worst.max((norm(pos[nb as usize]) - r).abs());
        }
    }
    worst
}

fn run(total: f32) -> (Mesh, f32) {
    let mut m = ridged();
    let pre = m.positions().to_vec();
    let mut s = SculptStroke::default();
    s.filter_begin(&m);
    sharpen_total_for_measurement(&mut s, &mut m, &brush(), total);
    let exc = pre
        .iter()
        .zip(m.positions())
        .map(|(a, c)| norm([c[0] - a[0], c[1] - a[1], c[2] - a[2]]))
        .fold(0.0f32, f32::max);
    (m, exc)
}

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_the_law_without_its_ceiling_and_what_a_step_costs() {
    let base = ridged();
    let step0 = max_step(&base);
    println!(
        "\n=== A MALHA DO PRODUTO — {} vértices | degrau em repouso {step0:.6} ===",
        base.vert_count()
    );

    println!("\n=== A LEI SEM O CLAMP — a 'saturação em 4,0' era o próprio teto ===");
    println!("\n  força | fatias | excursão | degrau    | Δdegrau");
    for f in [1.0f32, 2.0, 4.0, 6.0, 8.0, 16.0, 32.0, 64.0] {
        let (m, exc) = run(f);
        let st = max_step(&m);
        println!(
            "  {f:>5.1} | {:>6} | {exc:>8.5} | {st:>9.6} | {:>7.3}×",
            (f / 0.5).ceil().max(1.0) as u32,
            st / step0
        );
    }

    println!("\n=== O CUSTO — o candidato honesto a RECURSO do teto ===");
    println!("  (uma fatia percorre a malha INTEIRA: pré-passe + gather)");
    println!("\n  fatias | tempo total | por fatia");
    for f in [0.5f32, 2.0, 4.0, 8.0, 16.0] {
        let n = (f / 0.5).ceil().max(1.0) as u32;
        let mut m = ridged();
        let mut s = SculptStroke::default();
        s.filter_begin(&m);
        // ⚠️ **Uma corrida de aquecimento antes do relógio** — a primeira toca
        // páginas recém-alocadas, e é o custo da ALOCAÇÃO que ela mede.
        sharpen_total_for_measurement(&mut s, &mut m, &brush(), f);
        let t0 = std::time::Instant::now();
        sharpen_total_for_measurement(&mut s, &mut m, &brush(), f);
        let dt = t0.elapsed().as_secs_f64() * 1000.0;
        println!("  {n:>6} | {dt:>8.2} ms | {:>6.3} ms", dt / f64::from(n));
    }
}

/// **A HIPOTESE QUE RECONCILIA O REPORT** — a lei mede a variacao entre
/// vertices ADJACENTES, entao ela afia DETALHE FINO e nao feicao grande.
#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_whether_it_sharpens_fine_detail() {
    println!("\n=== DETALHE FINO vs FEICAO GRANDE — a mesma lei, duas malhas ===");
    println!(
        "  A lei le^ o laplaciano do ANEL IMEDIATO. Numa malha densa uma crista\n           larga e' quase PLANA localmente (o laplaciano dela ~ 0), entao nao ha'\n           nada que ela reconheca como detalhe."
    );
    for (label, build) in [
        (
            "CRISTA LARGA (98306 v, feicao de baixa frequencia)",
            (|| ridged()) as fn() -> Mesh,
        ),
        (
            "RUGA FINA (esfera UV 48x64 + ruido por-vertice)",
            (|| shapes::uv_sphere_noisy(48, 64, 1.0, 0.02)) as fn() -> Mesh,
        ),
    ] {
        let base = build();
        let step0 = max_step(&base);
        println!("\n  {label}");
        println!("    forca | excursao | degrau    | Δdegrau");
        for f in [1.0f32, 2.0, 4.0, 8.0] {
            let mut m = build();
            let pre = m.positions().to_vec();
            let mut s = SculptStroke::default();
            s.filter_begin(&m);
            sharpen_total_for_measurement(&mut s, &mut m, &brush(), f);
            let exc = pre
                .iter()
                .zip(m.positions())
                .map(|(a, c)| norm([c[0] - a[0], c[1] - a[1], c[2] - a[2]]))
                .fold(0.0f32, f32::max);
            let st = max_step(&m);
            println!(
                "    {f:>5.1} | {exc:>8.5} | {st:>9.6} | {:>7.3}×",
                st / step0
            );
        }
    }
}

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_what_the_intensify_term_buys() {
    println!("\n=== O TERMO QUE FALTA — o `sharpen_intensify_detail_strength` ===");
    println!(
        "  ⚠️ Com `INTENSIFY = 0` (o default do OPERADOR da referência) os dois\n  \
         termos restantes apontam AMBOS para a média, e a lei só ESTREITA a\n  \
         feição. O smoke reprovou isso (*\"parece alisar o mesh\"*).\n  \
         Esta tabela precisa de o const ser mudado à mão e a sonda re-rodada —\n  \
         ela mede o mundo que o binário atual tem."
    );
    let base = ridged();
    let step0 = max_step(&base);
    println!("\n  força | excursão | degrau    | Δdegrau  (INTENSIFY do binário)");
    for f in [1.0f32, 2.0, 4.0, 8.0] {
        let (m, exc) = run(f);
        println!(
            "  {f:>5.1} | {exc:>8.5} | {:>9.6} | {:>7.3}×",
            max_step(&m),
            max_step(&m) / step0
        );
    }
}
