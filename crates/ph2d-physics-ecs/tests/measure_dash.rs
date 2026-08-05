//! **A varredura do ARRANQUE** (W14) — os números que a wave escreveu saíram
//! daqui (CLAUDE.md §0).
//!
//! `cargo test -p ph2d-physics-ecs --test measure_dash -- --ignored --nocapture`
//!
//! ⚠️ **Sondas, não gates:** elas imprimem e não afirmam. O que fica gateado é a
//! PROPRIEDADE (o `platform_dash`); o que fica aqui é a tabela que escolheu cada
//! constante.

#[path = "platform_dash_rig.rs"]
mod rig_fixture;

use rig_fixture::{DASH_TIME, dash_right, pose, rig, walk_right};

/// Quantos tiques de 1/60 s um arranque de `DASH_TIME` ocupa.
fn dash_ticks() -> u64 {
    (DASH_TIME * 60.0).ceil() as u64
}

/// **A DISTÂNCIA que um arranque cobre** — o número que o artista de facto
/// julga (*"atravessa aquele buraco?"*).
///
/// ⚠️ A coluna do CONTROLE é a mesma cena a andar: sem ela, o número do arranque
/// não diz se ele é um arranque ou só uma caminhada com outro nome.
///
/// ## Medido (2026-08-05, `time = 0,15 s`, 9 tiques)
///
/// | `dash_speed` | percorrido | autorado | a andar, no mesmo tempo |
/// |---|---|---|---|
/// | 8 | 1,200 m | 1,200 | 0,900 m |
/// | 12 | 1,800 m | 1,800 | 0,900 m |
/// | **18** | **2,700 m** | **2,700** | 0,900 m |
/// | 26 | 3,900 m | 3,900 | 0,900 m |
///
/// ⚠️ **O percorrido é o autorado ao milímetro**, e é essa a propriedade que a
/// lei entrega: `speed × time` não é uma estimativa do que acontece, é o que
/// acontece. É ela que torna o par de números julgável sem o app aberto.
#[test]
#[ignore]
fn measure_the_distance_a_dash_covers() {
    let ticks = dash_ticks();
    eprintln!("== o que um arranque cobre em {DASH_TIME} s ({ticks} tiques) ==");
    for speed in [8.0_f32, 12.0, 18.0, 26.0] {
        // O arranque.
        let mut r = rig(speed, 0.9);
        let t = r.run(0, 40, walk_right());
        let (x0, _) = pose(&r.sim);
        let t = r.run(t, ticks, dash_right());
        let (x1, _) = pose(&r.sim);
        // O controle: a MESMA cena, os mesmos tiques, sem apertar nada.
        let mut c = rig(speed, 0.9);
        let ct = c.run(0, 40, walk_right());
        let (cx0, _) = pose(&c.sim);
        c.run(ct, ticks, walk_right());
        let (cx1, _) = pose(&c.sim);
        eprintln!(
            "  speed {speed:5.1} -> arranque {:6.3} m   a andar {:6.3} m   (autorado {:5.3})",
            x1 - x0,
            cx1 - cx0,
            speed * DASH_TIME
        );
        let _ = t;
    }
}

/// **O quanto ele SAGA sem o cancelamento da gravidade** — a tabela que torna o
/// canal da W11 load-bearing aqui.
///
/// ⚠️ O boost já põe a velocidade vertical em zero no topo de cada tique, então
/// a metade que falta é **sub-tique**: sem o `gravity_hold` o corpo cai
/// `½·g·dt²` por tique *dentro* dele. É pequeno e é exactamente o defeito que a
/// W11 mediu na rampa — a velocidade certa e o DESLOCAMENTO errado.
#[test]
#[ignore]
fn measure_the_sag_of_an_airborne_dash() {
    let ticks = dash_ticks();
    let mut r = rig(18.0, 6.0);
    // Cai um pouco, para o arranque acontecer a meio de uma queda de verdade.
    let t = r.run(0, 20, walk_right());
    let (_, y0) = pose(&r.sim);
    r.run(t, ticks, dash_right());
    let (_, y1) = pose(&r.sim);
    eprintln!("== um arranque no AR ==");
    eprintln!(
        "  y {y0:.4} -> {y1:.4}   queda {:.4} m em {ticks} tiques",
        y0 - y1
    );
    eprintln!(
        "  (a queda LIVRE nos mesmos tiques seria da ordem de {:.4} m)",
        0.5 * 9.81 * (ticks as f32 / 60.0).powi(2)
    );
}
