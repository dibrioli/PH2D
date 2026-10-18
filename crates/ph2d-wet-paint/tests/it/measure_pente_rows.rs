//! **Onde a divisão por linhas do PENTE do relevo se paga** — a sonda do piso [`MIN_CELLS_PENTE`] (W3b,
//! doc 50_medicao §4). Para cada raio, o MESMO pente (a caixa da pegada da lâmina do óleo, ⅓ do quadrado; a
//! lei real dos fios e do relevo) pelas duas rotas forçadas, ALTERNADAS dentro da ronda; fica o MÍNIMO.
//!
//! ```text
//! PH2D_RELEASE="mede velocidade" bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --release --test it \
//!     measure_pente_rows -- --ignored --nocapture
//! ```

use ph2d_wet_paint::bristles_fios::Fios;
use ph2d_wet_paint::bristles_relevo::{Relevo, entre};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::{MIN_CELLS_PENTE, Rows};
use std::time::Instant;

const AMOSTRAS: usize = 15;

/// `(ms série, ms paralelo, células da caixa)` do pente de um dab de raio `r`.
fn mede(r: f64) -> (f64, f64, usize) {
    let lado = (4.0 * r) as usize + 200;
    let mut e = Engine::new(lado, lado);
    let c = lado as f64 / 2.0;
    let meia = [r, r / 3.0];
    let fios = Fios {
        pressao: 1.2,
        levanta: 0.4,
        largura_px: 1.5,
        ..Fios::NEUTRO
    }
    .resolvido(300.0, [1.0, 0.0], 1.0);
    let relevo = Relevo {
        altura_px: 0.75,
        escava: 0.5,
    };
    let sil = move |x: i32, y: i32| -> f64 {
        let (dx, dy) = ((f64::from(x) - c) / meia[0], (f64::from(y) - c) / meia[1]);
        let d = dx * dx + dy * dy;
        if d >= 1.0 { 0.0 } else { 1.0 - d }
    };
    let alvo = move |x: i32, y: i32| -> f64 {
        let ([a, b], t) = fios.entre_fios([f64::from(x) - c, f64::from(y) - c]);
        let h = |k| relevo.alvo_do_fio(k, fios.ganho_do_fio(k), 1.0);
        entre(h(a), h(b), t)
    };
    let mut melhor = [f64::INFINITY; 2];
    for _ in 0..AMOSTRAS {
        for (k, mode) in [Rows::Serial, Rows::Parallel].into_iter().enumerate() {
            let t0 = Instant::now();
            e.pentear_relevo_rows(c, c, meia, &sil, &alvo, Some(mode));
            melhor[k] = melhor[k].min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    let caixa = (2 * (meia[0].ceil() as usize + 1) + 1) * (2 * (meia[1].ceil() as usize + 1) + 1);
    (melhor[0], melhor[1], caixa)
}

#[test]
#[ignore = "sonda de medicao (relogio); rode com --release --ignored --nocapture"]
fn measure_pente_rows() {
    println!(
        "load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!(
        "piso actual: {MIN_CELLS_PENTE} células\n{:>6} {:>8} | {:>9} {:>9} {:>6}",
        "raio", "células", "sér ms", "par ms", "razão"
    );
    for r in [8.0, 16.0, 24.0, 32.0, 40.0, 51.5, 80.0, 120.0, 200.0] {
        let (s, p, n) = mede(r);
        println!("{r:>6} {n:>8} | {s:>9.4} {p:>9.4} {:>5.2}x", s / p);
    }
}

/// ⭐ **As duas rotas do pente são a MESMA resposta** ao bit (a garantia estrutural do [`ph2d_wet_paint::par`]).
#[test]
fn o_pente_por_linhas_e_o_mesmo_em_serie_e_em_paralelo() {
    let campo = |mode| {
        let mut e = Engine::new(300, 300);
        let sil = |x: i32, y: i32| -> f64 {
            let (dx, dy) = ((f64::from(x) - 150.0) / 80.0, (f64::from(y) - 150.0) / 30.0);
            (1.0 - dx * dx - dy * dy).max(0.0)
        };
        let alvo = |x: i32, y: i32| -> f64 { (f64::from(x) * 0.37).sin() + f64::from(y) * 0.01 };
        for _ in 0..3 {
            e.pentear_relevo_rows(150.0, 150.0, [80.0, 30.0], &sil, &alvo, Some(mode));
        }
        e.layers[e.active_layer].grid.altura.clone()
    };
    let (s, p) = (campo(Rows::Serial), campo(Rows::Parallel));
    assert!(s.iter().any(|&h| h != 0.0));
    assert_eq!(s, p);
}
