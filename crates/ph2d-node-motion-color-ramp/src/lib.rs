#![forbid(unsafe_code)]
//! `motion.color_ramp` — **colour instances by a gradient**: the Blender "Color Ramp" /
//! the Houdini ramp parameter (Motion Nodes M1, colour — doc 01 §1.7 / doc 29). Maps a
//! per-instance scalar to a colour along a multi-stop gradient and writes the `tint`
//! column. Until now the only colour node was `motion.tint` (a single solid); this is
//! the continuous one — colour by index / distance / velocity / any value field.
//!
//! **Algorithm — sample a `ph2d-color::ColorRamp` per instance.** The scalar `t` for
//! element `i` is the `t` value input (`v[i]`, clamped `0..1`) when connected, else the
//! normalised index `i/(n-1)` (a gradient laid across the set). A `preset` picks a
//! built-in ramp (Rainbow / Heat / Ice / Grayscale) or a `Custom` two-stop from the
//! colour params; `interp` is Linear or Ease (smoothstep). The ramp is evaluated in
//! linear RGB — the same space the `tint` column and the compositor use — so no colour
//! conversion here. The foundational `ph2d-color` owns the ramp maths; this node is a
//! thin per-instance map. `Effect::Pure`.

use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `t` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Presets (the `preset` param).
const PRESET_RAINBOW: i64 = 0;
const PRESET_HEAT: i64 = 1;
const PRESET_ICE: i64 = 2;
const PRESET_GRAYSCALE: i64 = 3;
// PRESET_CUSTOM (4) = the two colour params.

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.color_ramp"),
    name: "motion.color_ramp",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Per-instance scalar to map (animatable). Optional: unconnected → normalised
        // index.
        PortSpec {
            name: "t",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Rainbow · 1 Heat · 2 Ice · 3 Grayscale · 4 Custom (a→b).
        ParamSpec {
            name: "preset",
            default: 0.0,
        },
        // 0 Linear · 1 Ease (smoothstep).
        ParamSpec {
            name: "interp",
            default: 0.0,
        },
        // Custom two-stop colours (linear RGB).
        ParamSpec {
            name: "a_r",
            default: 0.0,
        },
        ParamSpec {
            name: "a_g",
            default: 0.0,
        },
        ParamSpec {
            name: "a_b",
            default: 0.0,
        },
        ParamSpec {
            name: "b_r",
            default: 1.0,
        },
        ParamSpec {
            name: "b_g",
            default: 1.0,
        },
        ParamSpec {
            name: "b_b",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn rgb(r: f32, g: f32, b: f32) -> [f32; 4] {
    [r, g, b, 1.0]
}

/// Build the ramp for a preset (Rgb space; `interp` Linear or Ease). Custom uses the two
/// colour params `a`/`b`.
fn build_ramp(preset: i64, a: [f32; 4], b: [f32; 4], interp: RampInterp) -> ColorRamp {
    let stops: Vec<[f32; 4]> = match preset {
        PRESET_RAINBOW => vec![
            rgb(1.0, 0.0, 0.0),
            rgb(1.0, 1.0, 0.0),
            rgb(0.0, 1.0, 0.0),
            rgb(0.0, 1.0, 1.0),
            rgb(0.0, 0.0, 1.0),
            rgb(1.0, 0.0, 1.0),
            rgb(1.0, 0.0, 0.0),
        ],
        PRESET_HEAT => vec![
            rgb(0.0, 0.0, 0.0),
            rgb(0.7, 0.0, 0.0),
            rgb(1.0, 0.4, 0.0),
            rgb(1.0, 1.0, 0.2),
            rgb(1.0, 1.0, 1.0),
        ],
        PRESET_ICE => vec![
            rgb(0.0, 0.0, 0.25),
            rgb(0.0, 0.55, 1.0),
            rgb(0.75, 1.0, 1.0),
        ],
        PRESET_GRAYSCALE => vec![rgb(0.0, 0.0, 0.0), rgb(1.0, 1.0, 1.0)],
        _ => vec![a, b], // Custom
    };
    let n = stops.len().max(2);
    let ramp_stops: Vec<RampStop> = stops
        .iter()
        .enumerate()
        .map(|(i, c)| RampStop::new(i as f32 / (n as f32 - 1.0), *c))
        .collect();
    ColorRamp::new(ramp_stops, RampColorMode::Rgb, interp)
}

/// Map every instance's scalar through the ramp. `t_field` is the connected value input
/// (empty → normalised index).
fn colorize(n: usize, ramp: &ColorRamp, t_field: &[f32]) -> Vec<[f32; 4]> {
    (0..n)
        .map(|i| {
            let t = if t_field.is_empty() {
                if n <= 1 {
                    0.0
                } else {
                    i as f32 / (n as f32 - 1.0)
                }
            } else {
                t_field.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0)
            };
            ramp.eval(t)
        })
        .collect()
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

struct MotionColorRamp;

impl NodeOp for MotionColorRamp {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let preset = ctx.param("preset").round() as i64;
        let interp = if ctx.param("interp").round() as i64 == 0 {
            RampInterp::Linear
        } else {
            RampInterp::Ease
        };
        let a = rgb(ctx.param("a_r"), ctx.param("a_g"), ctx.param("a_b"));
        let b = rgb(ctx.param("b_r"), ctx.param("b_g"), ctx.param("b_b"));
        let ramp = build_ramp(preset, a, b, interp);
        let t_field = scalar_col(ctx.input(1), VALUE_COL);
        let input = ctx.input(0);
        let n = input.count();
        let tint = colorize(n, &ramp, &t_field);
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
    reg.register(Box::new(MotionColorRamp))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Color Ramp",
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
        param: "preset",
        label: "Preset",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Rainbow", "Heat", "Ice", "Grayscale", "Custom"],
        },
    },
    ParamUiHint {
        param: "interp",
        label: "Interp",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Ease"],
        },
    },
    chan("a_r", "A R"),
    chan("a_g", "A G"),
    chan("a_b", "A B"),
    chan("b_r", "B R"),
    chan("b_g", "B G"),
    chan("b_b", "B B"),
];

