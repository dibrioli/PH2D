#![forbid(unsafe_code)]
//! `fx.rgb_split` — **chromatic aberration / RGB split** as a per-instance stream
//! FX (Motion Nodes M4, doc 01 §3). The colour-fringed edge of a cheap lens, or of
//! a glitching monitor: the look every motion package ships (Cavalry's *RGB Split*,
//! AE's *Shift Channels* + offset preset, the *Chromatic Aberration* of every game
//! post-process stack).
//!
//! **Two looks, and they are genuinely different** (`mode`):
//!
//! - **Split** — a UNIFORM displacement `(x, y)`. The datamosh / glitch stylisation:
//!   the whole image's channels slide apart by the same amount.
//! - **Aberration** — a RADIAL displacement, `strength × (P − centroid)`. This is the
//!   physical one: *lateral* chromatic aberration is a wavelength-dependent
//!   **magnification**, so the fringe is zero at the optical axis and grows LINEARLY
//!   with the distance from it. It is what Unity/Unreal/Godot's post-process actually
//!   does (they sample the R/B channels at UVs scaled in and out from the screen
//!   centre) — the centre stays clean and the corners smear.
//!
//! **Why the shader's three channels become two ghosts here.** A pass FX splits the
//! image ADDITIVELY into R, G and B and slides them apart; the three sum back to
//! white where they overlap. This node runs on the *instance stream*, and the
//! renderer ALPHA-blends the quads (`premultiplied = 0`, standard over): three
//! stacked opaque copies would show only the topmost, not their sum — the centre
//! would come out solid blue instead of untouched. So the honest per-instance form
//! is the pair of **complementary ghosts BEHIND the untouched element**:
//!
//! ```text
//!   out = [ R ghost (+offset) ] ++ [ GB ghost (−offset) ] ++ [ the element itself ]
//!            (behind)                  (behind)                  (on top, verbatim)
//! ```
//!
//! On an opaque element that reproduces exactly the edges the additive split
//! produces — the `+` side shows R with G,B missing (a **red** fringe) and the `−`
//! side shows G,B with R missing (a **cyan** fringe) — while the body stays the
//! colour the artist authored.
//!
//! **Channel isolation is a MULTIPLY.** The `tint` column multiplies into the
//! element's colour, so masking it with `[1,0,0,·]` *is* the red channel of whatever
//! that element happened to be — a blue element throws no red fringe, exactly as it
//! should. Nothing about this node assumes white.
//!
//! Transcendental-free (HR-5): offsets and tints are add/multiply. `Effect::Pure`.
//!
//! **Place it downstream.** Like `motion.trail`, it duplicates `id`s (a ghost shares
//! its source's identity), so it belongs *after* anything that pairs state by id
//! (`motion.integrate`, `motion.spring`) — conventionally just before the Output.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod copies;
use copies::{falloff_at, positions, tile, tints};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// `mode` ≥ this → the radial (physical) aberration; below → the uniform split.
const MODE_ABERRATION: f32 = 0.5;

/// How many rows one element becomes: two ghosts + itself.
const COPIES: usize = 3;

