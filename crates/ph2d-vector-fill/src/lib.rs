#![forbid(unsafe_code)]
//! `ph2d-vector-fill` — the procedural-fill **shader graph** for the Vector
//! Module (ADR-0060, spec `05_procedural_fill.md`).
//!
//! A [`FillGraph`] is a small DAG of [`FillNode`]s that compiles, on the fly, to
//! a WGSL fill function. The defining property (ADR-0060 §2.3, resolves the
//! Antigravity "compile stutter" critique B) is the **topology vs params split**:
//!
//! - **Topology** — *which* nodes and *which* connections — is the only thing the
//!   WGSL string depends on. It is compiled **once** per topology (keyed by
//!   [`blake3`] hash; see [`cache`]).
//! - **Params** — every scalar/colour/enum value — ride a uniform buffer
//!   ([`ubo::FillParamsUbo`]) updated per frame. Crucially, **enum controls**
//!   (noise kind, blend mode, coord mode, math op) are *also* params: the WGSL
//!   emits all variants behind a `switch kind_idx { … }` and the index lives in
//!   the UBO. So animating a colour, a frequency, *or an enum* is a `write_buffer`
//!   — **zero** recompiles (gate `procedural_fill_no_recompile_on_animate`).
//!
//! ## Module map
//!
//! | Module            | Role                                                          |
//! |-------------------|---------------------------------------------------------------|
//! | `lib` (this)      | [`FillGraph`] / [`FillNode`] / [`Connection`] + validation    |
//! | [`eval`]          | CPU reference evaluator — the semantics WGSL must match       |
//! | [`wgsl_codegen`]  | DAG → WGSL string (SSA in `fill_main`) + topology hash        |
//! | [`ubo`]           | `FillParamsUbo` (per-frame params, zero-alloc updates)        |
//! | [`cache`]         | in-memory LRU compile cache (codegen + naga validate)         |
//! | [`diffusion_curve`] | diffusion-curve authoring model (Orzan sides → OKLab) — W7  |
//! | [`poisson_cpu`]   | deterministic multigrid Poisson solver → [`ColorField`] — W7  |
//!
//! ## W6 scope (handoff §2.X frontier)
//!
//! The 17-node enum is **frozen** (ADR-0060 §2.7). W6 implements the 12
//! pure-procedural generators/combinators with real WGSL codegen + CPU eval.
//! [`FillNode::MeshGradient`] is wired on the CPU in W7 step 2 — it samples a
//! solved [`ColorField`] (see [`poisson_cpu`] / [`eval::eval_color_with_fields`]);
//! its WGSL codegen still returns [`FillCodegenError::NotYetImplemented`] pending
//! the renderer's texture binding (step 3). The other 4 resource-bound nodes
//! ([`FillNode::Pattern`], [`FillNode::ProceduralShader`], [`FillNode::Image`],
//! [`FillNode::ImageSample`]) still return `NotYetImplemented` from both codegen
//! and eval — they need the W8 brush bridge / texture-binding infra.

use glam::Vec2;
use ph2d_color::OklchColor;
use smallvec::SmallVec;

pub mod cache;
pub mod diffusion_curve;
pub mod diffusion_gpu;
pub mod eval;
pub mod poisson_cpu;
pub mod ubo;
pub mod wgsl_codegen;

pub use cache::CompileCache;
pub use diffusion_curve::{ColorStop, DiffusionCurve, DiffusionCurveSet};
pub use diffusion_gpu::{
    DiffusionAlgorithm, DiffusionParams, DiffusionPlan, DiffusionTier, GpuSegment, WosConfig,
    pack_curves, walk_on_spheres_field, wos_estimate_point,
};
pub use eval::{eval_color, eval_color_with_fields};
pub use poisson_cpu::{
    ColorField, FieldResolver, FieldStore, NoFields, Resolution, solve_color_field,
};
pub use wgsl_codegen::{CompiledFill, TopologyHash};

/// Inline node capacity / hard cap on graph size (ADR-0060 §2.7 — adding nodes
/// past 16 inline is fine, but the UBO addresses exactly [`MAX_FILL_NODES`]).
pub const MAX_FILL_NODES: usize = 16;
/// Max gradient/ramp stops a single node carries (UBO stop table width).
pub const MAX_GRADIENT_STOPS: usize = 8;

