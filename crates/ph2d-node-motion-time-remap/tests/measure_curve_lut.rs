//! **SONDA — de que TAMANHO tem de ser a tabela que leva uma curva ao relógio?**
//!
//! O `TimeMap` do substrato é `Copy` e entra na **CHAVE DO MEMO** (`push_scope` mistura
//! os bits dele), e o `ph2d_curve::Curve` é um `Vec<Point>` que **aloca**. Uma curva no
//! relógio precisa então de um carregador de tamanho FIXO — e o precedente do `LutSpec`
//! diz qual: *o substrato fica agnóstico de curva*, então o que viaja é uma **tabela de
//! `f32`** que a crate do NÓ preenche.
//!
//! A pergunta que sobra é o **N**, e ela não se escolhe: um erro de tabela é um erro de
//! TEMPO, e o que decide é ele medido em **QUADROS** — abaixo de meio quadro o artista
//! não pode vê-lo, porque não existe amostra entre dois quadros.
//!
//! ⚠️ E há duas leituras, não uma: o **VALOR** (o instante que a sub-árvore recebe) e a
//! **VELOCIDADE** (a derivada — uma tabela interpolada linearmente tem velocidade
//! constante por segmento, e é ela que lê como *"o movimento está aos degraus"*).
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-motion-time-remap --test measure_curve_lut -- --ignored --nocapture`.

use ph2d_curve::{Curve, Interp, Point};

/// A curva de ease que um artista de facto desenha: dois pontos, tangentes chatas.
fn ease() -> Curve {
    Curve {
        points: vec![
            Point {
                x: 0.0,
                y: 0.0,
                interp: Interp::Smooth,
            },
            Point {
                x: 1.0,
                y: 1.0,
                interp: Interp::Smooth,
            },
        ],
    }
}

/// Uma curva com uma pausa no meio (o *hold* que um remap de tempo quer): três pontos.
fn hold_middle() -> Curve {
    Curve {
        points: vec![
            Point {
                x: 0.0,
                y: 0.0,
                interp: Interp::Smooth,
            },
            Point {
                x: 0.45,
                y: 0.5,
                interp: Interp::Hold,
            },
            Point {
                x: 0.55,
                y: 0.5,
                interp: Interp::Smooth,
            },
        ],
    }
}

/// Amostra a curva em `n` pontos e lê a tabela com interpolação LINEAR.
fn lut_eval(c: &Curve, n: usize, t: f32) -> f32 {
    let last = (n - 1) as f32;
    let u = (t.clamp(0.0, 1.0) * last).min(last);
    let i = u.floor() as usize;
    let f = u - i as f32;
    let a = c.eval(i as f32 / last);
    let b = c.eval(((i + 1).min(n - 1)) as f32 / last);
    a + (b - a) * f
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn how_big_the_clock_table_has_to_be() {
    // A janela do remap e a taxa de quadros que decidem a leitura.
    const WINDOW_S: f32 = 2.0;
    const FPS: f32 = 60.0;
    for (tag, c) in [("ease (2 pontos)", ease()), ("hold no meio", hold_middle())] {
        eprintln!("\n[lut] {tag} — janela {WINDOW_S} s a {FPS} fps");
        eprintln!("     N | pior |dt| (s) | em QUADROS | pior |dv| (frac)");
        for n in [16usize, 32, 64, 128, 256] {
            let mut worst_v = 0.0f32;
            let mut worst_d = 0.0f32;
            let steps = 4096;
            let mut prev = (0.0f32, 0.0f32);
            for k in 0..=steps {
                let t = k as f32 / steps as f32;
                let exact = c.eval(t);
                let table = lut_eval(&c, n, t);
                worst_v = worst_v.max((exact - table).abs());
                if k > 0 {
                    let dt = 1.0 / steps as f32;
                    let de = (exact - prev.0) / dt;
                    let dl = (table - prev.1) / dt;
                    worst_d = worst_d.max((de - dl).abs());
                }
                prev = (exact, table);
            }
            let secs = worst_v * WINDOW_S;
            eprintln!(
                "  {n:>4} | {secs:>12.6} | {:>10.4} | {worst_d:>16.4}",
                secs * FPS
            );
        }
    }
    eprintln!("\n  => o N certo e' o menor cujo erro em QUADROS fica bem abaixo de 0,5.");
}
