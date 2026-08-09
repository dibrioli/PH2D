#![forbid(unsafe_code)]
//! `motion.color_array` — **cycle a palette across instances by index**: the Cinema 4D
//! MoGraph "Color" / a palette cycler (Motion Nodes M1, colour — doc 01 §1.7 / doc 29).
//! Where `motion.color_ramp` is the *continuous* colour node (a gradient), this is the
//! *discrete* one: element `i` gets palette slot `i mod colours`, writing the `tint`
//! column — the classic hard-striped clone colouring.
//!
//! **Algorithm — a modular palette lookup.** Up to four colour slots (linear RGB) with
//! `colours` (2–4) active; element `i` takes slot `(i + offset) mod colours`. An `offset`
//! **value** input shifts which slot each index gets, so a `value.lfo` marches the
//! palette across the set. Transcendental-free (HR-5): a modulo lookup, no maths.
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `offset` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// The palette an untouched node paints with — red / green / blue / yellow, the four
/// the fixed params used to default to, so a graph that never authored a palette looks
/// exactly like it did before this wave.
///
/// ⚠️ **This is a DEFAULT, not a cap.** There is no maximum: the length lives in the
/// text param, and `cycle` reads `palette.len()`.
pub use ph2d_color::DEFAULT_PALETTE_FALLBACK as DEFAULT_PALETTE;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.color_array"),
    name: "motion.color_array",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Shifts which slot each index gets (animatable). Optional: unconnected → 0.
        PortSpec {
            name: "offset",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // ⚠️ **NO f32 params, and that IS the wave** (Enio: *"color array poderia ter
    // quantas cores o usuário quisesse, tire os limites"*). The palette used to be four
    // colours because four is how many `ParamSpec`s were written down, and the `colors`
    // count existed to shrink that fixed list. Both were limits of the REPRESENTATION.
    //
    // The palette now travels as the text param `palette` (`ph2d_color::palette_text`),
    // the canonical channel for a param that is not one f32 — the same road
    // `motion.color_ramp` took for its gradient, and for the same reason: a
    // variable-length list of colours cannot be a fixed set of `f32` params.
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
///
/// ⚠️ The tenth-and-something copy of this helper in the node library — see the note on
/// `motion.color_ramp::falloff_at`. The extraction is a wave of its own across nine
/// crates; this follows the convention the library already chose.
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The instance's existing colour (absent → opaque white).
fn existing_tint(stream: &Stream, i: usize) -> [f32; 4] {
    match stream.get("tint") {
        Some(Column::Vec4(v)) => v.get(i).copied().unwrap_or([1.0; 4]),
        _ => [1.0; 4],
    }
}

/// `lerp(existing, target, f)` per RGBA channel, endpoint-EXACT: at `f = 1` the first
/// term is `existing · 0.0` and the second `target · 1.0`, so an absent mask writes the
/// palette slot **bit for bit** — the substitution this node has always performed.
fn mixed_tint(existing: [f32; 4], target: [f32; 4], f: f32) -> [f32; 4] {
    let lerp = |e: f32, t: f32| e * (1.0 - f) + t * f;
    [
        lerp(existing[0], target[0]),
        lerp(existing[1], target[1]),
        lerp(existing[2], target[2]),
        lerp(existing[3], target[3]),
    ]
}

/// Assign palette slot `(i + offset) mod colours` to each of `n` elements, masked by
/// `falloff` — the same law `motion.color_ramp` and `motion.tint` apply, so a `field.*`
/// reaches the DISCRETE colour node and the continuous one identically (doc 89 fam. 9).
fn cycle(
    n: usize,
    palette: &[[f32; 4]],
    colors: usize,
    offset: i64,
    input: &Stream,
) -> Vec<[f32; 4]> {
    let colors = colors.clamp(1, palette.len());
    (0..n)
        .map(|i| {
            let idx = (i as i64 + offset).rem_euclid(colors as i64) as usize;
            mixed_tint(existing_tint(input, i), palette[idx], falloff_at(input, i))
        })
        .collect()
}

fn scalar_first(s: &Stream, name: &str) -> Option<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.first().copied(),
        _ => None,
    }
}

struct MotionColorArray;

impl NodeOp for MotionColorArray {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // The authored palette, or the factory one when the artist has not touched it.
        // A malformed string falls back too — `parse_palette` refuses rather than
        // silently shortening, so the visible default beats a palette missing its tail.
        let palette: Vec<[f32; 4]> = ctx
            .text_param("palette")
            .and_then(ph2d_color::parse_palette)
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| DEFAULT_PALETTE.to_vec());
        let colors = palette.len();
        let _ = &palette;
        let offset = scalar_first(ctx.input(1), VALUE_COL)
            .map(|v| v.round() as i64)
            .unwrap_or(0);
        let input = ctx.input(0);
        let n = input.count();
        let tint = cycle(n, &palette, colors, offset, input);
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "tint" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("tint", Column::Vec4(tint));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionColorArray))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Color Array",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// ONE row: the palette itself. The count is `palette.len()`, the colours are the
/// colours — there is nothing else to say, and no `colors` slider to disagree with the
/// list it was capping.
static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "palette",
    label: "Palette",
    min: 0.0,
    max: 0.0,
    step: 0.0,
    widget: ParamWidget::Palette,
}];