/// Index of a node inside [`FillGraph::nodes`]. Doubles as the UBO slot index.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u32);

impl NodeId {
    #[inline]
    fn idx(self) -> usize {
        self.0 as usize
    }
}

/// The value type carried on a wire. WGSL: `Scalar`→`f32`, `Vec2`→`vec2<f32>`,
/// `Vec3`→`vec3<f32>`, `Color`→`vec4<f32>` (linear-light RGBA).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FillType {
    Scalar,
    Vec2,
    Vec3,
    Color,
}

impl FillType {
    /// The WGSL type name this lowers to.
    pub fn wgsl(self) -> &'static str {
        match self {
            FillType::Scalar => "f32",
            FillType::Vec2 => "vec2<f32>",
            FillType::Vec3 => "vec3<f32>",
            FillType::Color => "vec4<f32>",
        }
    }
}

/// Which lattice/cellular noise a [`FillNode::Noise`] evaluates. The variant is
/// an **enum control** — it rides the UBO as a `u32`; the WGSL compiles all four
/// behind a `switch`, so changing kind never recompiles (ADR-0060 §2.3).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NoiseKind {
    Simplex,
    Perlin,
    Worley,
    /// Fractal sum of value noise; `lacunarity`/`persistence` ride the UBO.
    Fbm {
        lacunarity: f32,
        persistence: f32,
    },
}

impl NoiseKind {
    /// The `u32` the UBO carries / the `switch` case the WGSL dispatches on.
    pub fn index(self) -> u32 {
        match self {
            NoiseKind::Simplex => 0,
            NoiseKind::Perlin => 1,
            NoiseKind::Worley => 2,
            NoiseKind::Fbm { .. } => 3,
        }
    }
}

/// Coordinate space a [`FillNode::Coord`] produces from the ambient fill-space
/// position. `World`/`Screen` are identity until the renderer supplies its
/// matrices (Coordenador wiring) — the variant exists and dispatches via UBO so
/// adding the transform later is a UBO change, not a recompile.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CoordMode {
    Local,
    World,
    Screen,
    Polar,
}

impl CoordMode {
    pub fn index(self) -> u32 {
        match self {
            CoordMode::Local => 0,
            CoordMode::World => 1,
            CoordMode::Screen => 2,
            CoordMode::Polar => 3,
        }
    }
}

/// Scalar op a [`FillNode::Math`] applies. Unary ops ignore the second input.
/// Semantics follow WGSL exactly (`Fract` is `x - floor(x)`, ≥ 0 — *not* Rust's
/// sign-preserving `f32::fract`; see [`ph2d_expr`] gotchas) so CPU↔GPU agree.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    Abs,
    Sin,
    Cos,
    Floor,
    Fract,
    Sqrt,
    Pow,
    Mod,
}

impl MathOp {
    pub fn index(self) -> u32 {
        match self {
            MathOp::Add => 0,
            MathOp::Sub => 1,
            MathOp::Mul => 2,
            MathOp::Div => 3,
            MathOp::Min => 4,
            MathOp::Max => 5,
            MathOp::Abs => 6,
            MathOp::Sin => 7,
            MathOp::Cos => 8,
            MathOp::Floor => 9,
            MathOp::Fract => 10,
            MathOp::Sqrt => 11,
            MathOp::Pow => 12,
            MathOp::Mod => 13,
        }
    }
}

/// How [`FillNode::Mix`] combines its two colour inputs before the `factor`
/// lerp. Self-contained (does not pull the Painter compositor's 22 modes — this
/// is the fill graph's own small, pure set).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
    Lerp,
}

impl BlendMode {
    pub fn index(self) -> u32 {
        match self {
            BlendMode::Normal => 0,
            BlendMode::Multiply => 1,
            BlendMode::Screen => 2,
            BlendMode::Overlay => 3,
            BlendMode::Add => 4,
            BlendMode::Lerp => 5,
        }
    }
}

/// One stop of a gradient/ramp palette. `position` is in `[0, 1]`; colours are
/// authored OKLCH and packed into the UBO as linear-light RGBA.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GradientStop {
    pub color: OklchColor,
    pub position: f32,
}

impl GradientStop {
    pub fn new(color: OklchColor, position: f32) -> Self {
        Self { color, position }
    }
}

