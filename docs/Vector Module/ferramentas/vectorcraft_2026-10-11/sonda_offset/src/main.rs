// Sonda: o offset do PH2D na estrela do gate, mesmas 600 combinações da corrida do VectorCraft.
use ph2d_vec_scene::{LineJoin, OffsetSide, ShapeKind, cook};
use std::fmt::Write;
fn svg_d(p: &ph2d_vec_scene::VecPath) -> String {
    let b = ph2d_vec_render::build_fill_bezpath(p);
    let mut s = String::new();
    let t = |pt: kurbo_pt::P| ((pt.0 + 2.2) * 100.0 + 600.0, pt.1 * 100.0 + 600.0);
    for el in b.elements() {
        use kurbo::PathEl::*;
        match *el {
            MoveTo(a) => { let (x, y) = t((a.x, a.y)); write!(s, "M{x:.4} {y:.4} ").unwrap(); }
            LineTo(a) => { let (x, y) = t((a.x, a.y)); write!(s, "L{x:.4} {y:.4} ").unwrap(); }
            QuadTo(a, c) => { let (x, y) = t((a.x, a.y)); let (u, v) = t((c.x, c.y)); write!(s, "Q{x:.4} {y:.4} {u:.4} {v:.4} ").unwrap(); }
            CurveTo(a, c, e) => { let (x, y) = t((a.x, a.y)); let (u, v) = t((c.x, c.y)); let (m, n) = t((e.x, e.y)); write!(s, "C{x:.4} {y:.4} {u:.4} {v:.4} {m:.4} {n:.4} ").unwrap(); }
            ClosePath => s.push_str("Z "),
        }
    }
    s
}
mod kurbo_pt { pub type P = (f64, f64); }
fn main() {
    let out = std::env::args().nth(1).expect("dir");
    let star = cook(ShapeKind::Star, [-3.6, -1.4], [-0.8, 1.4], &[5.0, 0.45, 0.0]);
    std::fs::write(format!("{out}/star.d"), svg_d(&star)).unwrap();
    for (jn, j) in [("miter", LineJoin::Miter), ("round", LineJoin::Round), ("bevel", LineJoin::Bevel)] {
        for sign in [1.0, -1.0] {
            for i in 1..=100 {
                let d = sign * f64::from(i) * 0.04;
                let t0 = std::time::Instant::now();
                let r = ph2d_vec_boolean::offset_path(&star, d, j, OffsetSide::Outer);
                let us = t0.elapsed().as_micros();
                let ds: Vec<String> = r.iter().map(svg_d).collect();
                let fr = r.first().map(|p| format!("{:?}", p.fill_rule)).unwrap_or_default();
                println!("{jn}\t{}\t{}\t{us}\t{fr}\t{}", (d * 100.0).round(), r.len(), ds.join(" "));
            }
        }
    }
}
