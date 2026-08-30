//! A razão medida e a FAMÍLIA de cada molde — a tabela que o gate estrutural afirma.
//!
//! ⛔ Até 2026-08-30 isto imprimia a razão contra um LIMIAR. O limiar morreu: ver
//! `derive::Derived::grows_by_refining`.
use ph2d_node_source_lsystem::{PRESETS, probe_grows_by_refining, probe_growth_ratio_raw, shape};
fn main() {
    println!("molde      razão crua   família");
    for p in PRESETS {
        let r = probe_growth_ratio_raw(p.axiom, p.rules, &[("angle", p.angle)]);
        let f = probe_grows_by_refining(p.axiom, p.rules, &[("angle", p.angle)]);
        println!(
            "{:9}  {r:9.4}   {}",
            p.label,
            if f { "REFINA" } else { "ponta" }
        );
    }
    let g = shape::rules(&shape::Shape {
        branches: 2.0,
        segments: 1.0,
        variation: 0.0,
        bend: 0.0,
    });
    println!("\nmodo GUIADO (o default do nó), varrendo o Length Scale:");
    for ls in [0.5f32, 0.7, 0.89, 0.90, 0.95, 1.0] {
        let r = probe_growth_ratio_raw(shape::AXIOM, &g, &[("length_scale", ls)]);
        let f = probe_grows_by_refining(shape::AXIOM, &g, &[("length_scale", ls)]);
        println!(
            "  length_scale {ls:.2}  razão {r:8.4}   {}",
            if f { "REFINA" } else { "ponta" }
        );
    }
}
