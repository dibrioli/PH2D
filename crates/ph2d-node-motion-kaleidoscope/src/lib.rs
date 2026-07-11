#![forbid(unsafe_code)]
//! `motion.kaleidoscope` — **N-fold radial symmetry**: replicate a layout into
//! `segments` slices rotated around a pivot, optionally mirroring alternate slices — a
//! mandala / kaleidoscope generator (Motion Nodes M3, distributions — doc 01 §3 /
//! doc 26). The N-fold generalisation of `motion.mirror`, which is the 2-fold (D₁)
//! case: mirror reflects once, this rotates `segments` copies (and, with `reflect`,
//! mirrors every other one).
//!
//! **Algorithm — the orbit of the source under the dihedral group Dₙ.** For a source of
//! `n` elements and `segments = k`, the output has `k · n` elements: slice `s` is the
//! source rotated about `(pivot_x, pivot_y)` by `s · (1/k)` turn (plus a global `spin`).
//! With **rotational** symmetry (`reflect` off) that is the cyclic group Cₖ — `k` plain
//! rotated copies. With **reflected** symmetry (`reflect` on) every odd slice is first
//! mirrored (its local `y` negated), so adjacent slices meet as mirror images — the
//! dihedral group Dₖ, the true kaleidoscope look. Only the **position** `P` is
//! transformed; every other column (`size`, `tint`, `id`, …) is duplicated onto each
//! copy. Transcendental-free (HR-5): the rotation uses `cos_sin_cycles` — the parabolic
//! sine copied from `motion.orbit` — so no `sin`/`cos`; no `sqrt`. `Effect::Pure` (no
//! clock — the spin animation arrives through the value input).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `spin` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Max slices (a bound on the fan-out — `segments · n` elements).
const MAX_SEGMENTS: i64 = 256;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.kaleidoscope"),
    name: "motion.kaleidoscope",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Global rotation of the whole pattern, in degrees (animatable). Optional:
        // unconnected reads as 0.
        PortSpec {
            name: "spin",
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
        // Number of rotational slices (the N of N-fold).
        ParamSpec {
            name: "segments",
            default: 6.0,
        },
        // 0 = rotational (Cₙ); 1 = mirrored (Dₙ, the kaleidoscope look).
        ParamSpec {
            name: "reflect",
            default: 1.0,
        },
        // Centre of symmetry (world units).
        ParamSpec {
            name: "pivot_x",
            default: 0.0,
        },
        ParamSpec {
            name: "pivot_y",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Replicate `p` into `segments` slices about `pivot`, rotated by `spin_cycles`, with
/// every odd slice mirrored when `reflect`. Returns the `segments · n` positions
/// (slice-major: all of slice 0, then slice 1, …). A pure function.
fn kaleidoscope(
    p: &[[f32; 2]],
    segments: usize,
    reflect: bool,
    pivot: [f32; 2],
    spin_cycles: f32,
) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(segments * p.len());
    for s in 0..segments {
        let (c, sn) = cos_sin_cycles(s as f32 / segments.max(1) as f32 + spin_cycles);
        let mirror = reflect && s % 2 == 1;
        for q in p {
            let lx = q[0] - pivot[0];
            // Odd slices are mirrored across the local x-axis so neighbours meet as
            // mirror images (the dihedral fold).
            let ly = if mirror {
                -(q[1] - pivot[1])
            } else {
                q[1] - pivot[1]
            };
            out.push([lx * c - ly * sn + pivot[0], lx * sn + ly * c + pivot[1]]);
        }
    }
    out
}

/// Duplicate a column into `segments` copies (`[a, b] → [a, b, a, b, …]`).
fn dup_n(col: &Column, segments: usize) -> Column {
    fn rep<T: Clone>(v: &[T], n: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(v.len() * n);
        for _ in 0..n {
            out.extend_from_slice(v);
        }
        out
    }
    match col {
        Column::Scalar(v) => Column::Scalar(rep(v, segments)),
        Column::Vec2(v) => Column::Vec2(rep(v, segments)),
        Column::Vec3(v) => Column::Vec3(rep(v, segments)),
        Column::Vec4(v) => Column::Vec4(rep(v, segments)),
    }
}

struct MotionKaleidoscope;

impl NodeOp for MotionKaleidoscope {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let segments = (ctx.param("segments").round() as i64).clamp(1, MAX_SEGMENTS) as usize;
        let reflect = ctx.param("reflect").round() as i64 != 0;
        let pivot = [ctx.param("pivot_x"), ctx.param("pivot_y")];
        let spin = match ctx.input(1).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let positions = kaleidoscope(&p, segments, reflect, pivot, spin / 360.0);
        // Every column is duplicated onto each slice; only `P` is transformed.
        let mut out = Stream::new(positions.len());
        for (name, col) in input.columns() {
            if name == "P" {
                continue;
            }
            out.set(name.clone(), dup_n(col, segments));
        }
        out.set("P", Column::Vec2(positions));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionKaleidoscope))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Kaleidoscope",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "segments",
        label: "Segments",
        min: 1.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "reflect",
        label: "Reflect",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Rotational", "Mirrored"],
        },
    },
    ParamUiHint {
        param: "pivot_x",
        label: "Pivot X",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pivot_y",
        label: "Pivot Y",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    const O: [f32; 2] = [0.0, 0.0];

    /// `segments` multiplies the element count exactly.
    #[test]
    fn segments_multiply_the_count() {
        let p = vec![[1.0, 0.0], [1.5, 0.3], [0.8, -0.2]];
        assert_eq!(kaleidoscope(&p, 6, false, O, 0.0).len(), 18);
        assert_eq!(
            kaleidoscope(&p, 1, false, O, 0.0).len(),
            3,
            "1 slice = passthrough"
        );
    }

    /// Rotational symmetry (reflect off): a source point on +x is replicated at each
    /// `1/segments` turn. FALSIFIED if the copies landed anywhere but the ring.
    #[test]
    fn rotational_copies_sit_on_the_ring() {
        let p = vec![[2.0, 0.0]];
        let out = kaleidoscope(&p, 4, false, O, 0.0);
        // slice 0 → +x, 1 → +y, 2 → −x, 3 → −y (quarter turns).
        assert!(
            (out[0][0] - 2.0).abs() < 1e-3 && out[0][1].abs() < 1e-3,
            "0: {:?}",
            out[0]
        );
        assert!(
            out[1][0].abs() < 1e-2 && (out[1][1] - 2.0).abs() < 1e-2,
            "1: {:?}",
            out[1]
        );
        assert!(
            (out[2][0] + 2.0).abs() < 1e-2 && out[2][1].abs() < 1e-2,
            "2: {:?}",
            out[2]
        );
        assert!(
            out[3][0].abs() < 1e-2 && (out[3][1] + 2.0).abs() < 1e-2,
            "3: {:?}",
            out[3]
        );
    }

    /// Mirrored symmetry (reflect on): the odd slice is the source *mirrored* before it
    /// rotates, so it differs from the plain rotational copy. FALSIFIED if `reflect`
    /// were ignored (odd slice identical to the rotational one).
    #[test]
    fn reflect_mirrors_alternate_slices() {
        let p = vec![[1.0, 0.5]];
        let plain = kaleidoscope(&p, 4, false, O, 0.0);
        let mirrored = kaleidoscope(&p, 4, true, O, 0.0);
        assert_eq!(plain[0], mirrored[0], "even slice 0 identical");
        // Slice 1 rotated by 90°: plain (1,0.5)→(−0.5,1); mirrored reflects y first
        // (1,−0.5)→(0.5,1). The x-sign flips.
        assert!(
            (plain[1][0] + 0.5).abs() < 1e-2,
            "plain slice 1 x=−0.5: {:?}",
            plain[1]
        );
        assert!(
            (mirrored[1][0] - 0.5).abs() < 1e-2,
            "mirrored slice 1 x=+0.5: {:?}",
            mirrored[1]
        );
    }

    /// `spin` rotates the whole pattern: a quarter-turn spin sends a +x source to +y.
    #[test]
    fn spin_rotates_the_pattern() {
        let p = vec![[2.0, 0.0]];
        let out = kaleidoscope(&p, 3, false, O, 0.25); // +90°
        assert!(
            out[0][0].abs() < 1e-2 && (out[0][1] - 2.0).abs() < 1e-2,
            "spun to +y: {:?}",
            out[0]
        );
    }

    /// A pivot off the origin is the fixed point: a source *at* the pivot stays put
    /// under every slice.
    #[test]
    fn the_pivot_is_the_fixed_point() {
        let piv = [3.0, -1.0];
        let out = kaleidoscope(&[piv], 5, true, piv, 0.13);
        for q in &out {
            assert!(
                (q[0] - piv[0]).abs() < 1e-4 && (q[1] - piv[1]).abs() < 1e-4,
                "fixed: {q:?}"
            );
        }
    }

    /// Deterministic + cooks through the registry: `P` fans out to `segments · n` and
    /// every other column is duplicated to match.
    #[test]
    fn registers_and_folds_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.kaleidoscope.test.src"),
            name: "motion.kaleidoscope.test.src",
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
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[1.0, 0.0], [2.0, 0.5]]))
                        .with("size", Column::Vec2(vec![[0.3, 0.3], [0.3, 0.3]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionKaleidoscope),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.kaleidoscope.test.src");
        let k = g.add_node("motion.kaleidoscope");
        g.set_param(k, "segments", 6.0);
        g.connect(Edge {
            from: (src, 0),
            to: (k, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, k, 0.0).unwrap();
        let s = out[0].as_stream();
        assert_eq!(s.count(), 12, "2 elements × 6 slices");
        match s.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 12, "size duplicated per slice"),
            _ => panic!("size"),
        }
    }
}
