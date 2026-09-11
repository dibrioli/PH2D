//! **The GPU-resident extra draw binds a texture PER RUN** (this wave).
//!
//! The `gpu_extra` buffer is a `source.object` graph's device instance buffer,
//! each instance carrying its object's `texture_id` in word 41. The sprite
//! shader never reads that word (texture selection is a per-draw CPU bind), so
//! the cook hands the renderer a run partition (`&[GpuTexRun]`) and this draw
//! MUST loop over it, binding the object's texture through the same `material_bg`
//! door the scene runs use. Reverting to the pre-wave single
//! `material_bind_group` draw over the whole buffer would paint every object as
//! the shared atlas — the white-quads bug — and no unit test of the cook would
//! see it (the cook is correct; only the render regressed).
//!
//! An EMPTY partition is still the legacy single atlas draw (a non-object
//! stream, byte-identical) — so this gate asserts the per-run bind lives in the
//! `runs.is_empty()`-ELSE branch, not that the atlas draw is gone.

use std::fs;

#[test]
fn the_gpu_extra_draw_loops_the_runs_binding_material_bg() {
    let src = fs::read_to_string("src/renderer_draw.rs").expect("renderer_draw.rs");

    // The `gpu_extra` block of the normal pass.
    let block = src
        .split_once("if let Some((buffer, n, runs)) = gpu_extra")
        .expect("the gpu_extra draw destructures (buffer, n, runs) — the run list was dropped")
        .1;
    // Bound it to the block that follows (up to the clip pass), so a match in a
    // later pass can't stand in for it.
    let block = block
        .split_once("// Clip pass")
        .map(|(b, _)| b)
        .unwrap_or(block);

    // The empty-partition fallback: the legacy single atlas draw, byte-identical.
    assert!(
        block.contains("if runs.is_empty()"),
        "the gpu_extra draw must keep the empty-partition atlas fallback (non-object graphs)"
    );

    // The object path: one draw per run, binding the object's texture through
    // the shared `material_bg` door. This is the line a regression to the single
    // atlas draw removes.
    assert!(
        block.contains("for r in runs"),
        "the gpu_extra draw must LOOP over the texture runs (else objects paint as atlas)"
    );
    assert!(
        block.contains("material_bg(r.texture_id"),
        "each run must bind its object's texture via material_bg(r.texture_id, ..)"
    );
    let per_run = block.find("for r in runs").expect("checked above");
    let bind = block
        .find("material_bg(r.texture_id")
        .expect("checked above");
    let draw = block[per_run..]
        .find("r.start..r.end")
        .expect("each run must draw its own [start, end) slice, not the whole buffer");
    assert!(
        bind > per_run && per_run + draw > bind,
        "the per-run bind and its slice draw must live INSIDE the run loop"
    );
}
