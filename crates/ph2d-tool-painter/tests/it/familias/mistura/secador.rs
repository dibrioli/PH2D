//! **O secador seca a tinta JÁ NIVELADA** (`Knob::DryerLevel`, J6 lacuna 5 — `docs/Painter/Pinceis/exames/mistura.md`
//! «Motor J6»): a lei presa ao motor numa cena mínima, com o neutro ao bit.

use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 260;
const H: usize = 140;
const Y: usize = 70;
const X: usize = 130;

/// A faixa MOLHADA fresca com os sulcos das cerdas (y 58..82: 3 linhas carregadas — filme 0,5, pigmento 200 — e 3
/// rasas — 0,1 · 40 —, a concentração igual: a tinta fresca do Wet Round) e a FRANJA rasa de água por baixo (y 83..88,
/// filme 0,02, pigmento 1).
fn faixa_fresca() -> Engine {
    let mut e = Engine::new(W, H);
    let g = e.active_grid_mut();
    for y in Y - 12..=Y + 18 {
        for x in 20..=240 {
            let i = x + 1 + (y + 1) * g.s;
            let (f, m) = if y > Y + 12 {
                (0.02, 1.0)
            } else if (y - (Y - 12)) % 6 < 3 {
                (0.5, 200.0)
            } else {
                (0.1, 40.0)
            };
            g.film[i] = f;
            g.susp[i] = m;
            g.susp_rgb[i] = [191.0, 0.0, 18.0];
        }
    }
    e
}

/// O secador (Dry) em vertical por x = 130, raio 10, com o nível dado.
fn seca(nivel: Option<f64>) -> Engine {
    let mut e = faixa_fresca();
    if let Some(v) = nivel {
        e.tuning.set(Knob::DryerLevel, v);
    }
    let mut prev: Option<[f64; 2]> = None;
    let mut y = 20.0;
    while y <= 120.0 {
        e.dispatch_pressure_dab_tool(Tool::Dry, X as f64, y, 5.0, 10.0, prev);
        prev = Some([X as f64, y]);
        y += 2.0;
    }
    e
}

/// O pigmento (suspenso + assentado) no eixo do cruzamento (x 127..133), por linha.
fn eixo(e: &Engine, y: usize) -> f64 {
    let g = e.active_grid();
    (X - 3..=X + 3)
        .map(|x| {
            let i = x + 1 + (y + 1) * g.s;
            f64::from(g.susp[i]) + f64::from(g.sett[i])
        })
        .sum::<f64>()
        / 7.0
}

/// ⭐ Com `dryerLevel` 2 o cruzamento seca LISO (o desvio entre as linhas da faixa no eixo cai abaixo de 0,1 da média;
/// sem ele os sulcos ficam, 0,67), a massa da faixa fica (± 10 %) e a franja rasa NÃO enche (≤ 3 × o que tinha: sem o
/// piso de espessura recebia a média da faixa, ~100 ×). Sem o knob, o secador de antes ao bit.
#[test]
fn o_secador_seca_a_tinta_ja_nivelada() {
    let mede = |e: &Engine| {
        let linhas: Vec<f64> = (Y - 10..=Y + 10).map(|y| eixo(e, y)).collect();
        let med = linhas.iter().sum::<f64>() / linhas.len() as f64;
        let desvio = (linhas.iter().map(|v| (v - med).powi(2)).sum::<f64>() / linhas.len() as f64)
            .sqrt()
            / med.max(1e-9);
        let franja = (Y + 14..=Y + 18).map(|y| eixo(e, y)).sum::<f64>() / 5.0;
        (med, desvio, franja)
    };
    let (m0, d0, f0) = mede(&seca(None));
    let (m1, d1, f1) = mede(&seca(Some(2.0)));
    println!(
        "[mistura] secador nível: sem {d0:.3} (média {m0:.1}, franja {f0:.2}) · com 2 {d1:.3} (média {m1:.1}, franja {f1:.2})"
    );
    let mut falhas = Vec::new();
    if d0 < 0.4 {
        falhas.push(format!(
            "a cena não tem sulcos: desvio sem o knob {d0:.3} < 0,4"
        ));
    }
    if d1 > 0.1 {
        falhas.push(format!(
            "o cruzamento não nivelou: desvio {d1:.3} > 0,1 (sem o knob {d0:.3})"
        ));
    }
    if (m1 / m0 - 1.0).abs() > 0.1 {
        falhas.push(format!("a massa da faixa mudou: {m1:.1} contra {m0:.1}"));
    }
    if f1 > 3.0 * f0.max(1.0) {
        falhas.push(format!("a franja rasa encheu: {f1:.2} contra {f0:.2}"));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
    // Sem o knob = o secador de antes, ao bit.
    let (a, b) = (seca(None), seca(Some(0.0)));
    let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&a.active_grid().susp), bits(&b.active_grid().susp));
    assert_eq!(bits(&a.active_grid().film), bits(&b.active_grid().film));
}