/// Hard ceiling on the emitted element count — `3 × count` is an allocation driven
/// by an untrusted upstream. Over budget the FX **turns itself off** (the input,
/// verbatim) rather than half-drawing: a scene missing a third of its fringes reads
/// as a bug, an un-aberrated scene reads as "the effect is off".
const MAX_INSTANCES: usize = 65_536;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("fx.rgb_split"),
    name: "fx.rgb_split",
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
        // 0 = Split (uniform), 1 = Aberration (radial).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // Split: the uniform displacement, in world units.
        ParamSpec {
            name: "x",
            default: 0.06,
        },
        ParamSpec {
            name: "y",
            default: 0.0,
        },
        // Aberration: the displacement per unit of distance from the centroid.
        ParamSpec {
            name: "strength",
            default: 0.06,
        },
        // How present the fringes are. 1 = full channel isolation.
        ParamSpec {
            name: "opacity",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The fringe displacement of every element.
///
/// Split → the same `(x, y)` for all. Aberration → `strength × (P − centroid)`: zero
/// at the layout's own optical axis, growing linearly outward. The centroid (not the
/// world origin) is the axis, so the effect follows the layout wherever it is moved.
fn offsets(p: &[[f32; 2]], mode: f32, x: f32, y: f32, strength: f32) -> Vec<[f32; 2]> {
    let n = p.len();
    if mode < MODE_ABERRATION {
        return vec![[x, y]; n];
    }
    let sum = p
        .iter()
        .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    let c = [sum[0] / n.max(1) as f32, sum[1] / n.max(1) as f32];
    p.iter()
        .map(|q| [(q[0] - c[0]) * strength, (q[1] - c[1]) * strength])
        .collect()
}

/// One evaluation: the two complementary ghosts, then the untouched element.
fn split(input: &Stream, mode: f32, x: f32, y: f32, strength: f32, opacity: f32) -> Stream {
    let n = input.count();
    // Nothing to fringe, no budget for the ghosts, or the fringes were dialled to
    // nothing: forward the input verbatim rather than paying for invisible quads. A
    // junk `opacity` (NaN / ∞ — a loaded document, an MCP edit) counts as "off": it
    // would otherwise poison every fringe's alpha.
    let dead = !opacity.is_finite() || opacity <= 0.0;
    if n == 0 || n.saturating_mul(COPIES) > MAX_INSTANCES || dead {
        return input.clone();
    }
    let p = positions(input);
    let base = tints(input);
    let off = offsets(&p, mode, x, y, strength);

    let mut pos = Vec::with_capacity(n * COPIES);
    let mut tint = Vec::with_capacity(n * COPIES);
    // The R ghost, displaced `+off`.
    for i in 0..n {
        pos.push([p[i][0] + off[i][0], p[i][1] + off[i][1]]);
        let a = base[i][3] * opacity * falloff_at(input, i);
        tint.push([base[i][0], 0.0, 0.0, a]);
    }
    // The G+B ghost — the complement — displaced `−off`.
    for i in 0..n {
        pos.push([p[i][0] - off[i][0], p[i][1] - off[i][1]]);
        let a = base[i][3] * opacity * falloff_at(input, i);
        tint.push([0.0, base[i][1], base[i][2], a]);
    }
    // The element itself, verbatim and LAST, so it paints over its own fringes.
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

struct FxRgbSplit;

impl NodeOp for FxRgbSplit {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (mode, x, y) = (ctx.param("mode"), ctx.param("x"), ctx.param("y"));
        let (strength, opacity) = (ctx.param("strength"), ctx.param("opacity"));
        let out = split(ctx.input(0), mode, x, y, strength, opacity);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FxRgbSplit))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "RGB Split",
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
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Split", "Aberration"],
        },
    },
    ParamUiHint {
        param: "x",
        label: "Offset X",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "y",
        label: "Offset Y",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 0.5,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "opacity",
        label: "Opacity",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Two elements, one at the origin and one out at `x = 2`, with a size column
    /// riding along (so the copies must carry it).
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

    /// The layout: ghosts FIRST (they must draw behind), the element LAST and
    /// **verbatim**. FALSIFIED by any implementation that moves or recolours the
    /// element itself — the body of the shape must survive the effect untouched.
    #[test]
    fn the_ghosts_sit_behind_and_the_element_survives_verbatim() {
        let src = pair([1.0, 1.0, 1.0, 1.0]);
        let out = split(&src, 0.0, 0.1, 0.0, 0.0, 1.0);
        assert_eq!(out.count(), 6, "two ghosts + the element, per element");

        let (p, t) = (ps(&out), ts(&out));
        // Rows 0-1: the R ghost, displaced +x.
        assert_eq!(p[0], [0.1, 0.0]);
        assert_eq!(p[1], [2.1, 0.0]);
        // Rows 2-3: the G+B ghost, displaced −x.
        assert_eq!(p[2], [-0.1, 0.0]);
        assert_eq!(p[3], [1.9, 0.0]);
        // Rows 4-5: the elements themselves, where they always were.
        assert_eq!(p[4], [0.0, 0.0]);
        assert_eq!(p[5], [2.0, 0.0]);
        assert_eq!(t[4], [1.0; 4], "the element keeps its own colour");
        assert_eq!(t[5], [1.0; 4]);

        // The copies inherit every other column (here: `size`).
        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 6, "size rode along onto the ghosts"),
            _ => panic!("size"),
        }
    }

    /// Channel isolation is a MULTIPLY on the element's own tint — so a coloured
    /// element throws the fringes its colour actually contains. FALSIFIED by the
    /// naive "paint one ghost red and the other cyan", which would give a pure-blue
    /// element a red fringe out of nowhere.
    #[test]
    fn the_channels_are_isolated_out_of_the_elements_own_colour() {
        let out = split(&pair([0.0, 0.2, 0.8, 1.0]), 0.0, 0.1, 0.0, 0.0, 1.0);
        let t = ts(&out);
        // A blue-ish element has NO red to throw: its R ghost is black.
        assert_eq!(
            t[0],
            [0.0, 0.0, 0.0, 1.0],
            "no red in the source, no red ghost"
        );
        // …and its G+B ghost carries exactly the green and blue it does have.
        assert_eq!(t[2], [0.0, 0.2, 0.8, 1.0]);
        // The two ghosts summed = the source colour (the additive split, recovered).
        let source = ts(&pair([0.0, 0.2, 0.8, 1.0]))[0];
        for ((r, gb), src) in t[0].iter().zip(&t[2]).zip(&source).take(3) {
            assert_eq!(r + gb, *src, "the R and G+B ghosts partition the colour");
        }
    }

    /// Radial mode: the fringe is ZERO at the centroid and grows with the distance
    /// from it (lateral aberration). FALSIFIED by the uniform split, which would
    /// displace the centre element too.
    #[test]
    fn aberration_is_zero_at_the_axis_and_grows_outward() {
        // Three elements at x = 0, 1, 2 → the centroid is x = 1.
        let src = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
        let out = split(&src, 1.0, 0.0, 0.0, 0.5, 1.0);
        let p = ps(&out);
        // The R ghost of the CENTRE element sits exactly on it — no fringe on axis.
        assert_eq!(p[1], [1.0, 0.0], "zero displacement at the optical axis");
        // The outer ones smear outward, by strength × their distance from it.
        assert_eq!(p[0], [-0.5, 0.0], "1 unit out → 0.5 of fringe");
        assert_eq!(p[2], [2.5, 0.0]);
        // The G+B ghost mirrors them inward.
        assert_eq!(p[3], [0.5, 0.0]);
        assert_eq!(p[5], [1.5, 0.0]);
    }

    /// `falloff` fades the FRINGES, never the element — so an artist can aberrate a
    /// region of the layout and leave the rest clean.
    #[test]
    fn falloff_fades_the_fringes_and_never_the_element() {
        let src = pair([1.0, 1.0, 1.0, 1.0]).with("falloff", Column::Scalar(vec![0.0, 1.0]));
        let t = ts(&split(&src, 0.0, 0.1, 0.0, 0.0, 1.0));
        assert_eq!(t[0][3], 0.0, "element 0 is masked out → invisible fringe");
        assert_eq!(t[1][3], 1.0, "element 1 keeps its fringe");
        assert_eq!(t[4], [1.0; 4], "the masked element itself is untouched");
    }

    /// The effect turns ITSELF off rather than half-drawing: no opacity, or over the
    /// instance budget, forwards the input verbatim (same count, same rows).
    #[test]
    fn a_dead_opacity_or_an_over_budget_stream_forwards_the_input() {
        let src = pair([1.0, 1.0, 1.0, 1.0]);
        let off = split(&src, 0.0, 0.1, 0.0, 0.0, 0.0);
        assert_eq!(off.count(), 2);
        assert_eq!(ps(&off), ps(&src), "verbatim, not three copies of nothing");

        let huge = Stream::new(MAX_INSTANCES); // 3 × over the ceiling
        assert_eq!(split(&huge, 0.0, 0.1, 0.0, 0.0, 1.0).count(), MAX_INSTANCES);
        assert_eq!(split(&Stream::new(0), 0.0, 0.1, 0.0, 0.0, 1.0).count(), 0);
    }
}
