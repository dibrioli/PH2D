//! **A sonda dos verbos que PUXAM** — o perfil do Grab e a fidelidade do Snake
//! Hook à taxa de amostragem.
//!
//! Ela responde duas perguntas que nenhum gate deve *escolher*:
//!
//! 1. **Qual é o expoente do falloff do Grab?** O `Drag.js`/`Move.js` aplicam o
//!    falloff **uma** vez (`vAr[ind] += dir * fallOff`), e o nosso aplicador
//!    multiplica `(alvo − base)` pelo `accum`. Se o alvo já traz o peso, o peso
//!    entra **duas** vezes e o pincel fica pontudo.
//! 2. **Quanto o Snake Hook depende da taxa de polling?** Ele não é o envelope —
//!    é uma INTEGRAL DE LINHA, e o que ela promete é convergir com o
//!    espaçamento, não ser exata em qualquer um. O número da barra do gate sai
//!    daqui.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_pull_profile -- --ignored --nocapture
//! ```

use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

fn sphere() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)
}

/// A curva de falloff que o pincel default usa, avaliada em `t = d/r`.
fn fall(t: f32) -> f32 {
    ph2d_sculpt3d::Falloff::default().weight(t)
}

/// Arrasta um Snake Hook por um caminho RETO de `len` unidades de mundo,
/// entregue em `events` eventos de ponteiro — o laço do shell, sem câmera (a
/// unidade do [`ph2d_sculpt3d::walk`] é a do chamador, e num arrasto a
/// profundidade constante a régua da tela e a do mundo diferem por uma escala).
///
/// Devolve o quanto o vértice mais deslocado andou.
fn hook_path(radius: f32, strength: f32, len: f32, events: usize) -> f32 {
    let mut mesh = sphere();
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let at = [0.0, 0.0, 1.0];
    let brush = Brush {
        verb: Verb::SnakeHook,
        radius,
        strength,
        ..Brush::default()
    };
    let spacing = ph2d_sculpt3d::min_spacing(radius);

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let mut anchor = [0.0f32, 0.0];
    for e in 1..=events {
        let to = [len * e as f32 / events as f32, 0.0];
        let Some(steps) = ph2d_sculpt3d::walk(anchor, to, spacing) else {
            continue;
        };
        let mut prev = anchor;
        for step in steps {
            let c1 = [at[0] + step[0], at[1], at[2]];
            let d = [step[0] - prev[0], step[1] - prev[1], 0.0];
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::hooking(c1, radius, [0.0, 0.0, -1.0], d),
                Symmetry::default(),
            );
            prev = step;
        }
        anchor = to;
    }
    mesh.positions()
        .iter()
        .zip(&before)
        .map(|(p, b)| {
            ((p[0] - b[0]).powi(2) + (p[1] - b[1]).powi(2) + (p[2] - b[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max)
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_the_hook_against_the_polling_rate() {
    println!("\n  SNAKE HOOK — o mesmo caminho, entregue em taxas diferentes");
    println!("  (raio 0,4, força 1,0; o walk fixa o passo em 0,15·raio = 0,06)");
    println!("  eventos    passos    ponta viajou    razão vs 1 evento");
    let (radius, len) = (0.4f32, 1.2f32);
    let mut first = 0.0f32;
    for (i, events) in [1usize, 2, 4, 8, 32, 128].into_iter().enumerate() {
        let tip = hook_path(radius, 1.0, len, events);
        if i == 0 {
            first = tip;
        }
        // Quantos dabs o caminho inteiro produz nesta taxa.
        let mut steps = 0usize;
        let mut anchor = [0.0f32, 0.0];
        let spacing = ph2d_sculpt3d::min_spacing(radius);
        for e in 1..=events {
            let to = [len * e as f32 / events as f32, 0.0];
            if let Some(w) = ph2d_sculpt3d::walk(anchor, to, spacing) {
                steps += w.len() as usize;
                anchor = to;
            }
        }
        println!(
            "  {events:>7}   {steps:>7}    {tip:>12.5}    {:>16.4}",
            tip / first
        );
    }
    println!();
}

/// A pergunta do artista: *arrastar para fora e voltar deixa espinho?* É a
/// diferença VISÍVEL entre as duas leis, e o número dela desenha o gate.
#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_out_and_back_under_both_laws() {
    println!("\n  IDA E VOLTA — o mesmo gesto sob as duas leis");
    for verb in [Verb::Move, Verb::SnakeHook] {
        let mut mesh = sphere();
        let before: Vec<[f32; 3]> = mesh.positions().to_vec();
        let at = [0.0, 0.0, 1.0];
        let (radius, strength) = (0.4f32, 1.0f32);
        let brush = Brush {
            verb,
            radius,
            strength,
            ..Brush::default()
        };
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        // Sobe até y = +0,6 e volta a zero, em passos de 0,05.
        let mut prev = 0.0f32;
        let ups = (0..=12).map(|k| k as f32 * 0.05);
        let downs = (0..12).rev().map(|k| k as f32 * 0.05);
        for y in ups.chain(downs) {
            let dab = match verb {
                Verb::Move => Dab::pulling(at, radius, [0.0, 0.0, -1.0], [0.0, y, 0.0]),
                _ => Dab::hooking(
                    [at[0], at[1] + y, at[2]],
                    radius,
                    [0.0, 0.0, -1.0],
                    [0.0, y - prev, 0.0],
                ),
            };
            stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
            prev = y;
        }
        let worst = mesh
            .positions()
            .iter()
            .zip(&before)
            .map(|(p, b)| {
                ((p[0] - b[0]).powi(2) + (p[1] - b[1]).powi(2) + (p[2] - b[2]).powi(2)).sqrt()
            })
            .fold(0.0f32, f32::max);
        println!("  {:>12}   sobra depois da volta: {worst:.5}", verb.label());
    }
    println!();
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_the_grab_falloff_exponent() {
    let mut mesh = sphere();
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let at = [0.0, 0.0, 1.0];
    let radius = 0.5f32;
    let pull = [0.0, 0.4, 0.0];

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &Brush {
            verb: Verb::Move,
            radius,
            strength: 1.0,
            ..Brush::default()
        },
        &Dab::pulling(at, radius, [0.0, 0.0, -1.0], pull),
        Symmetry::default(),
    );

    println!("\n  GRAB — deslocamento medido contra as duas leis candidatas");
    println!("  (raio {radius}, força 1,0, puxão {})", pull[1]);
    println!("   d/r    medido     pull·fall   pull·fall²");
    for band in 1..=9 {
        let t = band as f32 / 10.0;
        // O vértice cuja posição CONGELADA está mais perto da banda.
        let mut best = (f32::MAX, 0usize);
        for (i, p) in before.iter().enumerate() {
            let d = ((p[0] - at[0]).powi(2) + (p[1] - at[1]).powi(2) + (p[2] - at[2]).powi(2))
                .sqrt()
                / radius;
            let err = (d - t).abs();
            if err < best.0 {
                best = (err, i);
            }
        }
        let i = best.1;
        let moved = mesh.positions()[i][1] - before[i][1];
        let f = fall(t);
        println!(
            "  {t:>4.1}   {moved:>8.5}   {:>9.5}   {:>10.5}",
            pull[1] * f,
            pull[1] * f * f
        );
    }
    println!();
}
