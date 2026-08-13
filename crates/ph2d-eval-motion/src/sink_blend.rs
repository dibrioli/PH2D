//! **How a render sink composites** — the sink's blend mode (doc 89, folha 17).
//!
//! Split out of `lib.rs` at the HR-18 LOC cap along the seam that was already
//! there: `lib.rs` runs the CLOCK (`MotionCookPump`) and `lower.rs` answers *what
//! does a cooked stream look like on screen*; this file answers the one question
//! that is neither — *in what MODE does this sink draw?* — and is the door both
//! render routes ask.
//!
//! The reference is unanimous and it decided the shape: Niagara puts blend on the
//! Sprite Renderer's material, Cavalry on the layer/shader, AE and Stardust on the
//! layer. Blend belongs to the RENDERER, not to a particle — so it is a param of
//! `motion.output` and a scalar of the lowering, never a per-element column.
//!
//! ⚠️ **The tag cannot travel as a stream column, and the reason is structural.**
//! On the device `motion.output` is `GpuKernel::PASSTHROUGH`: the sequencer emits
//! no pass for it, so anything its `eval` wrote would never reach the device
//! lowering. Both lowerings take the tag as an argument instead — the CPU pump
//! asks this door inside its sink loop, the shell asks it for the single sink the
//! GPU route accepts, and `ph2d-gpu-cook` receives it (that crate keeps
//! `ph2d-eval-motion` a DEV dependency on purpose).

use ph2d_nodegraph::graph::{Graph, NodeId};

/// The sink param that names a render sink's blend mode.
///
/// It is the SAME string `ph2d-node-motion-output` declares as `BLEND_PARAM`.
/// Neither crate may depend on the other (both are leaves of the node system), so
/// the agreement is pinned by a gate in the shell — the one place that sees both.
/// This substrate already knows a vocabulary of well-known NAMES (`"P"`, `"size"`,
/// `"rot"`, `"tint"`, `"uv_rect"`, `"texture_id"`, `"geometry_id"`); this joins it
/// as the first well-known *param* name rather than column.
pub const SINK_BLEND_PARAM: &str = "blend";

/// The blend tag a sink draws with — `ph2d_ecs::BlendMode::tag()`, `0..=5`.
///
/// **The one door.** The CPU pump asks it per sink (it loops many); the shell asks
/// it for the single sink the GPU route accepts. Two callers, one answer — a
/// second reader would be free to round or clamp differently, and the two routes
/// would composite the same document differently, which no gate that looks at one
/// route can see.
///
/// A node with no override, and any node that is not a sink, yields `0` (`Mix`) —
/// exactly the `flip_uv: 0` both lowerings hardcoded before this param existed.
/// Rounding is half-away-from-zero (`f32::round`) and the result is CLAMPED into
/// range rather than wrapped: a corrupt document composites normally instead of
/// selecting a blend nobody authored.
#[must_use]
pub fn sink_blend_tag(graph: &Graph, sink: NodeId) -> u8 {
    let v = graph
        .node_param_overrides(sink)
        .and_then(|p| p.get(SINK_BLEND_PARAM))
        .copied()
        .unwrap_or(0.0);
    if !v.is_finite() {
        return 0;
    }
    // The ceiling is the renderer's pipeline array, read FROM the renderer — a
    // literal `5` here would keep compiling on the day a seventh mode lands and
    // would silently refuse to select it.
    let top = (ph2d_render::pipeline::BLEND_PIPELINE_COUNT - 1) as f32;
    v.round().clamp(0.0, top) as u8
}

#[cfg(test)]
#[path = "sink_blend_tests.rs"]
mod tests;