/// The 17 canonical fill nodes (ADR-0060 §2.2 — **frozen**). Authored param
/// values live in the variant; they are the *initial* UBO contents and never
/// enter the topology hash.
#[derive(Clone, Debug, PartialEq)]
pub enum FillNode {
    // ── Generators (output Color) ──────────────────────────────────────────
    /// Flat colour.
    Solid { color: OklchColor },
    /// Axis gradient at `angle` (radians) across the ambient (or input) uv.
    LinearGradient {
        stops: SmallVec<[GradientStop; MAX_GRADIENT_STOPS]>,
        angle: f32,
    },
    /// Radial gradient from `center` out to `radius`.
    RadialGradient {
        stops: SmallVec<[GradientStop; MAX_GRADIENT_STOPS]>,
        center: Vec2,
        radius: f32,
    },
    /// Diffusion-curve mesh gradient. CPU eval samples the solved [`ColorField`]
    /// (W7 step 2, [`crate::eval::eval_color_with_fields`]); WGSL codegen is
    /// still pending its GPU texture binding (renderer wiring, W7 step 3).
    MeshGradient { gradient_id: u64 },
    /// Painter brush pattern — **W8 stub** (the `ph2d-brush-traits` bridge).
    Pattern { pattern_ref: u64 },
    /// Recursive sub-shader — **stub** (needs the resolver, out of W6 scope).
    ProceduralShader { shader_id: u64 },
    /// Sampled image generator — **stub** (texture binding, out of W6 scope).
    Image { image_ref: u64 },

    // ── Procedural primitives ──────────────────────────────────────────────
    /// Lattice/cellular noise → scalar. `kind` dispatches via UBO.
    Noise {
        kind: NoiseKind,
        frequency: f32,
        octaves: u32,
    },
    /// Cellular F1 distance → scalar.
    Voronoi { cells: u32, jitter: f32 },
    /// Palette lookup: scalar `t` → Color.
    Ramp {
        palette: SmallVec<[GradientStop; MAX_GRADIENT_STOPS]>,
    },

    // ── Combinators ────────────────────────────────────────────────────────
    /// Blend two colours by `mode`, then lerp toward input `a` by `factor`.
    Mix { mode: BlendMode, factor: f32 },
    /// Derive a surface normal (Vec3) from a scalar height field by `strength`.
    Bump { strength: f32 },

    // ── Coords + math ──────────────────────────────────────────────────────
    /// Transform the ambient (or input) coordinate into `mode` space.
    Coord { mode: CoordMode },
    /// Apply a scalar math op (unary ops ignore the 2nd input).
    Math { op: MathOp },
    /// Sample an image at a uv input — **stub** (texture binding).
    ImageSample { image_ref: u64 },

    // ── Temporal ───────────────────────────────────────────────────────────
    /// The animation clock (seconds) → scalar.
    Time,
    /// Deterministic per-position hash seeded by `seed` → scalar in `[0, 1)`.
    Random { seed: u32 },
}

impl FillNode {
    /// The wire type this node outputs.
    pub fn output_type(&self) -> FillType {
        match self {
            FillNode::Solid { .. }
            | FillNode::LinearGradient { .. }
            | FillNode::RadialGradient { .. }
            | FillNode::MeshGradient { .. }
            | FillNode::Pattern { .. }
            | FillNode::ProceduralShader { .. }
            | FillNode::Image { .. }
            | FillNode::Ramp { .. }
            | FillNode::Mix { .. }
            | FillNode::ImageSample { .. } => FillType::Color,
            FillNode::Noise { .. }
            | FillNode::Voronoi { .. }
            | FillNode::Math { .. }
            | FillNode::Time
            | FillNode::Random { .. } => FillType::Scalar,
            FillNode::Bump { .. } => FillType::Vec3,
            FillNode::Coord { .. } => FillType::Vec2,
        }
    }

