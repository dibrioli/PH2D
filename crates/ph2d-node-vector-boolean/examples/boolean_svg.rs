//! Visual smoke for `vector.boolean`: runs every op on two overlapping shapes
//! (a rect `a` and an ellipse `b`) and writes one SVG grid you can open in a
//! browser to eyeball the geometry.
//!
//! Run: `cargo run -p ph2d-node-vector-boolean --example boolean_svg`
//! Output: `boolean_ops.svg` in the current directory.
//!
//! Each cell draws `a` and `b` as faint outlines plus the boolean result filled
//! in blue — so you can see, e.g., that Subtract removes the ellipse from the
//! rect and Intersect keeps only the lens.

use glam::Vec2;
use ph2d_node_vector_boolean::boolean;
use ph2d_vector_doc::{BooleanOp, primitives};
use ph2d_vector_kurbo::network_to_bezpath;
use std::fmt::Write as _;

const CELL: f64 = 200.0;
const COLS: usize = 3;

fn main() {
    // Two overlapping shapes centered around (90, 90).
    let a = primitives::rect(Vec2::new(20.0, 20.0), Vec2::new(130.0, 130.0));
    let b = primitives::ellipse(Vec2::new(130.0, 130.0), Vec2::new(65.0, 65.0));

    let ops = [
        ("Union", BooleanOp::Union),
        ("Subtract", BooleanOp::Subtract),
        ("Intersect", BooleanOp::Intersect),
        ("Exclude", BooleanOp::Exclude),
        ("Divide", BooleanOp::Divide),
        ("Trim", BooleanOp::Trim),
        ("Merge", BooleanOp::Merge),
        ("Crop", BooleanOp::Crop),
        ("Outline", BooleanOp::Outline),
    ];

    let a_svg = network_to_bezpath(&a).to_svg();
    let b_svg = network_to_bezpath(&b).to_svg();
    let rows = ops.len().div_ceil(COLS);

    let mut svg = String::new();
    let _ = writeln!(
        svg,
        "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='{}' \
         font-family='sans-serif'>",
        COLS as f64 * CELL,
        rows as f64 * CELL
    );
    let _ = writeln!(svg, "<rect width='100%' height='100%' fill='#fafafa'/>");

    for (i, (name, op)) in ops.iter().enumerate() {
        let dx = (i % COLS) as f64 * CELL;
        let dy = (i / COLS) as f64 * CELL;
        let result = network_to_bezpath(&boolean(&a, &b, *op)).to_svg();
        let _ = write!(svg, "<g transform='translate({dx},{dy})'>");
        // Faint operands for reference.
        let _ = write!(
            svg,
            "<path d='{a_svg}' fill='none' stroke='#ccc' stroke-width='1'/>\
             <path d='{b_svg}' fill='none' stroke='#ccc' stroke-width='1'/>"
        );
        // The boolean result, filled (non-zero) + outlined.
        let _ = write!(
            svg,
            "<path d='{result}' fill='#3b82f6' fill-opacity='0.55' \
             fill-rule='nonzero' stroke='#1e3a8a' stroke-width='1.5'/>"
        );
        let _ = write!(
            svg,
            "<text x='10' y='{}' font-size='14' fill='#111'>{name}</text>",
            CELL - 12.0
        );
        let _ = write!(svg, "</g>");
    }
    let _ = writeln!(svg, "</svg>");

    std::fs::write("boolean_ops.svg", &svg).expect("write boolean_ops.svg");
    println!(
        "wrote boolean_ops.svg ({} ops, {} bytes) — open it in a browser",
        ops.len(),
        svg.len()
    );
}
