//! **ADR-0152 W0, gate #3: a formula-free apply builds no expression machinery.**
//!
//! W0 threads a [`LinkFrame`](../src/frame_solve.rs) through the blend so W1 can make
//! an expression a lane source — but on the common path (no global `binding.expr`, no
//! per-clip `clip.expr`) the frame is EMPTY and the `!scheduled` fork skips the whole
//! post-pass: no snapshot, no `Name` map, no topo-sort. An empty `BTreeMap` never
//! allocates, so threading it costs nothing (HR-3).
//!
//! The gate measures the CONTRAST inside one profiler: a formula-free document's
//! steady-state apply must allocate ZERO blocks, while the SAME document with one
//! expression added must allocate MORE than zero (the scheduler builds its snapshot +
//! topo every frame — caching was measured and rejected, `expr_pass.rs`). Without the
//! second half, the zero could be met by a mutation that makes nothing run at all;
//! with it, a mutation that builds a LinkFrame on the hot path (Phase A > 0) and a
//! mutation that never schedules the expression (Phase B == 0) are both caught.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    ClipLane, ClipStrip, LaneMode, PropKind, StripSource, TimelineDoc, apply_from_doc,
};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// A non-trivial bound document with a real clip STACK (two lanes, a crossfade
/// overlap, and an additive lane) — the hot path's most expensive shape, so a
/// per-frame allocation cannot hide. Formula-free: no binding carries `expr` and no
/// clip carries a per-clip expression. Returns the entities so the caller can stamp
/// an expression on one for Phase B.
fn stacked_doc() -> (World, TimelineDoc, Vec<Entity>) {
    let mut w = World::new();
    let mut doc = TimelineDoc::new();
    let s = RationalTime::from_seconds;
    let mut ents = Vec::new();
    for i in 0..6 {
        let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        ents.push(e);
        doc.insert_key(
            e.to_bits(),
            PropKind::TranslationX,
            s(0.0),
            AnimValue::Float(0.0),
            Interp::Linear,
        );
        doc.insert_key(
            e.to_bits(),
            PropKind::TranslationX,
            s(1.0),
            AnimValue::Float(i as f32),
            Interp::Hold,
        );
    }
    doc.add_clip("B".to_string());
    for (i, e) in ents.iter().enumerate() {
        doc.set_active(1);
        doc.insert_key(
            e.to_bits(),
            PropKind::TranslationX,
            s(0.0),
            AnimValue::Float(i as f32 * 2.0),
            Interp::Linear,
        );
        doc.set_active(0);
    }
    let mut base = ClipLane::new("Base");
    base.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 1.0));
    base.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 1.0)); // overlaps -> crossfade
    let mut add = ClipLane::new("Add");
    add.mode = LaneMode::Additive;
    add.insert(ClipStrip::new(StripSource::Clip(1), 0.0, 3.0, 1.0));
    doc.stack_mut().push(base);
    doc.stack_mut().push(add);
    (w, doc, ents)
}

/// Steady-state allocation of `count` applies over a document already warmed once.
fn steady_state_blocks(w: &mut World, doc: &mut TimelineDoc, count: usize) -> u64 {
    for i in 0..16 {
        apply_from_doc(w, doc, f64::from(i) / 16.0);
    }
    let warm = dhat::HeapStats::get();
    for i in 0..count {
        apply_from_doc(w, doc, (i % 100) as f64 / 99.0);
    }
    let steady = dhat::HeapStats::get();
    steady.total_blocks - warm.total_blocks
}

#[test]
fn no_expression_allocates_no_link_frame() {
    let _profiler = dhat::Profiler::builder().testing().build();

    // Phase A — formula-free. The `!scheduled` fork runs the keyed pass and returns;
    // the LinkFrame it threaded stays empty (zero-alloc), the post-pass never runs.
    let (mut wa, mut da, _) = stacked_doc();
    let free_blocks = steady_state_blocks(&mut wa, &mut da, 500);

    // Phase B — the same shape, one channel now driven by an expression. `scheduled`
    // is true, so the scheduler builds its snapshot + topo every frame and allocates.
    let (mut wb, mut db, ents) = stacked_doc();
    let tgt = db.bind(ents[0].to_bits(), PropKind::TranslationY);
    db.bindings_mut()
        .iter_mut()
        .find(|b| b.target == tgt)
        .expect("just bound")
        .expr = Some("value + time".to_string());
    let expr_blocks = steady_state_blocks(&mut wb, &mut db, 500);

    assert_eq!(
        free_blocks, 0,
        "a formula-free apply allocated {free_blocks} blocks — W0 must thread an EMPTY \
         LinkFrame and take the !scheduled fork (no snapshot, no topo-sort)"
    );
    assert!(
        expr_blocks > 0,
        "the with-expression apply allocated nothing ({expr_blocks} blocks) — the zero \
         above would then prove nothing ran, not that no LinkFrame was built"
    );
}
