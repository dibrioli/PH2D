#![forbid(unsafe_code)]
//! `rig.skin_deformer` — **Linear Blend Skinning**: wrap a cloud of points around a
//! skeleton so they deform when it moves. The flesh on the bones. Motion Nodes M4, doc 42.
//!
//! LBS (a.k.a. *skeleton subspace deformation*, the "smooth skin" of Maya, the Armature
//! modifier of Blender, the vertex skinning of every game engine) is the industry standard,
//! and it is one line:
//!
//! ```text
//! p' = Σ_j  w_j · ( R_j · (p − rest_origin_j) + posed_origin_j )
//! ```
//!
//! Each bone `j` carries a rest frame and a posed frame; a point is moved by every bone's
//! frame change, blended by its weights. The whole art is in the weights.
//!
//! ## Three inputs, because a skin needs a BIND pose
//!
//! `in` (the points) · **`rest`** (the skeleton as authored) · **`posed`** (the same
//! skeleton after the solvers ran). A skin is meaningless without both: the deformation is
//! the DIFFERENCE between them. So the graph says it out loud —
//!
//! ```text
//! skeleton ─┬─────────────────────────────> rest
//!           └─> fabrik(target) ───────────> posed
//! grid ─────────────────────────────────────> in
//! ```
//!
//! — and the bind pose is a wire you can see, not a hidden snapshot taken at some moment
//! you have to remember.
//!
//! ## The weights: envelopes (Blender's auto-bind), not bone heat
//!
//! `w_j ∝ 1 / (distance from the point to bone j's rest SEGMENT)^falloff`, normalised.
//! Distance to the segment, not to the joint — otherwise a point beside the middle of a
//! long bone would be captured by whichever end happened to be nearer, and the skin would
//! crease there. (Blender's other option, *bone heat*, solves a Laplacian over the mesh
//! surface; there is no mesh here, only points, so there is no surface to diffuse over.)
//!
//! `falloff` is the one dial: low = a soft, gooey skin that many bones share; high = a
//! stiff one where each point belongs to its nearest bone. `MAX_INFLUENCE` guards against
//! a point sitting exactly ON a bone (a zero distance is an infinite weight).
//!
//! **Identity when nothing moved**: if `posed` is the same as `rest`, every frame change is
//! the identity and the points come out **exactly** where they went in — the doc-39 rule
//! (a node at its default must not move a single element), and here it also means "no
//! skeleton wired, no deformation".
//!
//! LBS's known artefact — the **candy-wrapper** collapse at large twists — is inherent to
//! the linear blend of rotations and is why film uses dual quaternions. In 2D, with the
//! bend angles a hose or a limb actually reaches, it does not show. `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

// The `fk` leaf is carried BYTE-IDENTICAL by every `rig.*` crate (that is the point of a
// copied leaf: it cannot drift). This node only reads the skeleton, so the parts that write
// a pose go unused here.
#[allow(dead_code)]
mod fk;
mod trig;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// A full turn in the degrees the columns speak.
const DEGREES_PER_TURN: f32 = 360.0;

