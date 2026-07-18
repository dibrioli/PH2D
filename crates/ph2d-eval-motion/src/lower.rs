//! **Stream → instances** — the CPU lowering, and the one-shot cook around it.
//!
//! Split from `lib.rs` at the HR-18 LOC cap, along the seam that was already
//! there: this file answers *"what does a cooked stream look like on screen?"*
//! and knows nothing about ticks, memos or the `pre` feedback; `lib.rs` runs the
//! CLOCK (`MotionCookPump`) and calls in here to consume a result.
//!
//! The GPU path has the same split for the same reason — `ph2d-gpu-cook`'s
//! lowering is its own module and its own compute pass, so the two lowerings
//! stay comparable side by side (that comparison is the parity gate).

use crate::{Column, Graph, NodeId, OpResolver, PAR_THRESHOLD, RenderInstance, Stream};
use ph2d_nodegraph::cook::{Cook, CookError};
use rayon::prelude::*;

/// Lower a cooked instance stream **into `out`** (one instance per element),
/// reusing `out`'s capacity: `out` is cleared and refilled, so a steady stream
/// count frame-to-frame allocates nothing (M0.T11 — the per-frame bridge path;
/// zero-alloc gated by M0.T12). Pure + headless: no GPU.
///
/// `default_uv_rect` / `default_size` are the `atlas_uv` / `size` for an
/// instance whose stream lacks the matching column (the M0 case — no framing
/// node yet). The shell passes a single opaque atlas tile plus a `size` below
/// the grid spacing so the raw default document renders as clean, distinct
/// quads; a headless caller passes the whole-atlas rect `[0,0,1,1]` and unit
/// size `[1,1]`.
pub fn lower_to_instances_into(
    stream: &Stream,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) {
    out.clear();
    lower_to_instances_onto(stream, default_uv_rect, default_size, out);
}

/// Like [`lower_to_instances_into`] but **appends** — `out` keeps whatever it
/// already holds. This is how several render sinks compose into one draw: the
/// pump clears once, then each `motion.output` node's stream lowers onto the
/// same buffer (still zero-alloc in steady state — capacity is retained).
pub fn lower_to_instances_onto(
    stream: &Stream,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) {
    let n = stream.count();
    let p = stream.get("P");
    let size = stream.get("size");
    let rot = stream.get("rot");
    let tint = stream.get("tint");
    let uv_rect = stream.get("uv_rect");
    out.reserve(n);
    // Each instance is a pure function of its own index (a five-column gather +
    // one `sin_cos`); no cross-element dependency. Above the threshold
    // `par_extend` spreads it across cores, order-preserving → byte-identical to
    // the serial extend, so the render is unchanged. GPU/M5 Fase 0.
    let make = |i: usize| -> RenderInstance {
        // ADR-0070-amendment-4: RenderInstance carries the 2×2 world
        // basis, not a rotation scalar. A Motion stream emits only a
        // rotation (no skew), so the basis is a pure rotation matrix
        // `[cos, sin, -sin, cos]`. RenderInstance is PresentWorld-only
        // (HR-5 exempt), so std `sin_cos` is fine here.
        //
        // The `rot` column is in **degrees** — the app's authored-angle unit
        // (the Painter's `*_angle_deg` fields, the Inspector's `deg` boxes).
        // Radians live nowhere in the Motion authoring surface; only this
        // conversion, at the very edge where the basis is built.
        let (sin_r, cos_r) = scalar_at(rot, i, 0.0).to_radians().sin_cos();
        RenderInstance {
            world_pos: vec2_at(p, i, [0.0, 0.0]),
            size: vec2_at(size, i, default_size),
            atlas_uv: vec4_at(uv_rect, i, default_uv_rect),
            tint: vec4_at(tint, i, [1.0, 1.0, 1.0, 1.0]),
            basis: [cos_r, sin_r, -sin_r, cos_r],
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            // Sprite-Inspector-v2 v4 ABI fields: a Motion node stream has
            // no per-corner/opacity/flip authoring surface, so they take
            // their identity values (white gradient, full opacity, no
            // flip) — byte-identical render to the pre-v4 path.
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            texture_id: 0,
            // Node-graph emit doesn't have a hierarchy slot — every
            // motion node's instances share `z_order = 0`. Renderer's
            // tiebreaker (`texture_id`) groups them into one run.
            z_order: 0,
            sampling: 0,
            uv_xform: RenderInstance::IDENTITY_UV_XFORM,
            // Node-graph emit has no hierarchy → no clip silhouette.
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
        }
    };
    if n >= PAR_THRESHOLD {
        out.par_extend((0..n).into_par_iter().map(make));
    } else {
        out.extend((0..n).map(make));
    }
}

