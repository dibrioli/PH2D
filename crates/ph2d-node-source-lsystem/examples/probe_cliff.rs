//! ⛔ O PRECIPÍCIO DO LIMIAR — a bancada que o achou, e que prova que ele morreu.
//!
//! Até 2026-08-30 a família saía de `razão >= 1,25`. O modo GUIADO (o default do nó) ficou a
//! `0,017 %` do limiar quando a régua mudou, e um passo do `Length Scale` atravessava-o.
use ph2d_node_source_lsystem::{
    MODE_GUIDED, param, probe_build, probe_grows_by_refining, probe_growth_ratio_raw, shape,
};
use ph2d_nodegraph::attr::Column;

fn size(s: &ph2d_nodegraph::attr::Stream) -> f32 {
    let Some(Column::Vec2(v)) = s.get("P") else {
        return 0.0;
    };
    let mut t = 0.0f64;
    for k in 0..64 {
        let a = std::f32::consts::PI * k as f32 / 64.0;
        let (c, sn) = (a.cos(), a.sin());
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for q in v {
            let p = q[0] * c + q[1] * sn;
            lo = lo.min(p);
            hi = hi.max(p);
        }
        t += f64::from(hi - lo);
    }
    (t / 64.0) as f32
}

fn main() {
    let g = shape::rules(&shape::Shape {
        branches: 2.0,
        segments: 1.0,
        variation: 0.0,
        bend: 0.0,
    });
    println!(" length_scale   razão   família   tamanho (Growth=0,6)   salto");
    let mut prev = 0.0f32;
    for ls in [0.84f32, 0.86, 0.88, 0.89, 0.90, 0.91, 0.92, 0.94] {
        let r = probe_growth_ratio_raw(shape::AXIOM, &g, &[(param::LENGTH_SCALE, ls)]);
        let f = probe_grows_by_refining(shape::AXIOM, &g, &[(param::LENGTH_SCALE, ls)]);
        let s = size(&probe_build(
            shape::AXIOM,
            &g,
            5.0,
            &[
                (param::MODE, MODE_GUIDED as f32),
                (param::LENGTH_SCALE, ls),
                (param::GROWTH, 0.6),
            ],
        ));
        let jump = if prev > 0.0 {
            (s / prev - 1.0) * 100.0
        } else {
            0.0
        };
        println!(
            "{ls:12.2} {r:8.4}   {:6}   {s:20.4}   {jump:+6.1} %",
            if f { "REFINA" } else { "ponta" }
        );
        prev = s;
    }
}
