//! **A medição das fatias do Power Stroke** (`PIECES`) — o número na doc de `expand.rs` é o
//! que ISTO deu.
//!
//! Duas grandezas, porque há dois modos de falha: o TEMPO (é um comando de edição, não um
//! frame) e o DEGRAU — o quanto a largura salta entre fatias vizinhas no ponto mais inclinado
//! do perfil, que é o serrilhado que se VÊ no contorno.
//!
//! `#[ignore]`: é uma sonda de calibração, não um gate. Rode com
//! `cargo test -p ph2d-vec-boolean --release measure_power_stroke -- --ignored --nocapture`.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, WidthProfile};
use std::time::Instant;

/// Uma senoide de 4 cúbicas — curvatura que muda de sinal, que é onde um offset ingênuo
/// falharia e onde as fatias têm de emendar bem.
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

/// O maior salto de largura entre duas fatias vizinhas, como FRAÇÃO da largura de pico —
/// puramente do perfil e da contagem, sem rodar o motor.
fn step_ratio(profile: &WidthProfile, pieces: usize) -> f64 {
    let w = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let t = (i as f64 + 0.5) / pieces as f64;
        profile.at(t)
    };
    (1..pieces)
        .map(|i| (w(i) - w(i - 1)).abs())
        .fold(0.0_f64, f64::max)
        / profile.peak()
}

#[test]
#[ignore = "sonda de calibração — roda sob demanda, em release"]
fn measure_power_stroke_pieces() {
    let path = sine();
    let profile = WidthProfile {
        start: 0.2,
        mid: 2.0,
        end: 0.2,
        position: 0.5,
    };

    // ⚠️ O TEMPO é o da const `PIECES` EM VIGOR — este harness não a varre (ela é `const`).
    // A coluna `ms` da tabela na doc de `expand.rs` foi levantada editando a const e
    // re-rodando isto, uma linha por valor. A 1ª versão deste teste extrapolava supondo custo
    // linear e errou por 8× — o custo é SUPERLINEAR, porque o sweep varre tudo o que já
    // acumulou. Extrapolação não é medição.
    let t0 = Instant::now();
    let out = ph2d_vec_boolean::power_stroke(&path, &profile);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert!(!out.is_empty(), "o power stroke produziu geometria");
    let area: f64 = out.iter().map(ph2d_vec_boolean::area).sum();
    println!("const PIECES em vigor: {ms:.1} ms, area {area:.4}");

    // O DEGRAU é função só do perfil e da contagem — este, sim, se varre sem rodar o motor.
    println!("| fatias | degrau |");
    println!("|--------|--------|");
    for pieces in [8_usize, 16, 32, 64, 128] {
        println!(
            "| {pieces:>6} | {:>5.1}% |",
            step_ratio(&profile, pieces) * 100.0
        );
    }
}