    /// The ordered expected types of this node's input ports. An *unconnected*
    /// port falls back to a default ([`eval`]/[`wgsl_codegen`] supply it):
    /// `Vec2` → the ambient `coord`, `Scalar` → `0.0`, `Color` → transparent.
    pub fn input_ports(&self) -> &'static [FillType] {
        match self {
            FillNode::Solid { .. }
            | FillNode::MeshGradient { .. }
            | FillNode::Pattern { .. }
            | FillNode::ProceduralShader { .. }
            | FillNode::Image { .. }
            | FillNode::Time
            | FillNode::Random { .. } => &[],
            FillNode::LinearGradient { .. }
            | FillNode::RadialGradient { .. }
            | FillNode::Noise { .. }
            | FillNode::Voronoi { .. }
            | FillNode::Coord { .. }
            | FillNode::ImageSample { .. } => &[FillType::Vec2],
            FillNode::Ramp { .. } | FillNode::Bump { .. } => &[FillType::Scalar],
            FillNode::Mix { .. } => &[FillType::Color, FillType::Color],
            FillNode::Math { .. } => &[FillType::Scalar, FillType::Scalar],
        }
    }

    /// Human-readable port label (diagnostics only).
    pub fn port_name(&self, port: u32) -> &'static str {
        match (self, port) {
            (FillNode::Mix { .. }, 0) => "a",
            (FillNode::Mix { .. }, 1) => "b",
            (FillNode::Math { .. }, 0) => "a",
            (FillNode::Math { .. }, 1) => "b",
            (FillNode::Ramp { .. }, 0) => "t",
            (FillNode::Bump { .. }, 0) => "height",
            (_, 0) => "uv",
            _ => "?",
        }
    }

    /// `true` for the 5 nodes the **WGSL codegen** cannot emit yet. Four are
    /// pure resource stubs (brush `Pattern`, recursive `ProceduralShader`, the
    /// two texture nodes); `MeshGradient` is here only because its WGSL form
    /// needs a GPU texture binding for its solved field (renderer wiring, W7
    /// step 3) — on the CPU it is already evaluable ([`Self::lacks_cpu_eval`]
    /// excludes it).
    pub fn is_stub(&self) -> bool {
        matches!(
            self,
            FillNode::MeshGradient { .. }
                | FillNode::Pattern { .. }
                | FillNode::ProceduralShader { .. }
                | FillNode::Image { .. }
                | FillNode::ImageSample { .. }
        )
    }

    /// `true` for the resource-bound nodes the **CPU reference evaluator** still
    /// cannot reproduce: the Painter-brush `Pattern`, the recursive
    /// `ProceduralShader`, and the two texture nodes. `MeshGradient` is *not*
    /// among them — W7 step 2 makes it evaluable by sampling a solved
    /// [`ColorField`] ([`crate::eval::eval_color_with_fields`]).
    pub fn lacks_cpu_eval(&self) -> bool {
        matches!(
            self,
            FillNode::Pattern { .. }
                | FillNode::ProceduralShader { .. }
                | FillNode::Image { .. }
                | FillNode::ImageSample { .. }
        )
    }

    /// Stable, param-free tag — the *structural* identity that enters the
    /// topology hash. Two nodes with the same tag generate identical WGSL
    /// (their differing params ride the UBO), so the tag is exactly what the
    /// cache key must capture.
    pub fn topology_tag(&self) -> &'static str {
        match self {
            FillNode::Solid { .. } => "Solid",
            FillNode::LinearGradient { .. } => "LinearGradient",
            FillNode::RadialGradient { .. } => "RadialGradient",
            FillNode::MeshGradient { .. } => "MeshGradient",
            FillNode::Pattern { .. } => "Pattern",
            FillNode::ProceduralShader { .. } => "ProceduralShader",
            FillNode::Image { .. } => "Image",
            FillNode::Noise { .. } => "Noise",
            FillNode::Voronoi { .. } => "Voronoi",
            FillNode::Ramp { .. } => "Ramp",
            FillNode::Mix { .. } => "Mix",
            FillNode::Bump { .. } => "Bump",
            FillNode::Coord { .. } => "Coord",
            FillNode::Math { .. } => "Math",
            FillNode::ImageSample { .. } => "ImageSample",
            FillNode::Time => "Time",
            FillNode::Random { .. } => "Random",
        }
    }
}

/// A directed edge `from_node`'s (single) output → `to_node`'s `to_input` port.
/// `from_output` is reserved for multi-output nodes (always `0` in W6).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    pub from_node: NodeId,
    pub from_output: u32,
    pub to_node: NodeId,
    pub to_input: u32,
}

