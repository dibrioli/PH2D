//! SONDA (`--ignored`): **que laços o `kurbo::stroke` emite para um hexágono/estrela?**
//!
//! O `offset_ring` com `d<0` e quina Miter devolvia VAZIO (deveria devolver a erosão). A hipótese:
//! a Miter cria um SLIVER de auto-interseção que sobrevive ao filtro, e "menor bbox" o escolhe em
//! vez da erosão real. Dumpa (área ASSINADA, bbox) de cada laço para ver o discriminador certo.

use kurbo::{BezPath, Cap, Join, PathEl, Point, Shape, Stroke, StrokeOpts};

fn poly_bez(pts: &[[f64; 2]]) -> BezPath {
    let mut b = BezPath::new();
    b.move_to(Point::new(pts[0][0], pts[0][1]));
    for p in &pts[1..] {
        b.line_to(Point::new(p[0], p[1]));
    }
    b.close_path();
    b
}

fn hexagon() -> BezPath {
    let pts: Vec<[f64; 2]> = (0..6)
        .map(|i| {
            let a = std::f64::consts::PI * 2.0 * f64::from(i) / 6.0;
            [a.cos(), a.sin()]
        })
        .collect();
    poly_bez(&pts)
}

fn star() -> BezPath {
    let pts: Vec<[f64; 2]> = (0..10)
        .map(|i| {
            let a = std::f64::consts::PI * 2.0 * f64::from(i) / 10.0 - std::f64::consts::FRAC_PI_2;
            let r = if i % 2 == 0 { 1.0 } else { 0.42 };
            [a.cos() * r, a.sin() * r]
        })
        .collect();
    poly_bez(&pts)
}

fn split_loops(bez: &BezPath) -> Vec<BezPath> {
    let mut out = Vec::new();
    let mut cur = BezPath::new();
    for el in bez.elements() {
        if matches!(el, PathEl::MoveTo(_)) && !cur.elements().is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(*el);
    }
    if !cur.elements().is_empty() {
        out.push(cur);
    }
    out
}

fn unit_square() -> BezPath {
    poly_bez(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
}

#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn probe_square_loops() {
    let shape = unit_square();
    println!("quadrado: area_fonte={:+.4}", shape.area());
    for &d in &[0.1_f64, 0.2, 0.3] {
        let pen = Stroke::new(2.0 * d)
            .with_join(Join::Round)
            .with_caps(Cap::Butt);
        let traced = kurbo::stroke(&shape, &pen, &StrokeOpts::default(), 1e-4);
        let loops = split_loops(&traced);
        println!("  d={d:.2} Round ({} laços):", loops.len());
        for (i, l) in loops.iter().enumerate() {
            let b = l.bounding_box();
            println!(
                "    [{i}] area={:+.4} bbox_diag={:.4}",
                l.area(),
                b.width().hypot(b.height())
            );
        }
    }
}

#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn probe_ring_loops() {
    for (name, shape) in [("hexagono", hexagon()), ("estrela", star())] {
        let src_a = shape.area();
        let sb = shape.bounding_box();
        println!(
            "\n== {name}: area_assinada={src_a:+.4} bbox_diag={:.4} ==",
            sb.width().hypot(sb.height())
        );
        for join in [("Miter", Join::Miter), ("Round", Join::Round)] {
            for &d in &[0.15_f64] {
                let pen = Stroke::new(2.0 * d).with_join(join.1).with_caps(Cap::Butt);
                let traced = kurbo::stroke(&shape, &pen, &StrokeOpts::default(), 1e-4);
                let loops = split_loops(&traced);
                println!("  {} ({} laços):", join.0, loops.len());
                for (i, l) in loops.iter().enumerate() {
                    let b = l.bounding_box();
                    println!(
                        "    [{i}] area_assinada={:+.4} bbox_diag={:.4} verts~{}",
                        l.area(),
                        b.width().hypot(b.height()),
                        l.elements().len()
                    );
                }
            }
        }
    }
}