/// The largest weight one bone may claim before normalisation — the guard for a point
/// sitting exactly ON a bone, where the inverse distance would be infinite.
const MAX_INFLUENCE: f32 = 1.0e4;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("rig.skin_deformer"),
    name: "rig.skin_deformer",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The bind pose: the skeleton as authored.
        PortSpec {
            name: "rest",
            ty: INST_VEC2,
        },
        // The same skeleton, after the solvers.
        PortSpec {
            name: "posed",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "falloff",
        default: 3.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// One bone's rest and posed frames — the pair whose DIFFERENCE is the deformation.
struct Bone {
    rest_origin: [f32; 2],
    rest_tip: [f32; 2],
    posed_origin: [f32; 2],
    /// The rotation the bone underwent, as `(cos, sin)`, already normalised.
    turn: [f32; 2],
}

/// The bones shared by the `rest` and `posed` skeletons (same topology, or nothing).
fn bones(rest: &Stream, posed: &Stream) -> Vec<Bone> {
    let n = rest.count();
    if n == 0 || posed.count() != n {
        return Vec::new();
    }
    let parent = fk::scalars(rest, fk::PARENT, -1.0, n);
    let (rp, pp) = (fk::positions(rest), fk::positions(posed));
    let (rw, pw) = (
        fk::scalars(rest, fk::WROT, 0.0, n),
        fk::scalars(posed, fk::WROT, 0.0, n),
    );

    (0..n)
        .filter_map(|i| {
            let pi = parent[i];
            let j = (pi >= 0.0 && pi.is_finite() && (pi as usize) < i).then_some(pi as usize)?;
            let (cos, sin) = trig::cos_sin_cycles((pw[i] - rw[i]) / DEGREES_PER_TURN);
            // Normalise: an un-normalised parabolic pair would scale the skin by ~0.1 %
            // per bone, which reads as the flesh breathing.
            let inv = 1.0 / (cos * cos + sin * sin).sqrt();
            Some(Bone {
                rest_origin: rp[j],
                rest_tip: rp[i],
                posed_origin: pp[j],
                turn: [cos * inv, sin * inv],
            })
        })
        .collect()
}

/// Distance from `p` to the segment `a → b` — to the SEGMENT, so a point beside the middle
/// of a long bone belongs to that bone, not to whichever end is nearer.
fn dist_to_bone(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if len2 > f32::EPSILON {
        ((ap[0] * ab[0] + ap[1] * ab[1]) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let d = [ap[0] - t * ab[0], ap[1] - t * ab[1]];
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// The envelope weight of every bone for one point, normalised to sum to 1.
fn weights(p: [f32; 2], bones: &[Bone], falloff: f32) -> Vec<f32> {
    let raw: Vec<f32> = bones
        .iter()
        .map(|b| {
            let d = dist_to_bone(p, b.rest_origin, b.rest_tip);
            if d <= f32::EPSILON {
                return MAX_INFLUENCE; // the point is ON the bone
            }
            let mut w = 1.0;
            // `falloff` as a repeated divide: a `powf` is a transcendental (HR-5).
            for _ in 0..falloff.clamp(1.0, 8.0).round() as usize {
                w /= d;
            }
            w.min(MAX_INFLUENCE)
        })
        .collect();
    let total: f32 = raw.iter().sum();
    if total > f32::EPSILON {
        raw.iter().map(|w| w / total).collect()
    } else {
        // Unreachably far from every bone (or a degenerate skeleton): share equally
        // rather than divide by zero.
        vec![1.0 / bones.len().max(1) as f32; bones.len()]
    }
}

/// One evaluation: move every point by the blended frame change of the bones around it.
fn skin(input: &Stream, rest: &Stream, posed: &Stream, falloff: f32) -> Stream {
    let bones = bones(rest, posed);
    if bones.is_empty() {
        return input.clone(); // no skeleton wired → an honest no-op
    }
    let pts = fk::positions(input);
    let moved: Vec<[f32; 2]> = pts
        .iter()
        .map(|&p| {
            let w = weights(p, &bones, falloff);
            let mut out = [0.0f32, 0.0];
            for (bone, wj) in bones.iter().zip(&w) {
                // The point, in the bone's rest frame, replayed in its posed frame.
                let v = [p[0] - bone.rest_origin[0], p[1] - bone.rest_origin[1]];
                let (c, s) = (bone.turn[0], bone.turn[1]);
                let r = [v[0] * c - v[1] * s, v[0] * s + v[1] * c];
                out[0] += wj * (bone.posed_origin[0] + r[0]);
                out[1] += wj * (bone.posed_origin[1] + r[1]);
            }
            out
        })
        .collect();

    let mut out = Stream::new(input.count());
    for (name, col) in input.columns() {
        if name != "P" {
            out.set(name.clone(), col.clone());
        }
    }
    out.set("P", Column::Vec2(moved));
    out
}

struct RigSkinDeformer;

impl NodeOp for RigSkinDeformer {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let falloff = ctx.param("falloff");
        let out = skin(ctx.input(0), ctx.input(1), ctx.input(2), falloff);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(RigSkinDeformer))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Skin",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "falloff",
    label: "Stiffness",
    min: 1.0,
    max: 8.0,
    step: 1.0,
    widget: ParamWidget::IntSlider,
}];

#[cfg(test)]
mod tests {
    use super::*;

    /// A straight chain of `n` joints along `+x`, one unit apart.
    fn chain(n: usize) -> Stream {
        fk::resolve(
            &Stream::new(n)
                .with(
                    fk::PARENT,
                    Column::Scalar((0..n).map(|i| i as f32 - 1.0).collect()),
                )
                .with(
                    fk::LEN,
                    Column::Scalar((0..n).map(|i| if i == 0 { 0.0 } else { 1.0 }).collect()),
                )
                .with(fk::ROT, Column::Scalar(vec![0.0; n]))
                .with("P", Column::Vec2(vec![[0.0, 0.0]; n])),
        )
    }

    /// The same chain, rotated bodily about its root by `deg` (every local angle untouched
    /// except the root's, which is the limb's aim).
    fn turned(n: usize, deg: f32) -> Stream {
        let mut rot = vec![0.0; n];
        rot[0] = deg;
        fk::resolve(&chain(n).with(fk::ROT, Column::Scalar(rot)))
    }

    fn cloud(pts: Vec<[f32; 2]>) -> Stream {
        Stream::new(pts.len()).with("P", Column::Vec2(pts))
    }

    fn ps(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
        let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
        (dx * dx + dy * dy).sqrt()
    }

    /// **A rigid rotation of the skeleton rotates the skin RIGIDLY** — the sharpest test
    /// LBS can be given, because every weighting scheme gets the same answer only if the
    /// frames are being composed correctly. Rotate the whole chain 90° about its root, and
    /// every point must land exactly where that rotation would put it, whatever its
    /// weights, because every bone's frame change is the SAME rotation.
    ///
    /// FALSIFIED by blending the ANGLES instead of the frames, by forgetting the rest
    /// origin (points would fly to the bone), or by rotating about the wrong pivot.
    #[test]
    fn a_rigid_turn_of_the_skeleton_turns_the_skin_rigidly() {
        let pts = vec![[0.5, 0.3], [2.0, -0.4], [3.9, 0.1], [1.0, 1.2]];
        let out = ps(&skin(&cloud(pts.clone()), &chain(5), &turned(5, 90.0), 3.0));
        for (p, q) in pts.iter().zip(&out) {
            // 90° about the origin: (x, y) -> (-y, x).
            let want = [-p[1], p[0]];
            assert!(dist(*q, want) < 0.02, "{p:?} -> {q:?}, want {want:?}");
        }
    }

    /// **The identity**: a `posed` skeleton equal to `rest` moves nothing at all (doc 39),
    /// and neither does an unwired skeleton. A skin that drifts when the rig is at rest is
    /// the bug that ruins every bind.
    #[test]
    fn a_rig_at_rest_moves_no_point() {
        let pts = vec![[0.5, 0.3], [2.0, -0.4], [3.9, 0.1]];
        let out = ps(&skin(&cloud(pts.clone()), &chain(5), &chain(5), 3.0));
        for (p, q) in pts.iter().zip(&out) {
            assert!(dist(*p, *q) < 1e-4, "{p:?} drifted to {q:?}");
        }
        // No skeleton at all, and a mismatched pair: both are no-ops, not garbage.
        assert_eq!(
            ps(&skin(
                &cloud(pts.clone()),
                &Stream::new(0),
                &Stream::new(0),
                3.0
            )),
            pts
        );
        assert_eq!(
            ps(&skin(&cloud(pts.clone()), &chain(5), &chain(4), 3.0)),
            pts
        );
    }

    /// The skin FOLLOWS the bone it is nearest: bend only the far half of the chain, and
    /// the points out there travel while the ones by the root stay home. This is what the
    /// envelope weights buy.
    #[test]
    fn each_point_follows_the_bone_it_lies_along() {
        // Bend joint 3 by 90°: bones 1-2 are untouched, bones 3-4 swing up.
        let mut rot = vec![0.0; 5];
        rot[3] = 90.0;
        let posed = fk::resolve(&chain(5).with(fk::ROT, Column::Scalar(rot)));

        let near_root = [0.5, 0.1];
        let far_out = [3.8, 0.1];
        let out = ps(&skin(
            &cloud(vec![near_root, far_out]),
            &chain(5),
            &posed,
            4.0,
        ));
        assert!(
            dist(out[0], near_root) < 0.05,
            "the point by the root stayed: {:?}",
            out[0]
        );
        assert!(
            dist(out[1], far_out) > 1.0,
            "the point out on the bent end travelled: {:?}",
            out[1]
        );
    }

    /// Distance is to the SEGMENT, not to the joints — the difference that stops a skin
    /// creasing beside a long bone.
    #[test]
    fn the_weight_measures_distance_to_the_bone_not_to_its_ends() {
        // A point beside the middle of the bone (0,0)->(4,0) is 1 away from the BONE and
        // ~2.2 away from either end.
        assert!((dist_to_bone([2.0, 1.0], [0.0, 0.0], [4.0, 0.0]) - 1.0).abs() < 1e-4);
        // Past the end, it measures to the end.
        assert!((dist_to_bone([6.0, 0.0], [0.0, 0.0], [4.0, 0.0]) - 2.0).abs() < 1e-4);
        // A point ON the bone gets a bounded weight, never an infinite one.
        let b = bones(&chain(3), &chain(3));
        let w = weights([1.0, 0.0], &b, 3.0);
        assert!(w.iter().all(|x| x.is_finite()));
        assert!(
            (w.iter().sum::<f32>() - 1.0).abs() < 1e-4,
            "weights are normalised"
        );
    }
}
