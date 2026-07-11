#![forbid(unsafe_code)]
//! `motion.mirror` — **reflect and duplicate** the layout across an axis: a symmetry /
//! kaleidoscope modifier (Motion Nodes M3, distributions — doc 01 §3 / doc 25). The
//! "Symmetry"/"Mirror" of every 2D/3D package: take a layout and make it symmetric.
//!
//! **Algorithm — an axis reflection through the centroid.** Each element is kept, and a
//! reflected copy is added: for a **vertical** axis, `(x, y) → (2·cx − x, y)`; for a
//! **horizontal** axis, `(x, y) → (x, 2·cy − y)`, where `(cx, cy)` is the layout's
//! centroid. So `count → 2·count`, the two halves mirror-images. Only the **position**
//! `P` is reflected; every other column (`size`, `tint`, `id`, …) is copied onto the
//! twin — a mirror of the *layout*, which is exact for a positional distribution (a
//! moving sim's `vel`/`rot` are duplicated, not flipped). Transcendental-free (HR-5):
//! reflection is arithmetic — no trig, no `sqrt`. `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.mirror"),
    name: "motion.mirror",
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
        // 0 = Vertical axis (reflect x); 1 = Horizontal axis (reflect y).
        ParamSpec {
            name: "axis",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Reflect + duplicate the positions across the axis through their centroid. Returns
/// the `2n` positions (originals then their mirror images).
fn mirror_positions(p: &[[f32; 2]], vertical: bool) -> Vec<[f32; 2]> {
    let n = p.len();
    if n == 0 {
        return Vec::new();
    }
    let mut c = p
        .iter()
        .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    c = [c[0] / n as f32, c[1] / n as f32];
    let mut out = p.to_vec();
    out.extend(p.iter().map(|q| {
        if vertical {
            [2.0 * c[0] - q[0], q[1]]
        } else {
            [q[0], 2.0 * c[1] - q[1]]
        }
    }));
    out
}

struct MotionMirror;

impl NodeOp for MotionMirror {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let vertical = ctx.param("axis").round() as i64 == 0;
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let mirrored = mirror_positions(&p, vertical);
        // Every column is duplicated onto the twin; only `P` is reflected.
        let mut out = Stream::new(mirrored.len());
        for (name, col) in input.columns() {
            if name == "P" {
                continue;
            }
            out.set(name.clone(), dup(col));
        }
        out.set("P", Column::Vec2(mirrored));
        ctx.emit(out);
    }
}

/// Duplicate a column onto its mirror twin (`[a, b] → [a, b, a, b]`).
fn dup(col: &Column) -> Column {
    match col {
        Column::Scalar(v) => Column::Scalar([v.clone(), v.clone()].concat()),
        Column::Vec2(v) => Column::Vec2([v.clone(), v.clone()].concat()),
        Column::Vec3(v) => Column::Vec3([v.clone(), v.clone()].concat()),
        Column::Vec4(v) => Column::Vec4([v.clone(), v.clone()].concat()),
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionMirror))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Mirror",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "axis",
    label: "Axis",
    min: 0.0,
    max: 1.0,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: &["Vertical", "Horizontal"],
    },
}];

#[cfg(test)]
mod tests {
    use super::*;

    /// A vertical mirror doubles the count and reflects each element's x across the
    /// centroid, keeping y. FALSIFIED if the twin were a plain copy (x unchanged).
    #[test]
    fn vertical_mirror_reflects_x_and_doubles() {
        // Two points, centroid x = 1.
        let p = vec![[0.0, 2.0], [2.0, -1.0]];
        let out = mirror_positions(&p, true);
        assert_eq!(out.len(), 4, "count doubled");
        assert_eq!(&out[0..2], &p[..], "originals kept");
        // Reflected across cx=1: (0,2)→(2,2), (2,−1)→(0,−1).
        assert_eq!(out[2], [2.0, 2.0]);
        assert_eq!(out[3], [0.0, -1.0]);
    }

    /// A horizontal mirror reflects y instead.
    #[test]
    fn horizontal_mirror_reflects_y() {
        let p = vec![[1.0, 0.0], [-1.0, 4.0]]; // centroid y = 2
        let out = mirror_positions(&p, false);
        assert_eq!(out[2], [1.0, 4.0]); // (1,0) → (1, 4)
        assert_eq!(out[3], [-1.0, 0.0]); // (−1,4) → (−1, 0)
    }

    /// The reflected set is symmetric: its centroid equals the original's (the mirror
    /// adds no net drift).
    #[test]
    fn the_mirrored_layout_is_symmetric() {
        let p = vec![[0.5, 1.0], [3.0, -2.0], [1.5, 0.5]];
        let out = mirror_positions(&p, true);
        let c = out
            .iter()
            .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
        let c = [c[0] / out.len() as f32, c[1] / out.len() as f32];
        let orig_cx = p.iter().map(|q| q[0]).sum::<f32>() / p.len() as f32;
        assert!((c[0] - orig_cx).abs() < 1e-4, "centroid preserved");
    }

    /// Cooks through the registry: every column is duplicated (length `2n`) and `P`
    /// is reflected.
    #[test]
    fn registers_and_mirrors_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.mirror.test.src"),
            name: "motion.mirror.test.src",
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
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [4.0, 0.0]]))
                        .with("size", Column::Vec2(vec![[0.4, 0.4], [0.4, 0.4]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionMirror),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.mirror.test.src");
        let m = g.add_node("motion.mirror");
        g.connect(Edge {
            from: (src, 0),
            to: (m, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
        let s = out[0].as_stream();
        assert_eq!(s.count(), 4, "doubled");
        match s.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 4, "size duplicated onto the twin"),
            _ => panic!("size"),
        }
    }
}