impl Connection {
    /// Single-output convenience constructor.
    pub fn new(from_node: NodeId, to_node: NodeId, to_input: u32) -> Self {
        Self {
            from_node,
            from_output: 0,
            to_node,
            to_input,
        }
    }
}

/// A procedural-fill DAG: nodes + connections + the sink that produces the
/// final colour.
#[derive(Clone, Debug, PartialEq)]
pub struct FillGraph {
    pub nodes: SmallVec<[FillNode; MAX_FILL_NODES]>,
    pub connections: SmallVec<[Connection; 32]>,
    pub output_node_id: NodeId,
}

/// Why a [`FillGraph`] is malformed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FillGraphError {
    /// More than [`MAX_FILL_NODES`] nodes (UBO cannot address them).
    TooManyNodes { count: usize, max: usize },
    /// `output_node_id` is out of range.
    OutputOutOfRange { id: u32, len: usize },
    /// The output node does not produce a `Color`.
    OutputNotColor { ty: FillType },
    /// A connection references a node index that doesn't exist.
    BadConnectionNode { conn: usize },
    /// A connection targets a port the node doesn't have.
    PortOutOfRange {
        conn: usize,
        node: u32,
        port: u32,
        ports: usize,
    },
    /// A connection's source type doesn't match the target port type.
    TypeMismatch {
        conn: usize,
        from: FillType,
        to: FillType,
    },
    /// Two connections drive the same input port.
    DuplicateInput { node: u32, port: u32 },
    /// The graph has a cycle (not a DAG).
    Cycle,
}

impl core::fmt::Display for FillGraphError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FillGraphError::TooManyNodes { count, max } => {
                write!(f, "too many nodes: {count} > cap {max}")
            }
            FillGraphError::OutputOutOfRange { id, len } => {
                write!(f, "output_node_id {id} out of range (len {len})")
            }
            FillGraphError::OutputNotColor { ty } => {
                write!(f, "output node must be Color, got {ty:?}")
            }
            FillGraphError::BadConnectionNode { conn } => {
                write!(f, "connection {conn} references a missing node")
            }
            FillGraphError::PortOutOfRange {
                conn,
                node,
                port,
                ports,
            } => write!(
                f,
                "connection {conn}: node {node} has no input port {port} ({ports} ports)"
            ),
            FillGraphError::TypeMismatch { conn, from, to } => {
                write!(f, "connection {conn}: type {from:?} → {to:?} mismatch")
            }
            FillGraphError::DuplicateInput { node, port } => {
                write!(f, "node {node} input port {port} driven twice")
            }
            FillGraphError::Cycle => write!(f, "graph is cyclic (not a DAG)"),
        }
    }
}

impl std::error::Error for FillGraphError {}

/// Per-node DFS colour for cycle detection / topological ordering.
#[derive(Copy, Clone, PartialEq)]
enum Mark {
    Unvisited,
    InProgress,
    Done,
}

impl FillGraph {
    /// Validate structure, typing, single-driver-per-port, and acyclicity
    /// (edit-time check, spec §5.1.4). Returns `Ok(())` for a sound graph.
    pub fn validate(&self) -> Result<(), FillGraphError> {
        if self.nodes.len() > MAX_FILL_NODES {
            return Err(FillGraphError::TooManyNodes {
                count: self.nodes.len(),
                max: MAX_FILL_NODES,
            });
        }
        let out = self.output_node_id.idx();
        if out >= self.nodes.len() {
            return Err(FillGraphError::OutputOutOfRange {
                id: self.output_node_id.0,
                len: self.nodes.len(),
            });
        }
        let out_ty = self.nodes[out].output_type();
        if out_ty != FillType::Color {
            return Err(FillGraphError::OutputNotColor { ty: out_ty });
        }

        // Per-connection checks + duplicate-driver detection.
        let mut driven: SmallVec<[(u32, u32); 32]> = SmallVec::new();
        for (ci, c) in self.connections.iter().enumerate() {
            let (from, to) = (c.from_node.idx(), c.to_node.idx());
            if from >= self.nodes.len() || to >= self.nodes.len() {
                return Err(FillGraphError::BadConnectionNode { conn: ci });
            }
            let ports = self.nodes[to].input_ports();
            if c.to_input as usize >= ports.len() {
                return Err(FillGraphError::PortOutOfRange {
                    conn: ci,
                    node: c.to_node.0,
                    port: c.to_input,
                    ports: ports.len(),
                });
            }
            let from_ty = self.nodes[from].output_type();
            let to_ty = ports[c.to_input as usize];
            if from_ty != to_ty {
                return Err(FillGraphError::TypeMismatch {
                    conn: ci,
                    from: from_ty,
                    to: to_ty,
                });
            }
            let key = (c.to_node.0, c.to_input);
            if driven.contains(&key) {
                return Err(FillGraphError::DuplicateInput {
                    node: c.to_node.0,
                    port: c.to_input,
                });
            }
            driven.push(key);
        }

        // Acyclicity over the whole graph (3-colour DFS from every node).
        let mut mark = vec![Mark::Unvisited; self.nodes.len()];
        let mut scratch: Vec<NodeId> = Vec::new();
        for n in 0..self.nodes.len() {
            self.dfs(NodeId(n as u32), &mut mark, &mut scratch)?;
        }
        Ok(())
    }

