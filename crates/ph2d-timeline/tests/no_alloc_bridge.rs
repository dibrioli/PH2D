//! W1.T9 / HR-3: the per-frame apply pass ([`apply_from_doc`]) over a non-empty,
//! bound document must not allocate in steady state — the property write it does
//! each frame (the "playing" and "paused" paths both run it) is a table lookup +
//! a zero-alloc sample, no growable buffer.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Transform, World};
use ph2d_timeline::{ClipLane, ClipStrip, LaneMode, PropKind, TimelineDoc, apply_from_doc};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn apply_from_doc_is_zero_alloc_steady_state() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let mut w = World::new();
    // A handful of bound sprites, each with a keyed track.
    let mut doc = TimelineDoc::new();
    let s = RationalTime::from_seconds;
    let mut ents = Vec::new();
    for i in 0..8 {
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
        // Half of them carry a Time Remap track too, so the per-entity clock
        // lookup (`remapped_time`'s linear scan + sample) is under the gate.
        if i % 2 == 0 {
            doc.insert_key(
                e.to_bits(),
                PropKind::TimeRemap,
                s(0.0),
                AnimValue::Float(0.0),
                Interp::Linear,
            );
            doc.insert_key(
                e.to_bits(),
                PropKind::TimeRemap,
                s(1.0),
                AnimValue::Float(0.5),
                Interp::Linear,
            );
        }
    }

    // A real clip STACK on top (ADR-0115): two lanes, an overlap that crossfades,
    // and an additive lane. The stack is the hot path's most expensive shape, so
    // it is the one that must be under the gate — an evaluator that allocates a
    // buffer per frame would be invisible to a single-clip test and fatal here.
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
    base.insert(ClipStrip::new(0, 0.0, 2.0, 1.0));
    base.insert(ClipStrip::new(1, 1.0, 3.0, 1.0)); // overlaps -> crossfade
    let mut add = ClipLane::new("Add");
    add.mode = LaneMode::Additive;
    add.insert(ClipStrip::new(1, 0.0, 3.0, 1.0));
    doc.stack_mut().push(base);
    doc.stack_mut().push(add);

    std::hint::black_box(&ents);

    // Warm-up: first apply builds bevy's query/archetype state.
    for i in 0..16 {
        apply_from_doc(&mut w, &mut doc, f64::from(i) / 16.0);
    }
    let warm = dhat::HeapStats::get();

    // Steady-state sweep (advancing + a scrub back), the real per-frame path.
    for i in 0..2000 {
        apply_from_doc(&mut w, &mut doc, f64::from(i % 100) / 99.0);
    }
    let steady = dhat::HeapStats::get();

    let d_blocks = steady.total_blocks - warm.total_blocks;
    let d_bytes = steady.total_bytes - warm.total_bytes;
    assert_eq!(
        d_blocks, 0,
        "apply_from_doc allocated {d_blocks} blocks ({d_bytes} bytes) — must be zero-alloc (HR-3)"
    );
}
