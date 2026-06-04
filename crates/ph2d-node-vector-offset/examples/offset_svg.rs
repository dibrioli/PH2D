//! Visual smoke for `vector.offset`: offsets a star by several distances/joins
//! and writes one SVG grid you can open in a browser.
//!
//! Run: `cargo run -p ph2d-node-vector-offset --example offset_svg`
//! Output: `offset_ops.svg` in the current directory.
//!
//! Each cell draws the original star as a faint outline plus the offset result
//! filled — so you can see outward growth (round vs miter vs bevel corners) and
//! inward shrink.

use glam::Vec2;
use ph2d_node_vector_offset::offset;
use ph2d_vector_doc::primitives;
use ph2d_vector_kurbo::{Join, network_to_bezpath};
use std::fmt::Write as _;

const CELL: f64 = 200.0;
const COLS: usize = 3;

fn main() {
    // A 5-point star centered at (100,100): concave corners make the join style
    // and the inset visually obvious.
    let star = primitives::star(Vec2::new(100.0, 100.0), 70.0, 30.0, 5, 0.0);
    let star_svg = network_to_bezpath(&star).to_svg();

    let cases: [(&str, f32, Join); 5] = [
        ("original (offset 0)", 0.0, Join::Round),
        ("+15 Round", 15.0, Join::Round),
        ("+15 Miter", 15.0, Join::Miter),
        ("+15 Bevel", 15.0, Join::Bevel),
        ("-12 inset", -12.0, Join::Round),
    ];

    let rows = cases.len().div_ceil(COLS);
    let mut svg = String::new();
    let _ = writeln!(
        svg,
        "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='{}' \
         font-family='sans-serif'>",
        COLS as f64 * CELL,
        rows as f64 * CELL
    );
    let _ = writeln!(svg, "<rect width='100%' height='100%' fill='#fafafa'/>");

    for (i, (name, dist, join)) in cases.iter().enumerate() {
        let dx = (i % COLS) as f64 * CELL;
        let dy = (i / COLS) as f64 * CELL;
        let result = network_to_bezpath(&offset(&star, *dist, *join, 4.0)).to_svg();
        let _ = write!(svg, "<g transform='translate({dx},{dy})'>");
        let _ = write!(
            svg,
            "<path d='{star_svg}' fill='none' stroke='#ccc' stroke-width='1'/>"
        );
        let _ = write!(
            svg,
            "<path d='{result}' fill='#10b981' fill-opacity='0.5' \
             fill-rule='nonzero' stroke='#065f46' stroke-width='1.5'/>"
        );
        let _ = write!(
            svg,
            "<text x='10' y='{}' font-size='13' fill='#111'>{name}</text>",
            CELL - 12.0
        );
        let _ = write!(svg, "</g>");
    }
    let _ = writeln!(svg, "</svg>");

    std::fs::write("offset_ops.svg", &svg).expect("write offset_ops.svg");
    println!(
        "wrote offset_ops.svg ({} cases, {} bytes) — open it in a browser",
        cases.len(),
        svg.len()
    );
}
