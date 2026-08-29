//! Sonda: **o arrasto INTEIRO do slider**, não uma travessia só — e o tamanho de cada molde.
//!
//! Report do Enio (2026-08-29): *"ainda não linear. dragon é bem menor que os outros"*.
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::Column;

fn diag(p: &ls::Preset, g: f32) -> f32 {
    let s = ls::probe_build(
        p.axiom,
        p.rules,
        g,
        &[
            (ls::param::MODE, ls::MODE_GRAMMAR as f32),
            (ls::param::ANGLE, p.angle),
            (ls::param::STEP, p.step),
            (ls::param::WIDTH, p.width),
        ],
    );
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => {
            let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
            let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
            let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
            let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
            ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
        }
        _ => 0.0,
    }
}

fn main() {
    println!("== 1. O TAMANHO NO VALOR QUE O MOLDE DECLARA ==");
    println!(
        "{:8} {:>6} {:>10} {:>10} {:>10}",
        "molde", "gens", "largura", "altura", "diagonal"
    );
    for p in ls::PRESETS {
        let s = ls::probe_build(
            p.axiom,
            p.rules,
            p.generations,
            &[
                (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                (ls::param::ANGLE, p.angle),
                (ls::param::STEP, p.step),
                (ls::param::WIDTH, p.width),
            ],
        );
        let (w, h) = match s.get("P") {
            Some(Column::Vec2(v)) if !v.is_empty() => {
                let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
                let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
                let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
                let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
                (x1 - x0, y1 - y0)
            }
            _ => (0.0, 0.0),
        };
        println!(
            "{:8} {:>6.1} {w:>10.3} {h:>10.3} {:>10.3}",
            p.label,
            p.generations,
            (w * w + h * h).sqrt()
        );
    }

    println!("\n== 2. O ARRASTO INTEIRO (g = 1 ate' o que o molde declara) ==");
    println!("   uma rampa LINEAR daria passos iguais do principio ao fim\n");
    const N: usize = 24;
    for p in ls::PRESETS {
        let hs: Vec<f32> = (0..=N)
            .map(|k| diag(p, 1.0 + (p.generations - 1.0) * k as f32 / N as f32))
            .collect();
        let d: Vec<f32> = hs.windows(2).map(|w| w[1] - w[0]).collect();
        let lo = d.iter().copied().fold(f32::MAX, f32::min);
        let hi = d.iter().copied().fold(f32::MIN, f32::max);
        let mean = d.iter().sum::<f32>() / d.len() as f32;
        print!("{:8} ", p.label);
        for v in &hs {
            print!("{v:5.2}");
        }
        println!(
            "\n         passos: min {lo:+.3} max {hi:+.3} media {mean:+.3}   ONDULACAO {:.1}x",
            (hi - lo).abs() / mean.abs().max(1e-6)
        );
    }
}
