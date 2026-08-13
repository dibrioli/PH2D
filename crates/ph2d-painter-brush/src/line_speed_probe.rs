//! **SONDA (não é gate): o que o arremesso de HOJE faz com um gesto de chicote.**
//!
//! Rodar: `cargo test -p ph2d-painter-brush --release measure_the_throw -- --ignored --nocapture`

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::LineKind;
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};

fn sp(kind: LineKind, amount: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 8.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: kind,
        line_speed_amount: amount,
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

/// Um CHICOTE: 200 px devagar para a direita, depois 400 px MUITO rápido para cima.
/// Devolve (dabs, bbox da tinta, bbox do gesto).
fn whip(spec: BrushSpec) -> (usize, [f32; 4], [f32; 4]) {
    let mut s = Stroke::new(spec, dyn_(), 1);
    let mut out = Vec::new();
    let mut all: Vec<[f32; 2]> = Vec::new();
    let mut path: Vec<[f32; 2]> = Vec::new();
    let dt = 1.0 / 60.0;
    let push = |o: &Vec<[f32; 2]>, all: &mut Vec<[f32; 2]>| all.extend(o.iter().copied());
    let mut p = [100.0f32, 500.0];
    s.begin(
        StrokePoint {
            pos: p,
            pressure: 1.0,
        },
        &mut out,
    );
    push(&out.iter().map(|d| d.center).collect(), &mut all);
    path.push(p);
    // Perna 1: 200 px em 10 quadros (lento: 1200 px/s).
    for _ in 0..10 {
        for _ in 0..4 {
            p[0] += 5.0;
            s.extend(
                StrokePoint {
                    pos: p,
                    pressure: 1.0,
                },
                &mut out,
            );
            push(&out.iter().map(|d| d.center).collect(), &mut all);
            path.push(p);
        }
        s.tick(dt, &mut out);
        push(&out.iter().map(|d| d.center).collect(), &mut all);
    }
    // Perna 2: 400 px para CIMA em 2 quadros (chicote: 12 000 px/s).
    for _ in 0..2 {
        for _ in 0..4 {
            p[1] -= 50.0;
            s.extend(
                StrokePoint {
                    pos: p,
                    pressure: 1.0,
                },
                &mut out,
            );
            push(&out.iter().map(|d| d.center).collect(), &mut all);
            path.push(p);
        }
        s.tick(dt, &mut out);
        push(&out.iter().map(|d| d.center).collect(), &mut all);
    }
    let bb = |v: &[[f32; 2]]| {
        let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for q in v {
            b[0] = b[0].min(q[0]);
            b[1] = b[1].min(q[1]);
            b[2] = b[2].max(q[0]);
            b[3] = b[3].max(q[1]);
        }
        b
    };
    (all.len(), bb(&all), bb(&path))
}

#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_throw_on_a_whip() {
    println!("[speed] CHICOTE: 200 px devagar (1 200 px/s) e depois 400 px rapido (12 000 px/s)");
    println!(
        "{:>10} {:>8}  {:>34}  {:>10} {:>10}",
        "tipo", "dabs", "bbox da TINTA", "extra-X", "extra-Y"
    );
    let (_, _, gp) = whip(sp(LineKind::None, 0.0));
    println!(
        "{:>10} {:>8}  gesto: [{:7.1} {:7.1} {:7.1} {:7.1}]",
        "gesto", "-", gp[0], gp[1], gp[2], gp[3]
    );
    for a in [0.0f32, 1.0, 4.0, 8.0] {
        let (n, bb, _) = whip(sp(LineKind::Speed, a));
        println!(
            "{:>10} {:>8}  [{:7.1} {:7.1} {:7.1} {:7.1}]  {:10.1} {:10.1}",
            format!("a={a}"),
            n,
            bb[0],
            bb[1],
            bb[2],
            bb[3],
            (bb[2] - gp[2]).max(gp[0] - bb[0]),
            (gp[1] - bb[1]).max(bb[3] - gp[3]),
        );
    }
    println!("[speed] leitura: `extra-` e quanto a TINTA passou do que a mao desenhou, por eixo.");
}

/// Um ARCO rápido (quarto de círculo, raio 150, percorrido em 6 quadros) — a pergunta é o que o
/// arremesso faz numa CURVA: ele voa para FORA do giro (o que o manual descreve) ou corta para
/// DENTRO?  E a linha continua CONTÍNUA, ou ela se parte em pedaços deslocados?
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_throw_on_a_fast_arc() {
    println!("[speed] ARCO RAPIDO: quarto de circulo r=150 em 6 quadros (~2 400 px/s)");
    println!(
        "{:>10}  {:>12} {:>12}  {:>12} {:>12}",
        "tipo", "maior VAO", "vao/passo", "raio medio", "para FORA?"
    );
    for a in [0.0f32, 1.0, 2.0, 4.0] {
        let kind = if a == 0.0 {
            LineKind::None
        } else {
            LineKind::Speed
        };
        let mut s = Stroke::new(sp(kind, a), dyn_(), 1);
        let mut out = Vec::new();
        let mut centres: Vec<[f32; 2]> = Vec::new();
        let c = [0.0f32, 0.0];
        let r = 150.0f32;
        let n_frames = 6;
        let per_frame = 8;
        let total = n_frames * per_frame;
        let pt = |i: usize| {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32 / total as f32;
            // Quarto de circulo por bisseccao de vetores unitarios (sem transcendental — HR-5).
            let (mut u, mut v) = ([1.0f32, 0.0f32], [0.0f32, 1.0f32]);
            let mut lo = 0.0f32;
            let mut hi = 1.0f32;
            for _ in 0..20 {
                let m = 0.5 * (lo + hi);
                let mid = [u[0] + v[0], u[1] + v[1]];
                let l = (mid[0] * mid[0] + mid[1] * mid[1]).sqrt();
                let mid = [mid[0] / l, mid[1] / l];
                if t < m {
                    v = mid;
                    hi = m;
                } else {
                    u = mid;
                    lo = m;
                }
            }
            [c[0] + u[0] * r, c[1] + u[1] * r]
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
            let d = ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
            worst = worst.max(d);
        }
        #[allow(clippy::cast_precision_loss)]
        let mean_r = centres
            .iter()
            .map(|q| (q[0] * q[0] + q[1] * q[1]).sqrt())
            .sum::<f32>()
            / centres.len() as f32;
        let step = 2.0 * 8.0 * 0.1; // spacing x diametro = o passo nominal entre dabs
        println!(
            "{:>10}  {:>12.1} {:>12.1}  {:>12.1} {:>12}",
            format!("a={a}"),
            worst,
            worst / step,
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
    println!(
        "[speed] leitura: `vao/passo` > 2 e a linha PARTIDA; `raio medio` diz de que lado do giro a tinta cai."
    );
}
