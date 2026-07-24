//! `layers_no_alloc_hot_compose` (HR-3) — the layer-compositor op-flatten hot
//! path must not allocate once its scratch is warm: every frame it `clear`s and
//! refills the same [`GpuOpScratch`] buffers, reusing their capacity.
//!
//! We assert this by **scratch-capacity stability**, not by diffing dhat's
//! global `total_blocks` counter. That counter is process-wide and increments
//! on ANY allocation anywhere in the process — including test-harness and
//! background-thread traffic that lands in the measured wall-clock window
//! nondeterministically. That made the old gate flaky in CI: the *identical*
//! code passed one run and failed another with a handful of stray blocks
//! ("allocated 4 blocks over 200 iterations"), because the window happened to
//! catch unrelated allocations, not any from `flatten_layer_ops`.
//!
//! `flatten_layer_ops`'s only allocation sites are the `Vec`s inside
//! `GpuOpScratch` (pure `clear` + `push`; no `Box`/`format!`/temporaries — see
//! `layer_compositor::flatten_layer_ops`). So "the scratch capacity does not
//! change across a warm hot loop" is a complete AND deterministic proxy for
//! "zero allocation when warm", and it names the exact failure (a realloc) if a
//! regression ever reintroduces buffer growth in the hot path.

use ph2d_render::{GpuOpScratch, LayerOp, flatten_layer_ops};

#[test]
fn layers_no_alloc_hot_compose() {
    // A 50-op stack with a nested group — the worst-case flatten shape the
    // real-time compositor re-runs every frame.
    let mut ops = Vec::with_capacity(50);
    for i in 0..24u32 {
        ops.push(LayerOp::Layer {
            mask: None,
            clipping: false,
            key: (i % 8) as u64,
            blend_mode: (i % 22) as u8,
            opacity: 0.9,
        });
    }
    ops.push(LayerOp::PushGroup);
    for i in 0..23u32 {
        ops.push(LayerOp::Layer {
            mask: None,
            clipping: false,
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

    // Warm the scratch to its final capacity — the ONLY reallocs happen here,
    // before the measured window.
    flatten_layer_ops(&ops, slot_of, |k| Some(slot_of(k)), &mut scratch);
    let warm_cap = scratch.capacity();
    assert!(
        warm_cap >= ops.len(),
        "warm-up under-sized the scratch (cap {warm_cap} < {} ops)",
        ops.len()
    );

    // HR-3: 200 warm flattens must reuse the scratch without a single realloc.
    for _ in 0..200 {
        flatten_layer_ops(&ops, slot_of, |k| Some(slot_of(k)), &mut scratch);
    }

    assert_eq!(
        scratch.capacity(),
        warm_cap,
        "HR-3 violation: warm flatten reallocated its scratch (cap {warm_cap} → {})",
        scratch.capacity(),
    );
    assert_eq!(scratch.len(), ops.len());
}