const fn chan(param: &'static str, label: &'static str) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Grayscale by normalised index: the first element is black, the last is white, and
    /// the middle is mid-grey. FALSIFIED if the ramp were a single solid colour.
    #[test]
    fn grayscale_spreads_black_to_white_by_index() {
        let ramp = build_ramp(
            PRESET_GRAYSCALE,
            rgb(0.0, 0.0, 0.0),
            rgb(1.0, 1.0, 1.0),
            RampInterp::Linear,
        );
        let c = colorize(5, &ramp, &[]);
        assert!(c[0][0] < 0.05, "first is black: {:?}", c[0]);
        assert!(c[4][0] > 0.95, "last is white: {:?}", c[4]);
        assert!((c[2][0] - 0.5).abs() < 0.1, "middle is grey: {:?}", c[2]);
    }

    /// The `t` value field overrides the index: two elements both fed `t=1` are both the
    /// ramp's end colour (white), regardless of their index.
    #[test]
    fn the_t_field_overrides_the_index() {
        let ramp = build_ramp(
            PRESET_GRAYSCALE,
            rgb(0.0, 0.0, 0.0),
            rgb(1.0, 1.0, 1.0),
            RampInterp::Linear,
        );
        let c = colorize(2, &ramp, &[1.0, 1.0]);
        assert!(c[0][0] > 0.95 && c[1][0] > 0.95, "both white: {c:?}");
    }

    /// The rainbow preset actually spans hues: across the set the colours are not all
    /// equal (the red channel alone varies a lot).
    #[test]
    fn rainbow_spans_many_colours() {
        let ramp = build_ramp(
            PRESET_RAINBOW,
            rgb(0.0, 0.0, 0.0),
            rgb(0.0, 0.0, 0.0),
            RampInterp::Linear,
        );
        let c = colorize(12, &ramp, &[]);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for col in &c {
            lo = lo.min(col[2]); // blue channel sweeps 0→1→0 across the wheel
            hi = hi.max(col[2]);
        }
        assert!(hi - lo > 0.8, "the rainbow spans (blue {lo}..{hi})");
    }

    /// Deterministic + cooks through the registry: writes the `tint` column at the full
    /// count and passes the geometry columns through.
    #[test]
    fn registers_and_colours_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.color_ramp.test.src"),
            name: "motion.color_ramp.test.src",
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
                ctx.emit(
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionColorRamp),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.color_ramp.test.src");
        let cr = g.add_node("motion.color_ramp");
        g.set_param(cr, "preset", PRESET_GRAYSCALE as f32);
        g.connect(Edge {
            from: (src, 0),
            to: (cr, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, cr, 0.0).unwrap();
        let s = out[0].as_stream();
        assert!(s.get("P").is_some(), "geometry passes through");
        match s.get("tint").unwrap() {
            Column::Vec4(v) => {
                assert_eq!(v.len(), 3, "tint at full count");
                assert!(v[0][0] < 0.05 && v[2][0] > 0.95, "black to white by index");
            }
            _ => panic!("tint"),
        }
    }
}
