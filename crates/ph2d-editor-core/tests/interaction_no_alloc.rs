//! HR-3 enforcement bench for the editor input pipeline (ADR-0024).
//!
//! Asserts that the hot path — pre-populated [`WidgetStore`] +
//! per-frame [`HitIndex`] rebuild + 5 [`PointerEvent`]s/frame
//! dispatched through `dispatch_pointer` — performs **zero heap
//! allocations** across 10 simulated frames with 30 widgets.
//!
//! Allocations are counted via `dhat-rs` (`#[global_allocator]`
//! installed for this test binary only). The window of measurement
//! is bracketed: setup runs before profiling starts; only the hot
//! loop is observed.
//!
//! Failure here means a hot-path allocation slipped in (BTreeMap
//! insert, Vec::push past capacity, String allocation, etc.). When
//! that happens: STOP, root-cause, fix before adding more widgets.

use bumpalo::Bump;
use dhat::{HeapStats, Profiler};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore, dispatch_pointer};
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerEvent, PointerKind, PointerSource};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const WIDGETS: u64 = 30;
const FRAMES: usize = 10;
const EVENTS_PER_FRAME: usize = 5;

fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn populate_store() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(64);
    for i in 0..WIDGETS {
        store.register(
            NodeId(i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store
}

fn populate_hit_index() -> HitIndex {
    let mut hits = HitIndex::new();
    for i in 0..WIDGETS {
        hits.register(NodeId(i), Rect::new(i as f32 * 32.0, 0.0, 32.0, 32.0));
    }
    hits
}

#[test]
fn interaction_dispatch_no_alloc() {
    // Phase 1: warm everything BEFORE profiling. Allocations here
    // are construction-time and do not count against HR-3.
    let mut store = populate_store();
    let mut hits = populate_hit_index();
    let mut arena = Bump::with_capacity(4096);
    // Touch arena once so it has a live chunk before measurement.
    {
        let bump_warm: bumpalo::collections::Vec<u8> = bumpalo::collections::Vec::new_in(&arena);
        let _ = bump_warm;
    }

    // Phase 2: open the dhat profiler window. From here on every
    // heap allocation is counted.
    let profiler = Profiler::builder().testing().build();
    let stats_before = HeapStats::get();

    for frame in 0..FRAMES {
        // Mimic paint pass: clear + re-register hit rects.
        hits.clear_for_frame();
        for i in 0..WIDGETS {
            hits.register(NodeId(i), Rect::new(i as f32 * 32.0, 0.0, 32.0, 32.0));
        }
        // Dispatch 5 pointer events sweeping across the row.
        for ev in 0..EVENTS_PER_FRAME {
            let x = ((frame * EVENTS_PER_FRAME + ev) as f32 * 13.0) % (WIDGETS as f32 * 32.0);
            let _ = dispatch_pointer(
                &mut store,
                &hits,
                pointer(PointerKind::Move, x, 16.0),
                &arena,
            );
        }
        // End-of-frame: caller resets arena. This deallocates the
        // bumpalo chunks if any were used, but Bump::reset itself
        // does NOT call the global allocator (only resets pointers
        // within the existing chunk). Verified empirically.
        arena.reset();
    }

    let stats_after = HeapStats::get();
    drop(profiler);

    let blocks = stats_after.total_blocks - stats_before.total_blocks;
    let bytes = stats_after.total_bytes - stats_before.total_bytes;

    // Hard zero: every hot-path allocation budget = 0. Setup phase
    // happened before the profiler started so its allocs aren't
    // counted here.
    assert_eq!(
        blocks, 0,
        "HR-3 violation: hot path allocated {} blocks ({} bytes total) across {} frames. \
         Inspect dispatch_pointer / HitIndex / WidgetStore for the offender.",
        blocks, bytes, FRAMES
    );
}
