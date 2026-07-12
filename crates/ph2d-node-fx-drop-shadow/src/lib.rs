#![forbid(unsafe_code)]
//! `fx.drop_shadow` — a **drop shadow** as a per-instance stream FX (Motion Nodes
//! M4, doc 01 §3): every element casts a coloured ghost of itself, offset along a
//! direction, drawn behind the whole layout.
//!
//! **The reference is the Photoshop / After Effects layer effect** — Angle,
//! Distance, Opacity, Colour (+ Size/Spread, see below) — and the defaults are
//! theirs: **35 % opacity**, black, thrown down-and-right.
//!
//! ```text
//!   out = [ every element's shadow ] ++ [ every element, verbatim ]
//!              (behind, in block)            (on top)
//! ```
//!
//! **Whole-layout, not per-element.** The shadows are one block, ahead of the
//! elements, so every shadow is behind every element — Photoshop's layer shadow.
//! Interleaving (shadow, element, shadow, element, …) would let one element's shadow
//! fall ON its neighbour, which reads as dirt, not depth.
//!
//! **`Direction` is the direction the shadow FALLS** (AE's name and its meaning).
//! Photoshop calls the same dial `Angle` and points it at the *light* instead, which
//! is the opposite vector; the label here says which one it is, so nothing has to be
//! remembered. Degrees — the app's one authored-angle unit — measured
//! counter-clockwise from `+x`, in the y-up world of the Motion canvas, so the
//! default **315°** throws the shadow down-and-right.
//!
//! **A shadow is a COLOUR, not a dark copy.** Its RGB comes from the swatch (black by
//! default); only its ALPHA is inherited — `swatch.a × element.a × falloff` — so a
//! half-transparent element casts a half-transparent shadow, and a `falloff` region
//! decides *which* elements cast at all. (A `tint`-darkened copy would carry the
//! element's hue and give a red ball a red shadow.)
//!
//! **What is deliberately NOT here: blur.** Photoshop's `Size` (and the `Spread` that
//! chokes the blurred matte) are *raster* operations — they belong to the HDR
//! compositor pass FX (`fx.blur`/`fx.glow`), which is a cross-module decision, not a
//! stream node. This node is therefore a **hard-edged** shadow: the flat-design /
//! long-shadow look, honestly what it is, rather than a fake softness built from a
//! stack of ghosts. Scale the shadow with a `motion.scale` upstream if you want one.
//!
//! Transcendental-free (HR-5): the direction goes through the parabolic `cos/sin`
//! leaf. `Effect::Pure`. Like every ghost FX it duplicates `id`s, so place it
//! downstream of anything that pairs state by id — conventionally before the Output.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod copies;
mod trig;
use copies::{falloff_at, positions, tile, tints};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// How many rows one element becomes: its shadow + itself.
const COPIES: usize = 2;

/// A full turn, in the degrees the param stores — the `trig` leaf speaks **cycles**.
const DEGREES_PER_TURN: f32 = 360.0;

