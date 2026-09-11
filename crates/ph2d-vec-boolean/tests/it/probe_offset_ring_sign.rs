//! SONDA (`--ignored`): **o `offset_ring` encolhe com `d < 0` ou cresce?**
//!
//! O Enio reportou: `Side:Outer` com offset POSITIVO cresce (certo), mas com offset NEGATIVO
//! *"fica como na foto"* (cresce também). A hipótese: numa forma CÔNCAVA (estrela), o laço interno
//! (erosão) do `kurbo::stroke` se auto-intersecta, e a `area().abs()` (shoelace) mente sobre qual
//! laço é a dilatação e qual é a erosão — a MESMA lição do gate de paridade. Meça antes de corrigir.
//!
//! ⚠️ **A régua é a área NonZero** (o que o Vello preenche), NÃO o bbox das âncoras: os spikes de
//! Miter da erosão estendem os verts ao tamanho da fonte mesmo com a região preenchida menor — o
//! bbox mentiria (mediu 1º, e reportou "cresceu" sobre uma erosão correta).

use kurbo::{BezPath, Point, Shape};
use ph2d_vec_boolean::offset_ring;
use ph2d_vec_scene::{LineJoin, OffsetSide, VecPath, VecVertex};

fn poly(pts: &[[f64; 2]]) -> Vec<VecVertex> {
    pts.iter().copied().map(VecVertex::corner).collect()
}

fn closed(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

fn star() -> VecPath {
    let mut pts = Vec::new();
    for i in 0..10 {
        let a = std::f64::consts::PI * 2.0 * f64::from(i) / 10.0 - std::f64::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { 1.0 } else { 0.42 };
        pts.push([a.cos() * r, a.sin() * r]);
    }
    closed(poly(&pts))
}

fn hexagon() -> VecPath {
    let mut pts = Vec::new();
    for i in 0..6 {
        let a = std::f64::consts::PI * 2.0 * f64::from(i) / 6.0;
        pts.push([a.cos(), a.sin()]);
    }
    closed(poly(&pts))
}

/// A área que o rasterizador de fato preenche (winding NonZero != 0), amostrada numa grade fina.
fn nonzero_area(paths: &[VecPath]) -> f64 {
    let mut bez = BezPath::new();
    for path in paths {
        let n = path.verts.len();
        if n < 3 {
            continue;
        }
        bez.move_to(Point::new(path.verts[0].anchor[0], path.verts[0].anchor[1]));
        for i in 0..n {
            let a = &path.verts[i];
            let b = &path.verts[(i + 1) % n];
            bez.curve_to(
                Point::new(a.out_handle[0], a.out_handle[1]),
                Point::new(b.in_handle[0], b.in_handle[1]),
                Point::new(b.anchor[0], b.anchor[1]),
            );
        }
        bez.close_path();
    }
    if bez.elements().len() < 3 {
        return 0.0;
    }
    let bb = bez.bounding_box();
    let inside: u64 = (0..500u32)
        .flat_map(|iy| (0..500u32).map(move |ix| (ix, iy)))
        .filter(|&(ix, iy)| {
            let x = bb.x0 + (bb.x1 - bb.x0) * (f64::from(ix) + 0.5) / 500.0;
            let y = bb.y0 + (bb.y1 - bb.y0) * (f64::from(iy) + 0.5) / 500.0;
            bez.winding(Point::new(x, y)) != 0
        })
        .count() as u64;
    #[allow(clippy::cast_precision_loss)]
    let a = inside as f64 * (bb.x1 - bb.x0) * (bb.y1 - bb.y0) / 250_000.0;
    a
}

fn unit_square() -> VecPath {
    closed(poly(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]))
}

#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn probe_square_erosion() {
    let sq = unit_square();
    let src = nonzero_area(std::slice::from_ref(&sq));
    for join in [("Round", LineJoin::Round), ("Miter", LineJoin::Miter)] {
        for &d in &[-0.1_f64, -0.2, -0.3] {
            let out = offset_ring(&sq, d, join.1, OffsetSide::Outer);
            let a = out.as_deref().map_or(f64::NAN, nonzero_area);
            let n = out.as_deref().map_or(0, <[VecPath]>::len);
            println!(
                "quadrado {:5} d={d:+.2}: area={a:.4} (fonte {src:.4}) caminhos={n}",
                join.0
            );
        }
    }
}

#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn probe_offset_ring_sign() {
    for (name, shape) in [("hexagono", hexagon()), ("estrela", star())] {
        let src = nonzero_area(std::slice::from_ref(&shape));
        for join in [("Miter", LineJoin::Miter), ("Round", LineJoin::Round)] {
            for &d in &[0.15_f64, -0.15] {
                let out = offset_ring(&shape, d, join.1, OffsetSide::Outer);
                let a = out.as_deref().map_or(f64::NAN, nonzero_area);
                let verdict = if a.is_nan() {
                    "None (abstem)".to_string()
                } else if a > src * 1.02 {
                    format!("CRESCEU  {a:.3} > {src:.3}")
                } else if a < src * 0.98 {
                    format!("encolheu {a:.3} < {src:.3}")
                } else {
                    format!("~igual   {a:.3}")
                };
                let want = if d > 0.0 { "CRESCER" } else { "encolher" };
                println!("{name:9} {:5} d={d:+.2} (quer {want:8}): {verdict}", join.0);
            }
        }
    }
}
