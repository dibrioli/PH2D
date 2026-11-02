//! **A dinâmica contra o Trial 1.3.18.6** (`oracle_trial_dynamics_velocity.json`, `round2_size_velocity_rows`,
//! `docs/PixelArt/ferramentas/trial_dinamica.py` modo `r2rows`): cada arrasto recto gravado — os
//! eventos XTest com o relógio de cada um — refaz-se pelo NOSSO filtro, a NOSSA rampa e o NOSSO
//! traço, e a altura de cada coluna pintada compara-se com a que o Trial pintou.

use ph2d_pixel_core::brush::Brush;
use ph2d_pixel_core::dynamics::{Channel, Dynamics, Sense, Sensor, Velocity};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Paint, Sprite, Stroke, apply};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/oracle_trial_dynamics_velocity.json"
    ))
    .expect("json")
}

/// O canvas do `big_setup` (640 × 400 células) e o meio-alto da leitura (`row_sizes`, half = 14).
const W: u32 = 640;
const H: u32 = 400;
const HALF: i32 = 14;

/// As pegas de cada documento (o `threshold_fraction` da fixtura, lido na barra).
fn pegas(run: &str) -> (f64, f64) {
    match run {
        "r3lo50_T" => (0.49, 0.89),
        "r4hi29_U" | "r5z200_Z" => (0.085, 0.29),
        _ => (0.10, 0.89),
    }
}

/// px de ecrã por célula e a 1.ª coluna lida (`run_rows`: 100 % = 2 px; o r5 a 200 % = 4 px, e a
/// leitura dele começa na célula 10).
fn vista(run: &str) -> (f64, i32) {
    if run == "r5z200_Z" {
        (4.0, 10)
    } else {
        (2.0, 0)
    }
}

/// Refaz um arrasto: a altura de cada coluna lida (como o `row_sizes`) e a célula onde começa o
/// ÚLTIMO trecho — o Trial nunca o pinta (a ponta fica 2–24 px atrás do cursor; nós pintamos até ao
/// cursor, de propósito), então a comparação pára um raio antes dele.
fn refaz(row: &Value, d: Dynamics, (cell, off): (f64, i32), n: usize) -> (Vec<u32>, usize) {
    let j = row["row"].as_i64().expect("row") as i32;
    let log = row["log"].as_array().expect("log");
    let ev = |e: &Value| {
        let n = |k: usize| e[k].as_f64().expect("num");
        (n(0), n(1), n(2), n(3))
    };
    let (_, x0, y0, d0) = ev(&log[0]);
    let start = [x0 - d0, y0];
    let cel = |x: f64| 20 + ((x - start[0]) / cell).round() as i32;
    let mut s = Sprite::new(NewSprite {
        width: W,
        height: H,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    let key = (s.layers()[0].id, s.frames()[0].id);
    let preto = s.value_of_rgba([0, 0, 0, 255]);
    let mut v = Velocity::default();
    let paint = Paint {
        brush: Brush {
            size: d.dab(Sense::default()).size.expect("Size ← Velocity"),
            ..Brush::default()
        },
        ..Paint::default()
    };
    let mut st = Stroke::begin(&s, key.0, key.1, preto, paint, (20, j));
    let mut ultimo = 20;
    for e in log {
        let (t, x, y, _) = ev(e);
        ultimo = st.last().0;
        let vel = v.sample([x, y], (t * 1e9) as u64);
        st.set_dab(d.dab(Sense {
            pressure: 0.0,
            velocity: vel,
        }));
        st.to(&s, cel(x), j);
    }
    let op = st.op(&s).expect("op");
    apply(&mut s, op).expect("apply");
    let fim = (ultimo - off - 8).max(0) as usize;
    let alturas = (off..off + n as i32)
        .map(|x| {
            (j - HALF..=j + HALF)
                .filter(|&y| s.rgba_of_value(s.value_at(key, x, y))[3] != 0)
                .count() as u32
        })
        .collect();
    (alturas, fim.min(n))
}

fn dinamica(lo: f64, hi: f64) -> Dynamics {
    Dynamics {
        size: Channel {
            sensor: Sensor::Velocity,
            lo,
            hi,
            min: 1.0,
            max: 16.0,
        },
        ..Dynamics::default()
    }
}

#[test]
fn every_recorded_drag_paints_the_trials_column_heights() {
    let f = fx();
    let runs = f["round2_size_velocity_rows"]["runs"]
        .as_object()
        .expect("runs");
    let (mut soma, mut n, mut pior) = (0.0, 0usize, 0.0f64);
    let mut linhas = Vec::new();
    for (run, r) in runs {
        let (lo, hi) = pegas(run);
        for row in r["rows"].as_array().expect("rows") {
            let quer: Vec<u32> = row["read"]
                .as_array()
                .expect("read")
                .iter()
                .map(|v| v.as_u64().expect("h") as u32)
                .collect();
            let v = vista(run);
            let (tem, fim) = refaz(row, dinamica(lo, hi), v, quer.len());
            let cols: Vec<usize> = ((20 - v.1) as usize..fim)
                .filter(|&i| quer[i] > 0 || tem[i] > 0)
                .collect();
            let e: f64 = cols
                .iter()
                .map(|&i| f64::from(quer[i].abs_diff(tem[i])))
                .sum();
            let m = e / cols.len() as f64;
            pior = pior.max(m);
            linhas.push((
                m,
                format!(
                    "{run}/{}: {m:.3}\n quer {:?}\n tem  {:?}",
                    row["name"],
                    quer.iter().skip(20).step_by(40).collect::<Vec<_>>(),
                    tem.iter().skip(20).step_by(40).collect::<Vec<_>>()
                ),
            ));
            soma += e;
            n += cols.len();
        }
    }
    let m = soma / n as f64;
    linhas.sort_by(|a, b| b.0.total_cmp(&a.0));
    let piores: Vec<_> = linhas.iter().take(5).map(|l| l.1.as_str()).collect();
    // Medido 02/11: 0,146 em 41 776 colunas, pior arrasto 0,670 (fronteiras do `floor`, o ±1 do
    // Trial). As constantes da rodada 2 sem o −1 da barra e contando o passo desde o premir: 0,31.
    assert!(
        m < 0.16 && pior < 0.75,
        "erro médio {m:.3} (tecto 0,16) em {n} colunas, pior {pior:.3} (tecto 0,75):\n{}",
        piores.join("\n")
    );
}