/// Lower a cooked instance stream to render instances (one per element).
/// Pure + headless. Allocates a fresh `Vec`; the per-frame path uses
/// [`lower_to_instances_into`] to reuse a buffer instead. Uses the whole-atlas
/// UV `[0,0,1,1]` for any instance without a `uv_rect` column (the shell path
/// supplies a real tile via [`lower_to_instances_into`]'s `default_uv_rect`).
pub fn lower_to_instances(stream: &Stream) -> Vec<RenderInstance> {
    let mut out = Vec::new();
    lower_to_instances_into(stream, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0], &mut out);
    out
}

/// Cook `target` at `playhead` and lower its **output port 0** to render
/// instances. Reuse the same [`Cook`] across frames for incremental cheapness.
///
/// Lowering a single port is intentional: a Motion render target is one
/// instance stream. A target with several output ports has only port 0 lowered
/// here (a multi-port target would select the port at the call site — not
/// needed by any Motion node today, all of which have exactly one output). A
/// target that legitimately declares **zero** outputs yields an empty `Vec`;
/// note the cook itself already rejects a node that *declares* an output but
/// emits none ([`CookError::OutputCountMismatch`]), so an empty result here
/// means "no output port", never a dropped stream.
pub fn evaluate_motion(
    cook: &mut Cook,
    graph: &Graph,
    ops: &dyn OpResolver,
    target: NodeId,
    playhead: f64,
) -> Result<Vec<RenderInstance>, CookError> {
    let mut out = Vec::new();
    evaluate_motion_into(
        cook,
        graph,
        ops,
        target,
        playhead,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &mut out,
    )?;
    Ok(out)
}

/// Cook `target` at `playhead` and lower its output port 0 **into `out`**,
/// reusing the buffer's capacity (M0.T11 — the per-frame bridge entry). Same
/// single-port semantics as [`evaluate_motion`]; a target with no output port
/// leaves `out` empty. Reuse the same [`Cook`] AND the same `out` across frames
/// for the zero-alloc steady state (gated by M0.T12).
///
/// `default_uv_rect` / `default_size` are the `atlas_uv` / `size` fallbacks for
/// a stream without the matching column (see [`lower_to_instances_into`]).
#[allow(clippy::too_many_arguments)] // cook + graph + resolver + target + playhead + 2 defaults + out
pub fn evaluate_motion_into(
    cook: &mut Cook,
    graph: &Graph,
    ops: &dyn OpResolver,
    target: NodeId,
    playhead: f64,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) -> Result<(), CookError> {
    let outputs = cook.cook(graph, ops, target, playhead)?;
    // A cooked output port is a `CookValue`; a Motion target's port 0 is an
    // instance stream (ADR-0058-amendment-1). A non-stream value lowers to no
    // instances (its `as_stream()` is empty).
    match outputs.first() {
        Some(v) => lower_to_instances_into(v.as_stream(), default_uv_rect, default_size, out),
        None => out.clear(),
    }
    Ok(())
}

fn scalar_at(c: Option<&Column>, i: usize, default: f32) -> f32 {
    match c {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
fn vec2_at(c: Option<&Column>, i: usize, default: [f32; 2]) -> [f32; 2] {
    match c {
        Some(Column::Vec2(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
fn vec4_at(c: Option<&Column>, i: usize, default: [f32; 4]) -> [f32; 4] {
    match c {
        Some(Column::Vec4(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
