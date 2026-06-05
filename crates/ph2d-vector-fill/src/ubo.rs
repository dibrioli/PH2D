//! `FillParamsUbo` — the per-frame parameter channel (ADR-0060 §2.3, spec §5.4).
//!
//! Every scalar/colour/enum a [`FillGraph`](crate::FillGraph) reads lives here,
//! **indexed by node id**. The WGSL ([`crate::wgsl_codegen`]) reads the exact
//! same slots; the CPU evaluator ([`crate::eval`]) reads this struct too, so CPU
//! and GPU consume *identical* values — parity is by construction, not by luck.
//!
//! Animation is a zero-alloc [`bytemuck::bytes_of`] of a mutated `FillParamsUbo`
//! into `wgpu::Queue::write_buffer` (HR-3). Because **enum controls ride
//! [`Self::ucontrol`]** rather than being baked into the WGSL, even changing a
//! noise kind / blend mode / coord mode / math op is a UBO write — never a
//! recompile (gate `procedural_fill_no_recompile_on_animate`).
//!
//! ## std140 layout
//!
//! Scalars are packed into `vec4` lanes (never bare `array<f32, N>` — WGSL would
//! round each element to a 16-byte stride). The Rust struct is `#[repr(C,
//! align(16))]` with no padding holes, so it is `Pod` and the byte image matches
//! the WGSL `struct FillParams` declared in [`crate::wgsl_codegen`] exactly.

use crate::{
    BlendMode, CoordMode, FillGraph, FillNode, GradientStop, MAX_FILL_NODES, MAX_GRADIENT_STOPS,
    MathOp, NodeId, NoiseKind,
};

/// `MAX_GRADIENT_STOPS / 4` — gradient/ramp stop positions packed 4 per `vec4`.
pub const STOP_POS_LANES: usize = MAX_GRADIENT_STOPS / 4;

/// The uniform block. One row per node slot (`[..MAX_FILL_NODES]`). Field
/// meaning is node-kind specific; the packing is the single source of truth that
/// [`crate::wgsl_codegen`] and [`crate::eval`] both read back. See the per-field
/// docs and [`FillParamsUbo::pack_node`].
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FillParamsUbo {
    /// Per-node primary colour, linear-light RGBA (Solid). Gradient/ramp nodes
    /// use [`Self::stop_colors`] instead.
    pub colors: [[f32; 4]; MAX_FILL_NODES],
    /// Per-node float params (lane meaning is node-specific):
    /// - `Noise`   → `[frequency, lacunarity, persistence, _]`
    /// - `Voronoi` → `[jitter, _, _, _]`
    /// - `Mix`     → `[factor, _, _, _]`
    /// - `Bump`    → `[strength, _, _, _]`
    /// - `LinearGradient`  → `[angle, _, _, _]`
    /// - `RadialGradient`  → `[center.x, center.y, radius, _]`
    pub scalars: [[f32; 4]; MAX_FILL_NODES],
    /// Per-node `u32` controls (lane meaning is node-specific):
    /// - `Noise`   → `[kind, octaves, _, _]`
    /// - `Voronoi` → `[cells, _, _, _]`
    /// - `Mix`     → `[mode, _, _, _]`
    /// - `Coord`   → `[mode, _, _, _]`
    /// - `Math`    → `[op, _, _, _]`
    /// - `Ramp` / `LinearGradient` / `RadialGradient` → `[stop_count, _, _, _]`
    /// - `Random`  → `[seed, _, _, _]`
    pub ucontrol: [[u32; 4]; MAX_FILL_NODES],
    /// Per-node gradient/ramp stop colours, linear-light RGBA.
    pub stop_colors: [[[f32; 4]; MAX_GRADIENT_STOPS]; MAX_FILL_NODES],
    /// Per-node gradient/ramp stop positions in `[0, 1]`, packed 4 per `vec4`.
    pub stop_pos: [[[f32; 4]; STOP_POS_LANES]; MAX_FILL_NODES],
    /// Global animation clock (seconds) — read by [`FillNode::Time`].
    pub time: f32,
    /// std140 tail padding so the struct image matches WGSL's auto-padded block.
    pub _pad: [f32; 3],
}

impl Default for FillParamsUbo {
    fn default() -> Self {
        bytemuck::Zeroable::zeroed()
    }
}

impl FillParamsUbo {
    /// A fully zeroed block (every slot transparent / 0).
    pub fn zeroed() -> Self {
        bytemuck::Zeroable::zeroed()
    }

    /// Snapshot the graph's *authored* params into the UBO. This is the initial
    /// frame; animation then mutates individual slots in place.
    pub fn from_graph(graph: &FillGraph) -> Self {
        let mut ubo = Self::zeroed();
        for (i, node) in graph.nodes.iter().enumerate().take(MAX_FILL_NODES) {
            ubo.pack_node(NodeId(i as u32), node);
        }
        ubo
    }

