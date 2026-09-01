//! **A ESCADA DE TAMANHOS, e a curva fina que ela amostra** — a bancada que o remap do `Growth`
//! inverte (2026-08-31).
//!
//! Sem argumentos imprime a escada de cada molde e as razões entre degraus. Com um nome, faz
//! uma varredura FINA do tamanho contra `g` — que é como se descobriu que o `Sprig` fica
//! **parado** no primeiro quarto de cada geração.
use ph2d_node_source_lsystem::{PRESETS, param, probe_build, probe_size_ladder};
use ph2d_nodegraph::attr::Column;

fn mean_width(v: &[[f32; 2]]) -> f32 {
    let mut acc = 0.0f64;
    for k in 0..32 {
        let a = std::f32::consts::PI * k as f32 / 32.0;
        let (c, s) = (a.cos(), a.sin());
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for q in v {
            let t = q[0] * c + q[1] * s;
            lo = lo.min(t);
            hi = hi.max(t);
        }
        acc += f64::from(hi - lo);
    }
    (acc / 32.0) as f32
}

fn main() {
    let want: Vec<String> = std::env::args().skip(1).collect();
    for p in PRESETS {
        if !want.is_empty() && !want.iter().any(|w| w == p.label) {
            continue;
        }
        let ov = [
            (param::ANGLE, p.angle),
            (param::STEP, p.step),
            (param::WIDTH, p.width),
        ];
        let l = probe_size_ladder(p.axiom, p.rules, p.generations, &ov);
        let razoes: Vec<String> = l
            .windows(2)
            .map(|w| format!("{:.4}", w[1] / w[0]))
            .collect();
        println!(
            "{:8} G={:<4} escada {:?}\n         razoes  {}",
            p.label,
            p.generations,
            l.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>(),
            razoes.join(" ")
        );
        if want.is_empty() {
            continue;
        }
        println!("     g   elems    tamanho    passo");
        let mut prev = 0.0f32;
        let mut g = 1.0f32;
        while g <= p.generations + 1e-4 {
            let s = probe_build(p.axiom, p.rules, g, &ov);
            let w = match s.get("P") {
                Some(Column::Vec2(v)) => mean_width(v),
                _ => 0.0,
            };
            let (x0, x1, y0, y1) = match s.get("P") {
                Some(Column::Vec2(v)) => (
                    v.iter().map(|q| q[0]).fold(f32::MAX, f32::min),
                    v.iter().map(|q| q[0]).fold(f32::MIN, f32::max),
                    v.iter().map(|q| q[1]).fold(f32::MAX, f32::min),
                    v.iter().map(|q| q[1]).fold(f32::MIN, f32::max),
                ),
                _ => (0.0, 0.0, 0.0, 0.0),
            };
            let soma: f32 = match s.get("len") {
                Some(Column::Scalar(v)) => v.iter().sum(),
                _ => 0.0,
            };
            println!(
                "{g:6.2} {:7} {w:10.5} {:8.5}   x[{x0:.4},{x1:.4}] y[{y0:.4},{y1:.4}]  tinta {soma:.4}",
                s.count(),
                if prev > 0.0 { w - prev } else { 0.0 }
            );
            prev = w;
            g += 0.05;
        }
    }
}