/// Hard ceiling on the emitted element count (`2 × count`, an untrusted upstream).
/// Over budget the FX turns itself off (the input, verbatim) rather than shadowing
/// half the layout.
const MAX_INSTANCES: usize = 65_536;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("fx.drop_shadow"),
    name: "fx.drop_shadow",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // Where the shadow FALLS (degrees, ccw from +x). 315° = down-and-right.
        ParamSpec {
            name: "direction",
            default: 315.0,
        },
        ParamSpec {
            name: "distance",
            default: 0.2,
        },
        // The shadow's colour. `a` IS the opacity (Photoshop's default: 35 % black).
        ParamSpec {
            name: "r",
            default: 0.0,
        },
        ParamSpec {
            name: "g",
            default: 0.0,
        },
        ParamSpec {
            name: "b",
            default: 0.0,
        },
        ParamSpec {
            name: "a",
            default: 0.35,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The world-space offset of every shadow: `distance` along `direction`.
fn offset(direction_deg: f32, distance: f32) -> [f32; 2] {
    let (cos, sin) = trig::cos_sin_cycles(direction_deg / DEGREES_PER_TURN);
    [cos * distance, sin * distance]
}

/// One evaluation: the shadows (behind, in a block), then the elements verbatim.
fn cast(input: &Stream, direction_deg: f32, distance: f32, color: [f32; 4]) -> Stream {
    let n = input.count();
    // Nothing to shadow, no budget, or a fully transparent shadow colour: forward the
    // input verbatim rather than paying for invisible quads. A junk alpha (NaN / ∞ — a
    // loaded document, an MCP edit) counts as "off": it would otherwise poison every
    // shadow's alpha.
    let dead = !color[3].is_finite() || color[3] <= 0.0;
    if n == 0 || n.saturating_mul(COPIES) > MAX_INSTANCES || dead {
        return input.clone();
    }
    let p = positions(input);
    let base = tints(input);
    let off = offset(direction_deg, distance);

    let mut pos = Vec::with_capacity(n * COPIES);
    let mut tint = Vec::with_capacity(n * COPIES);
    // The shadows: the swatch's colour, carrying the element's own transparency.
    for i in 0..n {
        pos.push([p[i][0] + off[0], p[i][1] + off[1]]);
        let a = color[3] * base[i][3] * falloff_at(input, i);
        tint.push([color[0], color[1], color[2], a]);
    }
    // The elements themselves, verbatim and LAST, so they paint over their shadows.
    for i in 0..n {
        pos.push(p[i]);
        tint.push(base[i]);
    }

    let mut out = Stream::new(n * COPIES);
    for (name, col) in input.columns() {
        if name != "P" && name != "tint" {
            out.set(name.clone(), tile(col, COPIES));
        }
    }
    out.set("P", Column::Vec2(pos));
    out.set("tint", Column::Vec4(tint));
    out
}

struct FxDropShadow;

impl NodeOp for FxDropShadow {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (direction, distance) = (ctx.param("direction"), ctx.param("distance"));
        let color = [
            ctx.param("r"),
            ctx.param("g"),
            ctx.param("b"),
            ctx.param("a"),
        ];
        let out = cast(ctx.input(0), direction, distance, color);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FxDropShadow))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Drop Shadow",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "direction",
        label: "Direction",
        min: 0.0,
        max: DEGREES_PER_TURN,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "distance",
        label: "Distance",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "r",
        label: "Color",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["r", "g", "b", "a"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK_35: [f32; 4] = [0.0, 0.0, 0.0, 0.35];

    fn pair(tint: [f32; 4]) -> Stream {
        Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [2.0, 0.0]]))
            .with("tint", Column::Vec4(vec![tint, tint]))
            .with("size", Column::Vec2(vec![[0.5, 0.5], [0.5, 0.5]]))
    }

    fn ps(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }
    fn ts(s: &Stream) -> Vec<[f32; 4]> {
        match s.get("tint").unwrap() {
            Column::Vec4(v) => v.clone(),
            _ => panic!("tint"),
        }
    }

    /// The layout: shadows FIRST (they must draw behind), elements LAST and verbatim.
    /// At `direction = 0°` the shadow falls along `+x`. FALSIFIED by any order that
    /// puts the shadows on top, and by any implementation that moves the element.
    #[test]
    fn the_shadows_are_one_block_behind_the_untouched_elements() {
        let out = cast(&pair([1.0, 1.0, 1.0, 1.0]), 0.0, 0.5, BLACK_35);
        assert_eq!(out.count(), 4, "a shadow + an element, per element");

        let p = ps(&out);
        assert_eq!(p[0], [0.5, 0.0], "shadow of element 0, thrown +x");
        assert_eq!(
            p[1],
            [2.5, 0.0],
            "shadow of element 1 — still in the SHADOW block"
        );
        assert_eq!(p[2], [0.0, 0.0], "the elements, where they always were");
        assert_eq!(p[3], [2.0, 0.0]);

        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 4, "size rode along onto the shadows"),
            _ => panic!("size"),
        }
    }

    /// A shadow is the COLOUR you asked for, carrying only the element's alpha —
    /// never a darkened copy of it. FALSIFIED by tinting the copy (a red element
    /// would cast a red shadow) and by ignoring the element's transparency (a ghost
    /// element would cast a solid shadow).
    #[test]
    fn the_shadow_is_a_colour_and_inherits_only_the_transparency() {
        let out = cast(&pair([1.0, 0.0, 0.0, 0.5]), 0.0, 0.5, BLACK_35);
        let t = ts(&out);
        assert_eq!(
            t[0][0..3],
            [0.0, 0.0, 0.0],
            "a RED element casts a BLACK shadow"
        );
        // 0.35 (the swatch) × 0.5 (the element's own alpha).
        assert!((t[0][3] - 0.175).abs() < 1e-6, "alpha = {}", t[0][3]);
        assert_eq!(
            t[2],
            [1.0, 0.0, 0.0, 0.5],
            "the element keeps its own colour"
        );
    }

    /// `direction` is DEGREES in a y-up world, so the default 315° throws the shadow
    /// down-and-right (`+x`, `−y`). FALSIFIED by a radians/cycles mix-up (315 rad is
    /// somewhere else entirely) and by the sign flip that throws it up-and-left.
    #[test]
    fn the_default_direction_throws_the_shadow_down_and_right() {
        let off = offset(315.0, 1.0);
        // cos 315° = +1/√2, sin 315° = −1/√2 (the parabolic leaf is ~0.1 % off).
        let diag = std::f32::consts::FRAC_1_SQRT_2;
        assert!((off[0] - diag).abs() < 0.01, "x = {}", off[0]);
        assert!((off[1] + diag).abs() < 0.01, "y = {}", off[1]);
        // A full turn is the same direction (the leaf wraps).
        let wrapped = offset(315.0 + DEGREES_PER_TURN, 1.0);
        assert!((wrapped[0] - off[0]).abs() < 1e-5);
    }

    /// `falloff` decides WHICH elements cast — the shadow fades, the element does not.
    #[test]
    fn falloff_picks_the_casters() {
        let src = pair([1.0, 1.0, 1.0, 1.0]).with("falloff", Column::Scalar(vec![0.0, 1.0]));
        let t = ts(&cast(&src, 0.0, 0.5, BLACK_35));
        assert_eq!(t[0][3], 0.0, "element 0 casts nothing");
        assert_eq!(t[1][3], 0.35, "element 1 casts at full opacity");
        assert_eq!(t[2], [1.0; 4], "the non-caster is itself untouched");
    }

    /// The effect turns ITSELF off rather than shadowing half the layout: a fully
    /// transparent swatch, an empty stream, or an over-budget one forwards the input.
    #[test]
    fn a_transparent_swatch_or_an_over_budget_stream_forwards_the_input() {
        let src = pair([1.0, 1.0, 1.0, 1.0]);
        let off = cast(&src, 0.0, 0.5, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(off.count(), 2);
        assert_eq!(ps(&off), ps(&src), "verbatim");

        let huge = Stream::new(MAX_INSTANCES); // 2 × over the ceiling
        assert_eq!(cast(&huge, 0.0, 0.5, BLACK_35).count(), MAX_INSTANCES);
        assert_eq!(cast(&Stream::new(0), 0.0, 0.5, BLACK_35).count(), 0);
    }
}
