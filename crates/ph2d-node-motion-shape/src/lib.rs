#![forbid(unsafe_code)]
//! `source.shape` — **generate a parametric vector shape** (ADR-0154).
//!
//! The state of the art (Cavalry, After Effects shape layers, Blender Geometry
//! Nodes, Rive) treats a shape as **live vector geometry** that flows through the
//! graph and renders resolution-independent on GPU. This node is that door for
//! Motion: it emits **one instance carrying a `geometry_id`** — a handle to a
//! crisp `VecPath` the shell built from these params and drew through Vello. Cross
//! it with a `motion.grid` through a `motion.duplicator` and the shape is stamped,
//! crisp, at every point — with none of a baked tile's dead pixels (nothing
//! downstream can re-shape a raster; a `VecPath` composes with boolean/trim/deform).
//!
//! ## The same door `source.object` uses, one axis up
//!
//! A node is handed its params, its inputs and the playhead — nothing else (the
//! property that lets the cook memoize and replay bit-exactly), so it **cannot
//! reach the GPU or the vector library**. The shell builds the `VecPath` from
//! these params, registers it in its `VecPathStore`, and publishes a one-row
//! instance stream `(P, geometry_id, size, tint)` into the cook's external channel
//! under this shape's **content key** ([`shape_key`]); this node reads that key —
//! exactly as `source.object` reads an object by name. Identical descriptors share
//! ONE `VecPath` (content-addressed). A shape not yet published (a forward cook) is
//! an **empty external → empty stream**: the node emits nothing, never panics.
//!
//! `Effect::Pure` — the output is a pure function of the named external, whose
//! content-revision rides in the cook fingerprint, so editing a slider re-cooks
//! this node and only what is downstream of it.

use ph2d_node_registry::{NodeRegistry, ParamGate, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// **The shape family.** The index the `kind` param stores (`0.0..`) maps here; the
/// labels drive the `ParamWidget::Enum` dropdown. A superset of MiniCavalry's 7
/// (Circle..Heart) plus **Gear** (from `ph2d-vec-scene`, which MiniCavalry lacks).
/// All eight are FILLABLE closed shapes; Arc (wedge) and Spiral are follow-ups
/// (they need wedge-close / stroke, which the fill-based draw entry doesn't do).
/// Order is the wire format for the `kind` index — **append only** (Arc/Spiral will
/// append at 8/9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShapeKind {
    Circle,
    Square,
    Ellipse,
    Rectangle,
    Polygon,
    Star,
    Heart,
    Gear,
}

/// The dropdown labels — index-aligned to [`ShapeKind`] (the `Enum` widget stores
/// the selected INDEX as the `kind` param's f32).
pub static KIND_LABELS: &[&str] = &[
    "Circle",
    "Square",
    "Ellipse",
    "Rectangle",
    "Polygon",
    "Star",
    "Heart",
    "Gear",
];

impl ShapeKind {
    /// Decode the `kind` param (a rounded, clamped index) — the ONE place the f32
    /// index becomes a kind, so the node and the shell can never disagree about
    /// which shape an index names.
    #[must_use]
    pub fn from_index(idx: f32) -> ShapeKind {
        match (idx.round() as i64).clamp(0, KIND_LABELS.len() as i64 - 1) {
            0 => ShapeKind::Circle,
            1 => ShapeKind::Square,
            2 => ShapeKind::Ellipse,
            3 => ShapeKind::Rectangle,
            4 => ShapeKind::Polygon,
            5 => ShapeKind::Star,
            6 => ShapeKind::Heart,
            _ => ShapeKind::Gear,
        }
    }
}