#[cfg(test)]
mod tests {
    use super::*;

    const PAL: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0, 0.0, 1.0],
    ];

    /// The palette cycles by index: with 3 active colours, element `i` gets slot
    /// `i mod 3`. FALSIFIED if every element got one colour.
    #[test]
    fn the_palette_cycles_by_index() {
        let c = cycle(7, &PAL, 3, 0, &Stream::new(7));
        assert_eq!(c[0], PAL[0]);
        assert_eq!(c[1], PAL[1]);
        assert_eq!(c[2], PAL[2]);
        assert_eq!(c[3], PAL[0], "wraps after 3");
        assert_eq!(c[6], PAL[0]);
    }

    /// `offset` marches the palette: offset 1 shifts every element one slot along.
    #[test]
    fn offset_marches_the_palette() {
        let base = cycle(4, &PAL, 4, 0, &Stream::new(4));
        let shifted = cycle(4, &PAL, 4, 1, &Stream::new(4));
        assert_eq!(shifted[0], base[1], "element 0 took slot 1");
        assert_eq!(shifted[3], base[0], "element 3 wrapped to slot 0");
    }

    /// `colors` bounds the active slots: with 2 active, only slots 0 and 1 appear.
    #[test]
    fn the_palette_length_is_the_cycle_length() {
        let c = cycle(6, &PAL, 2, 0, &Stream::new(6));
        for col in &c {
            assert!(
                *col == PAL[0] || *col == PAL[1],
                "only two colours: {col:?}"
            );
        }
    }

    /// **The field masks the palette, and no field is byte-identical** (doc 89 fam. 9, P0).
    ///
    /// The DISCRETE colour node gets the same law as the continuous one — a `field.*`
    /// writes `falloff` and the stripes paint only where it reaches. The two halves are
    /// asserted with `assert_eq!` on raw bits, not an epsilon: at `f = 1` the lerp is
    /// `existing·0 + slot·1`, exactly the slot, and at `f = 0` exactly what was there.
    #[test]
    fn the_field_masks_the_palette_and_no_field_changes_nothing() {
        let existing = vec![[1.0, 0.0, 0.0, 1.0]; 3];
        let masked = Stream::new(3)
            .with("tint", Column::Vec4(existing.clone()))
            .with("falloff", Column::Scalar(vec![1.0, 0.5, 0.0]));
        let got = cycle(3, &PAL, 3, 0, &masked);
        let bare = cycle(3, &PAL, 3, 0, &Stream::new(3));

        assert_eq!(got[0], bare[0], "full falloff takes the slot EXACTLY");
        assert_eq!(got[2], existing[2], "zero falloff keeps the colour EXACTLY");
        // Half must be strictly between — the half a boolean mask would collapse.
        let (lo, hi) = (
            existing[1][0].min(bare[1][0]),
            existing[1][0].max(bare[1][0]),
        );
        assert!(
            hi - lo > 1e-6 && got[1][0] > lo + 1e-6 && got[1][0] < hi - 1e-6,
            "half falloff must land BETWEEN {lo} and {hi}, got {}",
            got[1][0]
        );

        // …and a stream carrying a colour but NO mask is the substitution of before.
        let no_field = Stream::new(3).with("tint", Column::Vec4(existing));
        assert_eq!(
            cycle(3, &PAL, 3, 0, &no_field),
            bare,
            "absent falloff must write the palette slot bit for bit"
        );
    }

    /// Deterministic + cooks through the registry: writes the `tint` column at the full
    /// count and passes geometry through.
    #[test]
    fn registers_and_colours_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.color_array.test.src"),
            name: "motion.color_array.test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(4).with(
                    "P",
                    Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
                ));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionColorArray),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.color_array.test.src");
        let ca = g.add_node("motion.color_array");
        // A TWO-colour palette: the count is the list's length, not a slider that caps
        // a longer fixed list — so authoring "two colours" is authoring two colours.
        g.set_text_param(
            ca,
            "palette",
            ph2d_color::serialize_palette(&[[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]]),
        );
        g.connect(Edge {
            from: (src, 0),
            to: (ca, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, ca, 0.0).unwrap();
        let s = out[0].as_stream();
        assert!(s.get("P").is_some(), "geometry passes through");
        match s.get("tint").unwrap() {
            Column::Vec4(v) => {
                assert_eq!(v.len(), 4, "tint at full count");
                assert_eq!(v[0], v[2], "slot 0 repeats at index 2 (2-colour cycle)");
                assert_ne!(v[0], v[1], "adjacent differ");
            }
            _ => panic!("tint"),
        }
    }
}
