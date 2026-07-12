//! The apply's cost SHAPE — the baseline the clip stack is measured against
//! (ADR-0115 §4, kill-criterion).
//!
//! Ignored by default: a wall-clock probe is the wrong thing to gate CI on (it
//! is machine- and load-dependent, and a flaky red gate teaches people to ignore
//! gates). It exists to be **run on demand**, and to be re-run when the stack
//! lands, so "> 2x the baseline" is a measurement and not a vibe:
//!
//! ```text
//! cargo test -p ph2d-timeline --release --all-features -- --ignored --nocapture
//! ```
//!
//! What it pins: the apply used to resolve each entity's Time Remap clock once
//! per BINDING, scanning every binding to do it — quadratic. `clock.rs` hoisted
//! it to once per ENTITY. The two shapes are both measured here, in one binary,
//! so the comparison is apples to apples.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Transform, World};
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc, remapped_time};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// `entities` sprites, each animating all six scene props AND carrying a Time
/// Remap track — the **worst case** for the clock (every entity has one).
fn scene(entities: usize) -> (World, TimelineDoc) {
    let mut world = World::new();
    let mut doc = TimelineDoc::new();
    for _ in 0..entities {
        let e = world.spawn(Transform::default()).id().to_bits();
        for prop in PropKind::ALL {
            doc.insert_key(e, prop, s(0.0), AnimValue::Float(0.0), Interp::Linear);
            doc.insert_key(e, prop, s(4.0), AnimValue::Float(10.0), Interp::Linear);
        }
        let tr = PropKind::TimeRemap;
        doc.insert_key(e, tr, s(0.0), AnimValue::Float(0.0), Interp::Linear);
        doc.insert_key(e, tr, s(4.0), AnimValue::Float(2.0), Interp::Linear);
    }
    (world, doc)
}

/// Wall-clock for `frames` applies, in microseconds per frame.
fn per_frame_us(entities: usize, frames: u32) -> f64 {
    let (mut world, mut doc) = scene(entities);
    apply_from_doc(&mut world, &mut doc, 0.0); // warm the buffers
    let start = std::time::Instant::now();
    for f in 0..frames {
        apply_from_doc(&mut world, &mut doc, f64::from(f) * 0.01);
    }
    start.elapsed().as_secs_f64() * 1e6 / f64::from(frames)
}

/// The OLD shape, reproduced: resolve the clock inside the per-binding loop.
/// This is what `apply_from_doc` did before `clock.rs`, and it is here only so
/// the hoist can be measured against something real.
fn per_frame_us_quadratic(entities: usize, frames: u32) -> f64 {
    let (_world, doc) = scene(entities);
    let start = std::time::Instant::now();
    let mut sink = 0.0_f64;
    for f in 0..frames {
        let t = f64::from(f) * 0.01;
        for b in doc.bindings() {
            sink += remapped_time(&doc, b.entity, t); // the per-BINDING scan
        }
    }
    assert!(sink.is_finite());
    start.elapsed().as_secs_f64() * 1e6 / f64::from(frames)
}

/// The hoist, measured — and measured on the metric that reveals the SHAPE.
///
/// Total wall-clock ratios are a poor test: 4x the data costs 4x under a linear
/// law and ~4.8x under `B log B`, and machine noise spans that gap. The **cost
/// per binding** does not have that problem. It is flat under a linear law, it
/// creeps under `B log B` (the binary searches), and under the quadratic law it
/// grows *in proportion to B* — 4x the bindings, 4x the cost of each one. That
/// is unmissable, and it is what regressing would look like.
///
/// Measured on the dev workstation, 2026-07-12 (release, every entity remapped —
/// the worst case for the clock):
///
/// ```text
///   entities  bindings   us/apply   us/binding
///         25       175       4.44        0.025
///        200      1400      51.84        0.037
///        800      5600     299.92        0.054
/// ```
///
/// 32x the data, 2.2x the per-binding cost: linearithmic. Before the hoist this
/// column tracked B outright. 5600 bindings cost 1.8% of a 60 Hz frame.
#[test]
#[ignore = "wall-clock probe; run on demand (see module docs)"]
fn the_apply_no_longer_scales_quadratically_in_bindings() {
    let (small, large) = (50, 200); // 4x the entities => 4x the bindings
    let (bind_s, bind_l) = (small * 7, large * 7); // six props + a Time Remap

    let hoisted_s = per_frame_us(small, 2_000);
    let hoisted_l = per_frame_us(large, 500);
    let old_s = per_frame_us_quadratic(small, 2_000);
    let old_l = per_frame_us_quadratic(large, 500);

    let (per_s, per_l) = (hoisted_s / bind_s as f64, hoisted_l / bind_l as f64);
    let (old_per_s, old_per_l) = (old_s / bind_s as f64, old_l / bind_l as f64);
    let (growth, old_growth) = (per_l / per_s, old_per_l / old_per_s);

    println!("   bindings |  us/apply | us/binding   (hoisted)");
    println!("  {bind_s:>9} | {hoisted_s:>9.1} | {per_s:>10.3}");
    println!("  {bind_l:>9} | {hoisted_l:>9.1} | {per_l:>10.3}");
    println!("  4x data   -> per-binding cost x{growth:.2} (was x{old_growth:.2})");

    assert!(
        growth < 2.5,
        "4x the bindings made each one {growth:.2}x dearer — the quadratic shape is back \
         (it would be ~4x; a binary-search law is ~1.3x)"
    );
    assert!(
        growth < old_growth,
        "the hoist must scale better than the per-binding scan it replaced"
    );
}
