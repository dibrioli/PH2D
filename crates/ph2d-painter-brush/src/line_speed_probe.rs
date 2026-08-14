//! **SONDA (não é gate): o que o arremesso de HOJE faz com o gesto de um artista.**
//!
//! Rodar: `cargo test -p ph2d-painter-brush --release measure_the -- --ignored --nocapture`

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::LineKind;
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};

fn sp(kind: LineKind, radius: f32) -> BrushSpec {
    BrushSpec {
        radius_px: radius,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: kind,
        ..Default::default()
    }
}

fn dyn_() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// **O gesto REAL: velocidade que MUDA.** Um traço de artista não corre a velocidade constante —
/// ele acelera na reta e freia na curva, e é a MUDANÇA que estica o caminho da tinta (o arremesso
/// vale `v·T`, então `d(arremesso)/d(arco) ≈ Δv/v`). A pergunta é a do olho: **o maior vão entre
/// dabs vizinhos passa de um DIÂMETRO?** Passou, a linha está pontilhada.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_gap_on_a_gesture_that_changes_speed() {
    println!("[speed] GESTO REAL: senoide de velocidade (200..3000 px/s), 3 voltas");
    println!(
        "{:>8} {:>10}  {:>12} {:>12} {:>12}  {:>12}",
        "raio", "tipo", "maior VAO", "vao/diam", "dabs", "veredito"
    );
    for radius in [4.0f32, 10.0, 25.0] {
        for kind in [LineKind::None, LineKind::Speed] {
            let mut s = Stroke::new(sp(kind, radius), dyn_(), 1);
            let mut out = Vec::new();
            let mut centres: Vec<[f32; 2]> = Vec::new();
            let mut p = [50.0f32, 300.0];
            s.begin(
                StrokePoint {
                    pos: p,
                    pressure: 1.0,
                },
                &mut out,
            );
            centres.extend(out.iter().map(|d| d.center));
            for f in 0..30u32 {
                let ph = f32::from(u8::try_from(f % 10).unwrap_or(0)) / 10.0;
                let u = if ph < 0.5 { ph * 2.0 } else { (1.0 - ph) * 2.0 };
                let per = (200.0 + 2800.0 * u) / 60.0 / 4.0;
                for _ in 0..4 {
                    p[0] += per;
                    p[1] += per * 0.35;
                    s.extend(
                        StrokePoint {
                            pos: p,
                            pressure: 1.0,
                        },
                        &mut out,
                    );
                    centres.extend(out.iter().map(|d| d.center));
                }
                s.tick(1.0 / 60.0, &mut out);
                centres.extend(out.iter().map(|d| d.center));
            }
            let mut worst = 0.0f32;
            for w in centres.windows(2) {
                worst = worst.max((w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]));
            }
            let diam = 2.0 * radius;
            println!(
                "{:>8.0} {:>10} {:>12.1} {:>12.2} {:>12}  {:>12}",
                radius,
                if kind == LineKind::None {
                    "controle"
                } else {
                    "Speed"
                },
                worst,
                worst / diam,
                centres.len(),
                if worst > diam { "PONTILHADO" } else { "solido" }
            );
        }
    }
    println!(
        "[speed] leitura: `vao/diam` > 1 e a tinta sai em CONTAS; o olho ja ve a partir de ~0,7."
    );
}

/// Um ARCO rápido (quarto de círculo, raio 150, percorrido em 6 quadros) — a pergunta é o que o
/// arremesso faz numa CURVA: ele voa para FORA do giro (o que o manual descreve) ou corta para
/// DENTRO?  E a linha continua CONTÍNUA?
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_throw_on_a_fast_arc() {
    println!("[speed] ARCO RAPIDO: quarto de circulo r=150 em 6 quadros (~2 400 px/s)");
    println!(
        "{:>10}  {:>12} {:>12}  {:>12} {:>12}",
        "tipo", "maior VAO", "vao/diam", "raio medio", "para FORA?"
    );
    for kind in [LineKind::None, LineKind::Speed] {
        let mut s = Stroke::new(sp(kind, 8.0), dyn_(), 1);
        let mut out = Vec::new();
        let mut centres: Vec<[f32; 2]> = Vec::new();
        let r = 150.0f32;
        let n_frames = 6;
        let per_frame = 8;
        let total = n_frames * per_frame;
        let pt = |i: usize| {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32 / total as f32;
            // Quarto de circulo por bisseccao de vetores unitarios (sem transcendental — HR-5).
            let (mut u, mut v) = ([1.0f32, 0.0f32], [0.0f32, 1.0f32]);
            let (mut lo, mut hi) = (0.0f32, 1.0f32);
            for _ in 0..20 {
                let m = 0.5 * (lo + hi);
                let mid = [u[0] + v[0], u[1] + v[1]];
                let l = mid[0].hypot(mid[1]);
                let mid = [mid[0] / l, mid[1] / l];
                if t < m {
                    v = mid;
                    hi = m;
                } else {
                    u = mid;
                    lo = m;
                }
            }
            [u[0] * r, u[1] * r]
        };
        s.begin(
            StrokePoint {
                pos: pt(0),
                pressure: 1.0,
            },
            &mut out,
        );
        centres.extend(out.iter().map(|d| d.center));
        for f in 0..n_frames {
            for k in 1..=per_frame {
                s.extend(
                    StrokePoint {
                        pos: pt(f * per_frame + k),
                        pressure: 1.0,
                    },
                    &mut out,
                );
                centres.extend(out.iter().map(|d| d.center));
            }
            s.tick(1.0 / 60.0, &mut out);
            centres.extend(out.iter().map(|d| d.center));
        }
        let mut worst = 0.0f32;
        for w in centres.windows(2) {
            worst = worst.max((w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]));
        }
        #[allow(clippy::cast_precision_loss)]
        let mean_r = centres.iter().map(|q| q[0].hypot(q[1])).sum::<f32>() / centres.len() as f32;
        println!(
            "{:>10}  {:>12.1} {:>12.2}  {:>12.1} {:>12}",
            if kind == LineKind::None {
                "controle"
            } else {
                "Speed"
            },
            worst,
            worst / 16.0,
            mean_r,
            if mean_r > r + 1.0 {
                "FORA"
            } else if mean_r < r - 1.0 {
                "DENTRO"
            } else {
                "sobre"
            }
        );
    }
    println!("[speed] leitura: `raio medio` diz de que lado do giro a tinta cai.");
}