    /// Sorted (by port) input connections of `node`. Small N — fine to rebuild.
    fn inputs_of(&self, node: NodeId) -> SmallVec<[Connection; 4]> {
        let mut v: SmallVec<[Connection; 4]> = self
            .connections
            .iter()
            .filter(|c| c.to_node == node)
            .copied()
            .collect();
        v.sort_by_key(|c| c.to_input);
        v
    }

    /// Dependency (post-order) DFS: pushes `node` to `order` after all its
    /// transitive inputs. Detects cycles. Deterministic (inputs visited in port
    /// order, then by appearance).
    fn dfs(
        &self,
        node: NodeId,
        mark: &mut [Mark],
        order: &mut Vec<NodeId>,
    ) -> Result<(), FillGraphError> {
        match mark[node.idx()] {
            Mark::Done => return Ok(()),
            Mark::InProgress => return Err(FillGraphError::Cycle),
            Mark::Unvisited => {}
        }
        mark[node.idx()] = Mark::InProgress;
        for c in self.inputs_of(node) {
            self.dfs(c.from_node, mark, order)?;
        }
        mark[node.idx()] = Mark::Done;
        order.push(node);
        Ok(())
    }

    /// Nodes reachable from the output, in dependency order (inputs first).
    /// Dead nodes (not feeding the output) are excluded — codegen/eval only
    /// touch what contributes to the final colour. Assumes [`validate`] passed.
    pub fn dependency_order(&self) -> Result<Vec<NodeId>, FillGraphError> {
        let mut mark = vec![Mark::Unvisited; self.nodes.len()];
        let mut order = Vec::new();
        self.dfs(self.output_node_id, &mut mark, &mut order)?;
        Ok(order)
    }

    /// The single connection driving `(node, port)`, if any.
    pub(crate) fn driver(&self, node: NodeId, port: u32) -> Option<NodeId> {
        self.connections
            .iter()
            .find(|c| c.to_node == node && c.to_input == port)
            .map(|c| c.from_node)
    }

    #[inline]
    pub fn node(&self, id: NodeId) -> &FillNode {
        &self.nodes[id.idx()]
    }
}

/// Codegen/eval failure (a [`FillGraphError`], an unimplemented stub node, or a
/// naga rejection of the generated WGSL).
#[derive(Clone, Debug, PartialEq)]
pub enum FillCodegenError {
    Graph(FillGraphError),
    /// A W6-stub node was reached by the output (handoff §2.X frontier).
    NotYetImplemented {
        node: u32,
        kind: &'static str,
    },
    NagaParse(String),
    NagaValidate(String),
}

impl core::fmt::Display for FillCodegenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FillCodegenError::Graph(e) => write!(f, "invalid graph: {e}"),
            FillCodegenError::NotYetImplemented { node, kind } => {
                write!(f, "node {node} ({kind}) is not implemented in W6 (stub)")
            }
            FillCodegenError::NagaParse(e) => write!(f, "generated WGSL failed to parse: {e}"),
            FillCodegenError::NagaValidate(e) => {
                write!(f, "generated WGSL failed to validate: {e}")
            }
        }
    }
}

impl std::error::Error for FillCodegenError {}

impl From<FillGraphError> for FillCodegenError {
    fn from(e: FillGraphError) -> Self {
        FillCodegenError::Graph(e)
    }
}