/// **The whole shape descriptor** — a pure value the node emits (as a key) and the
/// shell reads (to build the `VecPath`). Each shape shows only the params it uses
/// (the `ParamGate`s below): a circle is just `size`, a gear adds `tooth_depth` +
/// `hole`. A few params are shared where the geometry genuinely is (`aspect` for
/// ellipse/rectangle, `sides` for polygon/star/gear, `corner` rounds boxes/polys/
/// star points); the rest are dedicated so every slider reads honestly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeParams {
    pub kind: ShapeKind,
    /// Primary size in world units (radius for round kinds, half-extent for boxes).
    pub size: f32,
    /// Height ÷ width — ellipse / rectangle stretch (`1.0` = round/square).
    pub aspect: f32,
    /// Sides / points / teeth — polygon / star / gear.
    pub sides: u32,
    /// Corner rounding as a FRACTION of `size` (`0` = sharp) — box/polygon/star.
    pub corner: f32,
    /// Star point depth: the inner-radius ratio `0.05..0.95` (small = spiky).
    pub star_depth: f32,
    /// Heart cleft: how deep the top dip sits (`0.02..0.45`).
    pub cleft: f32,
    /// Gear tooth depth as a fraction of the radius (`0.05..0.6`).
    pub tooth_depth: f32,
    /// Gear centre hole as a fraction of the root circle (`0` = solid).
    pub hole: f32,
}

/// The names of the f32 params — the ONE list the manifest, the UI hints, the
/// gates, the node's `eval` and the shell's reader all key off (no drifting
/// string literal).
pub mod param {
    pub const KIND: &str = "kind";
    pub const SIZE: &str = "size";
    pub const ASPECT: &str = "aspect";
    pub const SIDES: &str = "sides";
    pub const CORNER: &str = "corner";
    pub const STAR_DEPTH: &str = "star_depth";
    pub const CLEFT: &str = "cleft";
    pub const TOOTH_DEPTH: &str = "tooth_depth";
    pub const HOLE: &str = "hole";
}

impl ShapeParams {
    /// Read the descriptor from any param source — the node passes
    /// `|n| ctx.param(n)`, the shell passes a closure over the graph's overrides +
    /// the manifest defaults. **One reader, two callers** ⇒ the node's key and the
    /// shell's key are the same f32 bits through the same code.
    #[must_use]
    pub fn read(get: impl Fn(&str) -> f32) -> ShapeParams {
        ShapeParams {
            kind: ShapeKind::from_index(get(param::KIND)),
            size: get(param::SIZE),
            aspect: get(param::ASPECT),
            // Clamp is deliberate here (not in `from_index`): the UI caps at 32, but
            // the builder decides the floor; keep the int stable so the key is exact.
            sides: (get(param::SIDES).round() as i64).clamp(3, 32) as u32,
            corner: get(param::CORNER),
            star_depth: get(param::STAR_DEPTH),
            cleft: get(param::CLEFT),
            tooth_depth: get(param::TOOTH_DEPTH),
            hole: get(param::HOLE),
        }
    }
}

/// **The content key** of a shape descriptor (ADR-0154) — the external name the
/// node reads and the shell publishes under. Content-addressed: identical
/// descriptors share ONE `VecPath` in the store. Deterministic and exact — the
/// same f32 bits format the same string, so the node's lookup and the shell's
/// publish cannot diverge (gated). Hashes the WHOLE descriptor: a hidden param
/// stays at its default for any shape an artist authors, so identical shapes still
/// share geometry, and no per-kind branch can drift from the reader.
#[must_use]
pub fn shape_key(p: &ShapeParams) -> String {
    format!(
        "shape:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        p.kind as u32,
        p.size.to_bits(),
        p.aspect.to_bits(),
        p.sides,
        p.corner.to_bits(),
        p.star_depth.to_bits(),
        p.cleft.to_bits(),
        p.tooth_depth.to_bits(),
        p.hole.to_bits(),
    )
}

