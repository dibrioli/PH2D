//! **DE QUE A FAIXA É FEITA, E DE ONDE VINHA A ESPÍCULA** — as sondas do trilho de fora e das
//! travessas, irmãs das do [`super::ribbon_probe`] (que mede o que a fita CUSTA e quanto ela ATRASA).
//!
//! ⚠️ **O corte é de ASSUNTO:** *quanto tempo a fita atrasa e o que ela cobra por evento* e *de que
//! é feita a faixa* são perguntas diferentes, e foi a segunda que resolveu o report de 2026-08-15.
//!
//! Rodar: `cargo test -p ph2d-painter-brush --release measure_ -- --ignored --nocapture`

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::LineKind;
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};

fn spec(weight: f32, friction: f32, gravity: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: LineKind::Ribbon,
        ribbon_weight: weight,
        ribbon_friction: friction,
        ribbon_gravity: gravity,
        ..Default::default()
    }
}

fn plain() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// **Quantos fios uma fita costura, e de que tamanho?** — a pergunta que separa *"a faixa não abre"*
/// de *"a faixa não existe"*, e ela é do MOTOR (o depósito é do tool).
#[test]
#[ignore = "sonda"]
fn measure_the_ribbon_rungs() {
    for (nome, w, ru) in [
        ("rungs 0.0", 0.20f32, 0.0f32),
        ("rungs 0.5", 0.20, 0.5),
        ("rungs 1.0", 0.20, 1.0),
        ("peso 0.45", 0.45, 0.5),
    ] {
        let mut sp = spec(w, 0.30, 0.0);
        sp.radius_px = 6.0;
        sp.ribbon_rungs = ru;
        let mut s = Stroke::new(sp, plain(), 1);
        let mut out = Vec::new();
        let mut fios = Vec::new();
        let mut total = 0usize;
        let mut comp = 0.0f32;
        let p = |x: f32| StrokePoint {
            pos: [x, 300.0],
            pressure: 1.0,
        };
        s.begin(p(100.0), &mut out);
        for i in 1..=120 {
            #[allow(clippy::cast_precision_loss)]
            let x = 100.0 + i as f32 * 20.0;
            s.extend(p(x), &mut out);
            s.tick(1.0 / 60.0, &mut out);
            s.take_threads(&mut fios);
            for f in &fios {
                total += 1;
                comp += ((f[2] - f[0]).powi(2) + (f[3] - f[1]).powi(2)).sqrt();
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let medio = if total == 0 { 0.0 } else { comp / total as f32 };
        println!("[fios] {nome}: {total} fios, comprimento medio {medio:.1} px");
    }
}

/// **DE ONDE VÊM AS ESPÍCULAS?** — o report do Enio (2026-08-15, com a foto): a faixa ficou boa e
/// sobraram riscos retos e ESCUROS atravessando o desenho, finos demais para serem dabs.
///
/// A sonda classifica cada fio pela ordem de emissão (`rung0, rung1, rail01, rung2, rail12, …` ⇒ de
/// 1 em diante, ímpar = TRAVESSA, par = TRILHO) e reporta a distribuição de comprimentos das duas
/// famílias sobre o gesto da foto: uma onda RÁPIDA com inversões.
#[test]
#[ignore = "sonda"]
fn measure_where_the_band_spikes_come_from() {
    for (nome, w, fr) in [
        ("default 0,45/0,30", 0.45f32, 0.30f32),
        ("peso 1,0", 1.0, 0.30),
    ] {
        let mut sp = spec(w, fr, 0.0);
        sp.radius_px = 6.0;
        sp.ribbon_rungs = 0.5;
        let mut s = Stroke::new(sp, plain(), 1);
        let (mut out, mut buf, mut all) = (Vec::new(), Vec::new(), Vec::new());
        // A onda da foto: rápida, com inversões duras (é nelas que a fita PARA e reverte).
        let pt = |u: f32| StrokePoint {
            pos: [80.0 + u * 900.0, 400.0 + (u * 16.0).sin() * 220.0],
            pressure: 1.0,
        };
        s.begin(pt(0.0), &mut out);
        for i in 1..=140 {
            #[allow(clippy::cast_precision_loss)]
            let u = i as f32 / 140.0;
            s.extend(pt(u), &mut out);
            s.tick(1.0 / 60.0, &mut out);
            s.take_threads(&mut buf);
            all.append(&mut buf);
        }
        let comp = |t: &[f32; 4]| ((t[2] - t[0]).powi(2) + (t[3] - t[1]).powi(2)).sqrt();
        let mut trav: Vec<f32> = Vec::new();
        let mut trilho: Vec<f32> = Vec::new();
        for (i, t) in all.iter().enumerate() {
            if i == 0 || i % 2 == 1 {
                trav.push(comp(t));
            } else {
                trilho.push(comp(t));
            }
        }
        let est = |v: &mut Vec<f32>| {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            #[allow(clippy::cast_precision_loss)]
            let med = v[v.len() / 2];
            (
                med,
                v[v.len() - 1],
                v.iter().filter(|&&x| x > 4.0 * med).count(),
            )
        };
        let (tm, tmax, tlongos) = est(&mut trav);
        let (rm, rmax, rlongos) = est(&mut trilho);
        println!(
            "[espicula] {nome}: travessas n={} mediana {tm:.1} pior {tmax:.1} px \
             ({tlongos} acima de 4x a mediana) | trilho n={} mediana {rm:.1} pior {rmax:.1} px \
             ({rlongos} acima de 4x)",
            trav.len(),
            trilho.len()
        );
    }
}

/// **QUE FASE DESENHA A ESPÍCULA?** — a espícula é de DABS (a sonda dos fios inocentou o feixe), e
/// as três candidatas são: o gesto contínuo · a mão PARADA com o botão preso · a CAUDA do pen-up.
///
/// O oráculo é a maior corrida COLINEAR de dabs de cada fase: uma fita a percorrer uma curva emite
/// dabs que viram; um trecho reto e longo é a assinatura da espícula.
#[test]
#[ignore = "sonda"]
fn measure_which_phase_draws_the_spike() {
    let reta = |dabs: &[crate::stroke::Dab]| -> f32 {
        // A maior corrida de dabs cujo desvio da corda fica sob meio pixel.
        let (mut melhor, mut i) = (0.0f32, 0usize);
        while i + 2 < dabs.len() {
            let a = dabs[i].center;
            let mut j = i + 2;
            let mut fim = i + 1;
            while j < dabs.len() {
                let b = dabs[j].center;
                let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                let n = (dx * dx + dy * dy).sqrt();
                if n < 1e-6 {
                    j += 1;
                    continue;
                }
                let ok = (i + 1..j).all(|k| {
                    let p = dabs[k].center;
                    ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / n < 0.5
                });
                if !ok {
                    break;
                }
                fim = j;
                j += 1;
            }
            let e = dabs[fim].center;
            melhor = melhor.max(((e[0] - a[0]).powi(2) + (e[1] - a[1]).powi(2)).sqrt());
            i = fim.max(i + 1);
        }
        melhor
    };
    for (nome, w) in [("default 0,45", 0.45f32), ("peso 1,0", 1.0)] {
        let mut sp = spec(w, 0.30, 0.0);
        sp.radius_px = 6.0;
        sp.ribbon_rungs = 0.5;
        let pt = |u: f32| StrokePoint {
            pos: [80.0 + u * 900.0, 400.0 + (u * 16.0).sin() * 220.0],
            pressure: 1.0,
        };
        // Três fases, medidas SEPARADAS: o gesto, a pausa com o botão preso, a cauda do pen-up.
        let mut s = Stroke::new(sp, plain(), 1);
        let (mut out, mut gesto) = (Vec::new(), Vec::new());
        s.begin(pt(0.0), &mut out);
        for i in 1..=100 {
            #[allow(clippy::cast_precision_loss)]
            let u = i as f32 / 140.0;
            s.extend(pt(u), &mut out);
            gesto.extend_from_slice(&out);
            s.tick(1.0 / 60.0, &mut out);
            gesto.extend_from_slice(&out);
        }
        let mut pausa = Vec::new();
        for _ in 0..30 {
            #[allow(clippy::cast_precision_loss)]
            let u = 100.0f32 / 140.0;
            s.extend(pt(u), &mut out); // a mão PAROU, o botão continua preso
            pausa.extend_from_slice(&out);
            s.tick(1.0 / 60.0, &mut out);
            pausa.extend_from_slice(&out);
        }
        let mut cauda = Vec::new();
        s.finish(&mut out);
        cauda.extend_from_slice(&out);
        println!(
            "[fase] {nome}: gesto {} dabs / maior reta {:.0} px · PAUSA {} dabs / reta {:.0} px \
             · CAUDA {} dabs / reta {:.0} px",
            gesto.len(),
            reta(&gesto),
            pausa.len(),
            reta(&pausa),
            cauda.len(),
            reta(&cauda)
        );
    }
}
