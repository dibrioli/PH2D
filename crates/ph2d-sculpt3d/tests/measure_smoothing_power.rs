//! **QUANTO CADA MODO DO SMOOTH DE FACTO ALISA** — a sonda que responde ao
//! report *"o `l-mode` é tão discreto que é quase imperceptível"*.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_smoothing_power \
//!   -- --ignored --nocapture --test-threads=1
//! ```
//!
//! ⚠️ **A fixture tem de conter o fenómeno, e uma esfera LISA não contém.** O
//! que um artista alisa é uma superfície com RUGAS; sobre uma esfera perfeita
//! não há alta frequência para atenuar e todo filtro passa-baixa mede zero por
//! construção — mediríamos o encolhimento (que é o que a sonda irmã
//! `measure_smooth_shrinkage` já mede) e chamaríamos isso de *"não alisa"*.
//! Daqui sai a [`ph2d_mesh::shapes::uv_sphere_noisy`].
//!
//! ⚠️ **O ORÁCULO é a RUGOSIDADE, e ela é a magnitude do laplaciano** —
//! `|p − média do anel|`, exactamente a grandeza que todo operador de suavização
//! ataca. Medi-la pelo raio médio responderia *"encolheu?"*, que é outra
//! pergunta; medi-la pelo deslocamento responderia *"mexeu?"*, que um pincel que
//! empurra tudo para o mesmo lado também satisfaz.
//!
//! ⚠️ **E a sonda mede pela porta do PRODUTO** (`SculptStroke::dab`), nunca por
//! um laço próprio sobre o `ring_average`: um laço próprio mediria a FÓRMULA e
//! ficaria cego ao que o traço faz com ela.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, Falloff, RefMode, SculptStroke, Symmetry, Verb};

fn noisy() -> Mesh {
    ph2d_mesh::shapes::uv_sphere_noisy(48, 64, 1.0, 0.03)
}

/// **A RUGOSIDADE** — a magnitude média do laplaciano uniforme.
///
/// ⚠️ **Uniforme mesmo no `l-mode`, de propósito:** ela é a RÉGUA, e uma régua
/// que trocasse de operador junto com o pincel mediria cada coluna com uma
/// unidade diferente. O que se compara é quanto de ruga sobra, não como cada
/// modo a define.
fn roughness(mesh: &Mesh) -> f64 {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let mut sum = 0.0f64;
    for v in 0..pos.len() {
        let a = ph2d_mesh::ring_average(adj, v as u32, pos[v], |nb| pos[nb as usize]);
        let d = [pos[v][0] - a[0], pos[v][1] - a[1], pos[v][2] - a[2]];
        sum += f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))).sqrt();
    }
    sum / pos.len() as f64
}

/// Quanto a malha ANDOU no total — o *"aconteceu alguma coisa?"* que o olho lê.
fn travel(before: &[[f32; 3]], mesh: &Mesh) -> f64 {
    let pos = mesh.positions();
    let mut sum = 0.0f64;
    for i in 0..pos.len() {
        let d = [
            pos[i][0] - before[i][0],
            pos[i][1] - before[i][1],
            pos[i][2] - before[i][2],
        ];
        sum += f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))).sqrt();
    }
    sum / pos.len() as f64
}

fn mean_radius(mesh: &Mesh) -> f64 {
    let p = mesh.positions();
    p.iter()
        .map(|v| f64::from(v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2]))).sqrt())
        .sum::<f64>()
        / p.len() as f64
}

fn brush(mode: RefMode) -> Brush {
    Brush {
        verb: Verb::Smooth,
        mode,
        radius: 4.0,
        strength: 1.0,
        falloff: Falloff::Constant,
        ..Brush::default()
    }
}

fn whole_dab() -> Dab {
    Dab::at([0.0, 0.0, 0.0], 4.0, [0.0, 0.0, -1.0])
}

/// **A MEDIÇÃO.** A mesma ruga, os três modos, força cheia.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn how_much_each_mode_actually_smooths() {
    println!("\n  RUGOSIDADE que SOBRA (|p - media do anel|), esfera com ruga 0,03\n");
    println!("  dabs        S            B            L        L/S");
    println!("  ----   ----------   ----------   ----------   -----");
    let base = noisy();
    let r0 = roughness(&base);
    println!("  {:>4}   {r0:>10.6}   {r0:>10.6}   {r0:>10.6}   1.000", 0);

    let mut m: Vec<Mesh> = vec![base.clone(), base.clone(), base.clone()];
    let modes = [RefMode::S, RefMode::B, RefMode::L];
    for dabs in 1..=32 {
        for (i, &mode) in modes.iter().enumerate() {
            let mut s = SculptStroke::default();
            s.begin(&m[i]);
            s.dab(&mut m[i], &brush(mode), &whole_dab(), Symmetry::default());
        }
        if dabs <= 4 || dabs % 8 == 0 {
            let (rs, rb, rl) = (roughness(&m[0]), roughness(&m[1]), roughness(&m[2]));
            println!(
                "  {dabs:>4}   {rs:>10.6}   {rb:>10.6}   {rl:>10.6}   {:>5.3}",
                rl / rs
            );
        }
    }

    println!("\n  QUANTO A MALHA ANDOU (media por vertice), o raio e a DERIVA\n");
    println!("  modo     andou     raio medio   deriva tangencial");
    println!("  ----   ---------   ----------   -----------------");
    let before = base.positions().to_vec();
    let nrm = base.normals().to_vec();
    for (i, name) in ["S", "B", "L"].iter().enumerate() {
        println!(
            "  {name:>4}   {:>9.6}   {:>10.6}   {:>17.8}",
            travel(&before, &m[i]),
            mean_radius(&m[i]),
            drift(&before, &nrm, m[i].positions())
        );
    }
}