/// The static contract of this node type (ADR-0031). The `kind` param is an
/// enum index; the eight geometry params are f32 sliders, gated per-kind by
/// [`PARAM_GATES`]. Frozen `NodeManifest` (8 fields) untouched — these are
/// `ParamSpec`s (f32), not new fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("source.shape"),
    name: "source.shape",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: param::KIND,
            default: 0.0,
        },
        ParamSpec {
            name: param::SIZE,
            default: 1.0,
        },
        ParamSpec {
            name: param::ASPECT,
            default: 1.0,
        },
        ParamSpec {
            name: param::SIDES,
            default: 6.0,
        },
        ParamSpec {
            name: param::CORNER,
            default: 0.0,
        },
        ParamSpec {
            name: param::STAR_DEPTH,
            default: 0.45,
        },
        ParamSpec {
            name: param::CLEFT,
            default: 0.2,
        },
        ParamSpec {
            name: param::TOOTH_DEPTH,
            default: 0.35,
        },
        ParamSpec {
            name: param::HOLE,
            default: 0.45,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct SourceShape;

impl NodeOp for SourceShape {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let p = ShapeParams::read(|n| ctx.param(n));
        let key = shape_key(&p);
        // The shell built this shape's `VecPath`, stored it, and published a
        // one-row instance stream `(P, geometry_id, size, tint)` under `key`.
        // Clone is refcount (Arc columns); a key with no published shape (a
        // forward cook, before the shell's publish pass ran) is the empty
        // external → an empty stream ⇒ nothing drawn, no panic.
        let stream = ctx.external(&key).clone();
        ctx.emit(stream);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SourceShape))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Shape",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    // Its output carries `geometry_id` (a live vector shape). The GPU cook's lowering
    // is sprite-only and has no vector path, so a document bringing a shape in recuses
    // to the CPU render (ADR-0154/0155).
    reg.register_appearance_source(MANIFEST.id);
    Ok(())
}

/// The param rows: a real dropdown for the shape family (the segmented `Enum`
/// widget the Vector panel uses for Cap/Join), then the geometry sliders. Every
/// row past `size` is gated by [`PARAM_GATES`], so the panel shows ONLY the
/// controls the current `kind` uses.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: param::KIND,
        label: "Shape",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Enum {
            labels: KIND_LABELS,
        },
    },
    ParamUiHint {
        param: param::SIZE,
        label: "Size",
        min: 0.05,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::ASPECT,
        label: "Aspect (H/W)",
        min: 0.1,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::SIDES,
        label: "Sides / Points / Teeth",
        min: 3.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: param::CORNER,
        label: "Corner Radius",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::STAR_DEPTH,
        label: "Point Depth",
        min: 0.05,
        max: 0.95,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::CLEFT,
        label: "Cleft",
        min: 0.02,
        max: 0.45,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::TOOTH_DEPTH,
        label: "Tooth Depth",
        min: 0.05,
        max: 0.6,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::HOLE,
        label: "Hole",
        min: 0.0,
        max: 0.9,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// **Per-kind visibility** — a param appears only when `kind` is one of the listed
/// values (the enum indices from [`ShapeKind`]). `kind` and `size` have no gate
/// (always shown). This is the SAME per-kind truth the builder keys off, so a
/// shown control is a control that does something.
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: param::ASPECT,
        when: param::KIND,
        values: &[ShapeKind::Ellipse as i32, ShapeKind::Rectangle as i32],
    },
    ParamGate {
        param: param::SIDES,
        when: param::KIND,
        values: &[
            ShapeKind::Polygon as i32,
            ShapeKind::Star as i32,
            ShapeKind::Gear as i32,
        ],
    },
    ParamGate {
        param: param::CORNER,
        when: param::KIND,
        values: &[
            ShapeKind::Square as i32,
            ShapeKind::Rectangle as i32,
            ShapeKind::Polygon as i32,
            ShapeKind::Star as i32,
        ],
    },
    ParamGate {
        param: param::STAR_DEPTH,
        when: param::KIND,
        values: &[ShapeKind::Star as i32],
    },
    ParamGate {
        param: param::CLEFT,
        when: param::KIND,
        values: &[ShapeKind::Heart as i32],
    },
    ParamGate {
        param: param::TOOTH_DEPTH,
        when: param::KIND,
        values: &[ShapeKind::Gear as i32],
    },
    ParamGate {
        param: param::HOLE,
        when: param::KIND,
        values: &[ShapeKind::Gear as i32],
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
