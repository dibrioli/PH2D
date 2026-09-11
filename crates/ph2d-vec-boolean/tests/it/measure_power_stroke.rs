//! **A medição da fita do Power Stroke** — a rugosidade da borda e o custo, agora que o motor
//! é a fita de trilhos (`ribbon_into` / `RIBBON_SAMPLES` em `expand.rs`), não mais a união de
//! discos.
//!
//! O que MUDOU: a antiga versão fatiava o arco e unia discos, e a métrica era o *degrau* de
//! largura entre fatias vizinhas (o serrilhado). A fita segue o perfil direto; a métrica agora
//! é o DESVIO da borda ao perfil (o festão residual), medido numa linha reta onde o perfil é a
//! única coisa que molda a borda. `RIBBON_SAMPLES` é `const` em `expand.rs` — para varrê-la,
//! edite lá e re-rode isto.
//!
//! `#[ignore]`: sonda de calibração, não gate. Rode com
//! `cargo test -p ph2d-vec-boolean --release measure_power_stroke -- --ignored --nocapture`.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, WidthProfile};
use std::time::Instant;

/// Uma senoide de 4 cúbicas — curvatura que muda de sinal, onde os trilhos se cruzam e o pinço
/// deixa lascas (que o `drop_slivers` varre). O caso mais caro.
fn sine() -> VecPath {
    let mut verts = Vec::new();
    for i in 0..5 {
        let x = f64::from(i) * 2.0;
        let y = if i % 2 == 0 { -1.0 } else { 1.0 };
        let mut v = VecVertex::corner([x, y]);
        v.in_handle = [x - 0.9, y];
        v.out_handle = [x + 0.9, y];
        verts.push(v);
    }
    let mut p = VecPath {
        verts,
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.6));
    p
}

fn straight(len: f64, width: f64) -> VecPath {
    let mut p = VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([len, 0.0])],
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), width));
    p
}

/// O maior desvio da borda de cima ao perfil, numa linha reta — o festão residual, em unidades
/// de mundo (a linha vale 20, largura 1, pico 2 ⇒ meia-largura de pico 1.0). Lê as âncoras do
/// polígono assado direto (o motor devolve poligonal densa; sem precisar de kurbo).
fn ripple(out: &[VecPath], len: f64, width: f64, profile: &WidthProfile) -> f64 {
    let pts: Vec<[f64; 2]> = out[0].verts.iter().map(|v| v.anchor).collect();
    let top_at = |x: f64| {
        pts.iter()
            .filter(|p| (p[0] - x).abs() < 0.08)
            .map(|p| p[1])
            .fold(f64::MIN, f64::max)
    };
    let mut dev = 0.0_f64;
    for i in 10..=190 {
        let x = len * f64::from(i) / 200.0;
        let predicted = 0.5 * width * profile.at(x / len);
        let m = top_at(x);
        if m > f64::MIN {
            dev = dev.max((m - predicted).abs());
        }
    }
    dev
}

#[test]
#[ignore = "sonda de calibração — roda sob demanda, em release"]
fn measure_power_stroke_ribbon() {
    let profile = WidthProfile {
        start: 0.2,
        mid: 2.0,
        end: 0.2,
        position: 0.5,
    };

    // CUSTO no caso pior (a senoide que auto-cruza), com o `RIBBON_SAMPLES` EM VIGOR.
    let path = sine();
    let t0 = Instant::now();
    let out = ph2d_vec_boolean::power_stroke(&path, &profile.to_stops());
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert!(!out.is_empty());
    let area: f64 = out.iter().map(ph2d_vec_boolean::area).sum();
    println!("senoide: {ms:.2} ms, {} peça(s), area {area:.4}", out.len());

    // RUGOSIDADE: o desvio da borda ao perfil numa reta, com o `RIBBON_SAMPLES` em vigor.
    let (len, width) = (20.0, 1.0);
    let out = ph2d_vec_boolean::power_stroke(&straight(len, width), &profile.to_stops());
    let dev = ripple(&out, len, width, &profile);
    println!(
        "reta: desvio máx da borda ao perfil = {dev:.5} (o festão da união de discos era ~0.08)"
    );
}
