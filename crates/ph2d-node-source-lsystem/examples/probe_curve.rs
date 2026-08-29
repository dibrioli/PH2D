//! Sonda: **a FORMA da curva de crescimento**, não só o pior passo.
//!
//! Report do Enio (2026-08-29): *"Melhorou muito. Mas o crescimento dos que não cresciam
//! suavemente não é linear"*. `pior_passo` mede o SALTO; ele está a falar da DERIVADA.
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::Column;

fn size(p: &ls::Preset, g: f32) -> f32 {
    let s = ls::probe_build(
        p.axiom,
        p.rules,
        g,
        &[
            (ls::param::MODE, ls::MODE_GRAMMAR as f32),
            (ls::param::ANGLE, p.angle),
            (ls::param::STEP, p.step),
            (ls::param::CONTINUOUS_ANGLE, 1.0),
        ],
    );
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => {
            let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
            let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
            let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
            let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
            // A DIAGONAL: `max(w, h)` tem um joelho onde os dois eixos se cruzam.
            ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
        }
        _ => 0.0,
    }
}

fn main() {
    const N: usize = 20;
    println!("O TAMANHO DENTRO DE UMA GERACAO (frac 0,00 -> 1,00), normalizado ao inicio");
    println!("uma rampa LINEAR imprimiria 1.00 1.05 1.10 ... e um `d` constante\n");
    for p in ls::PRESETS {
        let g0 = (p.generations - 1.0).max(2.0).floor();
        let a = size(p, g0);
        let b = size(p, g0 + 1.0);
        println!(
            "{:8} geracao {g0:.0} -> {:.0}   ({a:.3} -> {b:.3}, razao {:.2}x)",
            p.label,
            g0 + 1.0,
            b / a.max(1e-6)
        );
        print!("   norm ");
        let vals: Vec<f32> = (0..=N)
            .map(|k| size(p, g0 + k as f32 / N as f32) / a.max(1e-6))
            .collect();
        for v in &vals {
            print!("{v:5.2}");
        }
        println!();
        print!("   d    ");
        let d: Vec<f32> = vals.windows(2).map(|w| w[1] - w[0]).collect();
        for v in &d {
            print!("{v:5.2}");
        }
        let (lo, hi) = (
            d.iter().copied().fold(f32::MAX, f32::min),
            d.iter().copied().fold(f32::MIN, f32::max),
        );
        let mean = d.iter().sum::<f32>() / d.len() as f32;
        println!(
            "     d_min {lo:+.3}  d_max {hi:+.3}  media {mean:+.3}  ONDULACAO {:.1}x",
            (hi - lo).abs() / mean.abs().max(1e-6)
        );
    }
}