    /// Pack one node's authored params into its slot. Idempotent; safe to call
    /// for any node (stubs simply write nothing meaningful).
    pub fn pack_node(&mut self, id: NodeId, node: &FillNode) {
        let i = id.0 as usize;
        if i >= MAX_FILL_NODES {
            return;
        }
        match node {
            FillNode::Solid { color } => {
                self.colors[i] = color.to_linear().as_array();
            }
            FillNode::LinearGradient { stops, angle } => {
                self.scalars[i][0] = *angle;
                self.pack_stops(i, stops);
            }
            FillNode::RadialGradient {
                stops,
                center,
                radius,
            } => {
                self.scalars[i] = [center.x, center.y, *radius, 0.0];
                self.pack_stops(i, stops);
            }
            FillNode::Noise {
                kind,
                frequency,
                octaves,
            } => {
                let (lac, pers) = match kind {
                    NoiseKind::Fbm {
                        lacunarity,
                        persistence,
                    } => (*lacunarity, *persistence),
                    _ => (2.0, 0.5),
                };
                self.scalars[i] = [*frequency, lac, pers, 0.0];
                self.ucontrol[i][0] = kind.index();
                self.ucontrol[i][1] = *octaves;
            }
            FillNode::Voronoi { cells, jitter } => {
                self.scalars[i][0] = *jitter;
                self.ucontrol[i][0] = *cells;
            }
            FillNode::Ramp { palette } => {
                self.pack_stops(i, palette);
            }
            FillNode::Mix { mode, factor } => {
                self.scalars[i][0] = *factor;
                self.ucontrol[i][0] = mode.index();
            }
            FillNode::Bump { strength } => {
                self.scalars[i][0] = *strength;
            }
            FillNode::Coord { mode } => {
                self.ucontrol[i][0] = mode.index();
            }
            FillNode::Math { op } => {
                self.ucontrol[i][0] = op.index();
            }
            FillNode::Random { seed } => {
                self.ucontrol[i][0] = *seed;
            }
            FillNode::Time => {}
            // Stubs carry no UBO params in W6.
            FillNode::MeshGradient { .. }
            | FillNode::Pattern { .. }
            | FillNode::ProceduralShader { .. }
            | FillNode::Image { .. }
            | FillNode::ImageSample { .. } => {}
        }
    }

    fn pack_stops(&mut self, i: usize, stops: &[GradientStop]) {
        let n = stops.len().min(MAX_GRADIENT_STOPS);
        self.ucontrol[i][0] = n as u32;
        for (s, stop) in stops.iter().take(n).enumerate() {
            self.stop_colors[i][s] = stop.color.to_linear().as_array();
            self.stop_pos[i][s / 4][s % 4] = stop.position;
        }
    }

    /// The whole block as bytes for `wgpu::Queue::write_buffer` (zero alloc).
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    // ── Zero-alloc animation setters (the per-frame hot path) ───────────────

    /// Update a `Solid` node's colour (linear-light RGBA).
    pub fn set_color(&mut self, id: NodeId, rgba: [f32; 4]) {
        if let Some(slot) = self.colors.get_mut(id.0 as usize) {
            *slot = rgba;
        }
    }

    /// Update lane `lane` (0..4) of a node's float params.
    pub fn set_scalar(&mut self, id: NodeId, lane: usize, v: f32) {
        if let Some(slot) = self.scalars.get_mut(id.0 as usize)
            && lane < 4
        {
            slot[lane] = v;
        }
    }

    /// Update lane `lane` (0..4) of a node's `u32` controls — this is how an
    /// **enum** (noise kind / blend mode / coord / math op) animates without a
    /// recompile.
    pub fn set_control(&mut self, id: NodeId, lane: usize, v: u32) {
        if let Some(slot) = self.ucontrol.get_mut(id.0 as usize)
            && lane < 4
        {
            slot[lane] = v;
        }
    }

    /// Convenience: set a `Noise`/`Coord`/`Math`/`Mix` node's enum control.
    pub fn set_noise_kind(&mut self, id: NodeId, kind: NoiseKind) {
        self.set_control(id, 0, kind.index());
    }
    pub fn set_coord_mode(&mut self, id: NodeId, mode: CoordMode) {
        self.set_control(id, 0, mode.index());
    }
    pub fn set_math_op(&mut self, id: NodeId, op: MathOp) {
        self.set_control(id, 0, op.index());
    }
    pub fn set_blend_mode(&mut self, id: NodeId, mode: BlendMode) {
        self.set_control(id, 0, mode.index());
    }

    /// Advance the animation clock.
    pub fn set_time(&mut self, seconds: f32) {
        self.time = seconds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubo_is_std140_sized_and_pod() {
        // Multiple of 16 (std140 struct), no surprise padding.
        assert_eq!(core::mem::size_of::<FillParamsUbo>() % 16, 0);
        assert_eq!(core::mem::align_of::<FillParamsUbo>(), 16);
        // bytes_of must not panic and must cover the whole struct.
        let ubo = FillParamsUbo::zeroed();
        assert_eq!(ubo.as_bytes().len(), core::mem::size_of::<FillParamsUbo>());
    }

    #[test]
    fn enum_control_is_a_pure_write() {
        let mut ubo = FillParamsUbo::zeroed();
        ubo.set_noise_kind(NodeId(2), NoiseKind::Worley);
        assert_eq!(ubo.ucontrol[2][0], NoiseKind::Worley.index());
        ubo.set_noise_kind(NodeId(2), NoiseKind::Perlin);
        assert_eq!(ubo.ucontrol[2][0], NoiseKind::Perlin.index());
    }
}
