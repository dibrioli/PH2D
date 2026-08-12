//! **A CERCA DO INFLATE, MEDIDA** — quanto a normal VIVA gira debaixo de um
//! traço parado, que é a premissa que a nossa divergência declara.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_inflate_normal_drift \
//!   -- --ignored --nocapture
//! ```
//!
//! # A cerca, e por que ela é medida em vez de removida
//!
//! O `Inflate.js:64-66` lê `nAr` — as normais **VIVAS**, que o `updateGeometry`
//! recomputa a cada dab — e o `inflate.cc` do Blender também. Nós lemos a
//! congelada no pen-down, e o `stroke_target.rs` declara o motivo:
//!
//! > *a normal viva sobe junto com a tinta, e um traço parado passaria a inflar
//! > numa direção que gira sozinha.*
//!
//! ⚠️ **É uma cerca de Chesterton COM motivo escrito, e o motivo é uma
//! AFIRMAÇÃO SOBRE UM NÚMERO** — exatamente a forma que o §0 do `CLAUDE.md`
//! manda medir antes de decidir. As duas referências discordam de nós; se o giro
//! for pequeno, a cerca custa paridade e não compra nada. Se for grande, ela
//! está certa e o preço fica NOMEADO em vez de suposto.
//!
//! ⚠️ **Esta sonda NÃO muda o kernel.** Ela mede a premissa — quanto a normal de
//! um vértice gira ao longo de um traço parado — sem construir o ramo que a
//! usaria. Construir primeiro e medir depois é como se decide a favor do que se
//! construiu.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(48, 96, 1.0)
}

/// O dab pousa no equador, longe dos polos da esfera UV (onde a topologia
/// degenera e o número falaria sobre a malha).
fn dab_at(radius: f32) -> Dab {
    let c = [0.7f32.cos(), 0.0, 0.7f32.sin()];
    let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    let eye = [-c[0] / len, -c[1] / len, -c[2] / len];
    Dab::at(c, radius, eye)
}

fn angle_deg(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
    dot.acos().to_degrees()
}

/// **QUANTO A NORMAL GIRA NUM TRAÇO PARADO** — a tabela que decide a cerca.
///
/// Ela varre a FORÇA porque a premissa da cerca é sobre a tinta acumulada, e a
/// força é o que a acumula: um traço parado de força baixa e um de força cheia
/// são duas perguntas diferentes com o mesmo gesto.
#[test]
#[ignore = "sonda: imprime a tabela, não afirma nada"]
fn how_far_does_the_live_normal_turn_under_a_parked_stroke() {
    const R: f32 = 0.45;
    println!("\n== A NORMAL VIVA sob um traço PARADO (Inflate, {R} de raio) ==");
    println!(
        "{:>6} {:>6} {:>12} {:>12} {:>12} {:>6}",
        "força", "dabs", "giro médio°", "giro máx°", "no MIOLO°", "n"
    );

    for &strength in &[0.3f32, 0.6, 1.0] {
        for &dabs in &[1usize, 4, 16, 64] {
            let mut mesh = sphere();
            let d = dab_at(R);
            // As normais do pen-down, que é o que o nosso kernel congela.
            let frozen: Vec<[f32; 3]> = mesh.normals().to_vec();
            let frozen_pos: Vec<[f32; 3]> = mesh.positions().to_vec();

            let b = Brush {
                verb: Verb::Inflate,
                radius: R,
                strength,
                falloff: Falloff::Plateau,
                ..Brush::default()
            };
            let mut s = SculptStroke::default();
            s.begin(&mesh);
            for _ in 0..dabs {
                s.dab(&mut mesh, &b, &d, Symmetry::default());
            }

            // O giro por vértice DA PEGADA, e o do miolo isolado: a cerca fala
            // da direção em que a tinta é empurrada, e no miolo é onde ela é
            // empurrada mais.
            // ⚠️ **A distância é medida na posição CONGELADA, e a primeira
            // versão desta sonda a media na VIVA** — o Inflate empurra os
            // vértices para FORA, então o conjunto `dist < R/4` esvaziava e o
            // `max` sobre o vazio devolvia o inicializador: a coluna do miolo
            // imprimia `0,000°`, que se lê como *"o miolo não gira"* e
            // significava *"não sobrou miolo pela minha régua"*. A pegada é
            // ancorada no pen-down; a régua tem de ser também.
            let (mut sum, mut n, mut worst, mut core, mut core_n) =
                (0.0f32, 0usize, 0.0f32, 0.0f32, 0usize);
            for (v, live) in mesh.normals().iter().enumerate() {
                let p = frozen_pos[v];
                let r = [p[0] - d.center[0], p[1] - d.center[1], p[2] - d.center[2]];
                let dist = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
                if dist >= R {
                    continue;
                }
                let a = angle_deg(frozen[v], *live);
                sum += a;
                n += 1;
                worst = worst.max(a);
                if dist < R * 0.25 {
                    core = core.max(a);
                    core_n += 1;
                }
            }
            let mean = if n == 0 { 0.0 } else { sum / n as f32 };
            // ⚠️ A CONTAGEM ao lado do miolo: sem ela um conjunto vazio
            // imprimiria `0,000°` e leria como o oposto do que significa.
            println!(
                "{strength:>6.1} {dabs:>6} {mean:>11.3} {worst:>11.3} {core:>11.3} {core_n:>6}"
            );
        }
    }

    println!(
        "\n  ^ a cerca do `stroke_target.rs` afirma que a normal viva \"gira\n    \
         sozinha\" num traço parado. Estes são os graus."
    );
}
