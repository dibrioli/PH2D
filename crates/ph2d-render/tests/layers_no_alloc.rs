//! `layers_no_alloc_hot_compose` (HR-3) — the layer-compositor op-flatten hot
//! path must not allocate once its scratch is warm. Mirrors
//! `ph2d-painter-brush`'s `painter_no_alloc_hot_path`: a dedicated test binary
//! with the dhat global allocator (gated off miri, which hooks the allocator
//! too). CPU-only — no GPU device needed, so this runs in the normal CI lane.

#![cfg(not(miri))]

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use ph2d_render::{GpuOpScratch, LayerOp, flatten_layer_ops};

#[test]
fn layers_no_alloc_hot_compose() {
    let _profiler = dhat::Profiler::builder().testing().build();

    // A 50-op stack with a nested group — the worst-case flatten shape the
    // real-time compositor re-runs every frame.
    let mut ops = Vec::with_capacity(50);
    for i in 0..24u32 {
        ops.push(LayerOp::Layer {
            key: (i % 8) as u64,
            blend_mode: (i % 22) as u8,
            opacity: 0.9,
        });
    }
    ops.push(LayerOp::PushGroup);
    for i in 0..23u32 {
        ops.push(LayerOp::Layer {
            key: (i % 8) as u64,
            blend_mode: (i % 22) as u8,
            opacity: 0.5,
        });
    }
    ops.push(LayerOp::PopGroup {
        blend_mode: 9,
        opacity: 0.6,
    });

    let slot_of = |k: u64| (k % 8) as u32;
    let mut scratch = GpuOpScratch::new();

    // Warm the scratch (one realloc to final capacity, outside the window).
    flatten_layer_ops(&ops, slot_of, &mut scratch);

    let before = dhat::HeapStats::get();
    for _ in 0..200 {
        flatten_layer_ops(&ops, slot_of, &mut scratch);
    }
    let after = dhat::HeapStats::get();

    assert_eq!(
        after.total_blocks - before.total_blocks,
        0,
        "HR-3 violation: warm flatten allocated {} blocks over 200 iterations",
        after.total_blocks - before.total_blocks,
    );
    assert_eq!(scratch.len(), ops.len());
}