/// Quanto os vértices escorregaram **ao longo** da superfície — a grandeza que
/// o operador por cotangentes existe para reduzir, e a que o gate
/// `cotangent_operator.rs` compara entre os chips.
///
/// ⚠️ **A aritmética é a MESMA do gate** (`f32`, soma simples, sem `mul_add`) e
/// não é cerimónia: a primeira versão somava em `f64` com `mul_add` e imprimia
/// `0,01227699` onde o gate mede `0,01227704` — **dois números para a mesma
/// medição**, e o doc de um deles passaria a citar o do outro.
#[allow(clippy::suboptimal_flops)] // espelha a aritmética do gate, de propósito.
fn drift(before: &[[f32; 3]], nrm: &[[f32; 3]], after: &[[f32; 3]]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..before.len() {
        let d = [
            after[i][0] - before[i][0],
            after[i][1] - before[i][1],
            after[i][2] - before[i][2],
        ];
        let n = nrm[i];
        let along = d[0] * n[0] + d[1] * n[1] + d[2] * n[2];
        let t = [
            d[0] - along * n[0],
            d[1] - along * n[1],
            d[2] - along * n[2],
        ];
        sum += (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
    }
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de vértices.
    let n = before.len() as f32;
    sum / n
}

/// **UM DAB SÓ** — o que o artista sente na primeira pincelada, que é onde o
/// veredito *"quase imperceptível"* nasce.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn what_a_single_dab_does() {
    println!("\n  UM DAB, forca cheia\n");
    println!("  modo   rugosidade   queda    andou");
    println!("  ----   ----------   ------   ---------");
    let base = noisy();
    let r0 = roughness(&base);
    let before = base.positions().to_vec();
    for (name, mode) in [("S", RefMode::S), ("B", RefMode::B), ("L", RefMode::L)] {
        let mut m = base.clone();
        let mut s = SculptStroke::default();
        s.begin(&m);
        s.dab(&mut m, &brush(mode), &whole_dab(), Symmetry::default());
        let r = roughness(&m);
        println!(
            "  {name:>4}   {r:>10.6}   {:>5.1}%   {:>9.6}",
            (r0 - r) / r0 * 100.0,
            travel(&before, &m)
        );
    }
    println!("\n  (rugosidade inicial {r0:.6})");
}

/// **A DERIVA NA FIXTURE DO GATE** — a tabela que o doc de
/// `cotangent_operator.rs::the_literature_chip_slides_the_surface_far_less_than_the_blender_one`
/// cita, re-medida sempre que o `λ` se mexe.
///
/// ⚠️ **A fixture é a do GATE e não a desta sonda** (esfera LISA, 4 dabs), de
/// propósito: um doc que cita números medidos noutra malha é um doc que envelhece
/// sem ninguém notar.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn the_drift_table_the_gate_cites() {
    let radius = 4.0;
    let base = ph2d_mesh::shapes::uv_sphere(24, 32, 1.0);
    let before = base.positions().to_vec();
    let nrm = base.normals().to_vec();
    println!("\n  DERIVA TANGENCIAL, uv_sphere 24x32 LISA, 4 dabs, forca cheia\n");
    println!("  verbo       S            B            L         B/L");
    println!("  -------   ----------   ----------   ----------   -----");
    for verb in [Verb::Smooth, Verb::Sharpen] {
        let mut row = [0.0f32; 3];
        for (i, mode) in [RefMode::S, RefMode::B, RefMode::L].into_iter().enumerate() {
            let b = Brush {
                verb,
                mode,
                radius,
                strength: 1.0,
                falloff: Falloff::Constant,
                ..Brush::default()
            };
            let mut m = base.clone();
            for _ in 0..4 {
                let mut s = SculptStroke::default();
                s.begin(&m);
                s.dab(&mut m, &b, &whole_dab(), Symmetry::default());
            }
            row[i] = drift(&before, &nrm, m.positions());
        }
        println!(
            "  {:<7}   {:>10.8}   {:>10.8}   {:>10.8}   {:>5.1}x",
            format!("{verb:?}"),
            row[0],
            row[1],
            row[2],
            row[1] / row[2]
        );
    }
}

/// **QUANTO DO CURSO DO SLIDER É ÚTIL** — a força é o que o artista tem na mão,
/// e um modo que precisa dela toda para fazer o que o vizinho faz a um terço é
/// um modo com o curso desperdiçado.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn how_much_strength_the_literature_mode_needs() {
    println!("\n  QUEDA DE RUGOSIDADE em UM dab, por forca\n");
    println!("  forca        S        B        L");
    println!("  -----   ------   ------   ------");
    let base = noisy();
    let r0 = roughness(&base);
    for strength in [0.25f32, 0.5, 0.75, 1.0] {
        let mut row = [0.0f64; 3];
        for (i, mode) in [RefMode::S, RefMode::B, RefMode::L].into_iter().enumerate() {
            let b = Brush {
                strength,
                ..brush(mode)
            };
            let mut m = base.clone();
            let mut s = SculptStroke::default();
            s.begin(&m);
            s.dab(&mut m, &b, &whole_dab(), Symmetry::default());
            row[i] = (r0 - roughness(&m)) / r0 * 100.0;
        }
        println!(
            "  {strength:>5.2}   {:>5.1}%   {:>5.1}%   {:>5.1}%",
            row[0], row[1], row[2]
        );
    }
}
