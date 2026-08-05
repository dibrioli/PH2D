//! ADR-0154 — the shell half of `source.shape`: **build each shape's `VecPath`
//! from its graph node's params, publish it into the cook under the content key
//! the node reads, and draw the resulting instances as LIVE GPU vector.**
//!
//! A node is handed only its params, its inputs and the playhead — it cannot reach
//! the vector library or the GPU (the property that lets the cook memoize and
//! replay bit-exactly). So the shell is where a shape *descriptor* becomes
//! *geometry*: [`publish`] scans every `source.shape` node, reads its params
//! through the SAME door the node reads ([`ShapeParams::read`] over
//! override-else-manifest-default — and the node has no inputs, so its `ctx.param`
//! has no *driven* layer to disagree with), builds the `VecPath` once, interns it
//! under [`shape_key`], and sets a one-row instance stream `(P, geometry_id)`
//! external the node's `eval` clones. Identical descriptors share ONE `VecPath`
//! (the store is content-addressed), so a `motion.duplicator` stamping 10k stars
//! interns once. [`encode`] draws each cooked instance into the shared vector
//! scene, so a shape composites behind the chrome and over the sprites (Fase 1).

use std::collections::BTreeMap;

use ph2d_eval_motion::VectorInstance;
use ph2d_node_motion_shape::{MANIFEST, ShapeKind, ShapeParams, shape_key};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_vec_scene::{
    VecPath, ellipse, gear, heart, regular_polygon_rounded, rounded_rect, star_rounded,
};
use ph2d_vector::{Affine, VectorScene};

use crate::motion_state::MotionState;

/// A content-addressed cache of shape geometry: a shape's `VecPath` is interned
/// under its [`shape_key`], and an instance carries the resulting handle in its
/// `geometry_id` column (`index + 1`, so 0 stays "no geometry" — the byte-identical
/// fallback for every stream without the column). Kept across frames, so a static
/// shape builds ONCE; the only growth is one entry per distinct descriptor an
/// artist visits (an animated slider re-interns each value — named, bounded by the
/// session's distinct shapes, the voronoi-cook precedent).
#[derive(Default)]
pub(crate) struct VecPathStore {
    by_handle: Vec<VecPath>,          // index = handle - 1
    handle_of: BTreeMap<String, u32>, // key -> handle
}

impl VecPathStore {
    /// The `VecPath` for a `geometry_id` handle, or `None` for 0 / out of range.
    pub(crate) fn get(&self, handle: u32) -> Option<&VecPath> {
        (handle >= 1)
            .then(|| self.by_handle.get((handle - 1) as usize))
            .flatten()
    }

    /// Intern a shape under its content key, building it once. Returns the handle
    /// (`>= 1`). Identical keys share the stored `VecPath`.
    fn intern(&mut self, key: &str, build: impl FnOnce() -> VecPath) -> u32 {
        if let Some(&h) = self.handle_of.get(key) {
            return h;
        }
        self.by_handle.push(build());
        let h = self.by_handle.len() as u32; // index + 1
        self.handle_of.insert(key.to_owned(), h);
        h
    }

    /// Store a `VecPath` with NO content key, returning a fresh handle (`>= 1`).
    /// The keyed [`intern`](Self::intern) dedups by descriptor for `source.shape`
    /// primitives; a `source.object` DOCUMENT vector has no descriptor string, and
    /// its own content-cache ([`crate::motion_object_bake::ObjectBake`], keyed by
    /// `VecPathId` + content) already decides WHEN to re-store, so this just parks
    /// the current geometry and hands back the handle the membrane emits as
    /// `geometry_id`. One entry per content CHANGE (a static object stores once);
    /// the growth is the store's named, session-bounded trade (like an animated
    /// shape slider re-interning each value).
    pub(crate) fn push(&mut self, path: VecPath) -> u32 {
        self.by_handle.push(path);
        self.by_handle.len() as u32 // index + 1
    }
}

/// The manifest default for a param NAME — the fallback the node's `ctx.param`
/// takes when there is no override (and, for `source.shape`, no driven layer).
pub(crate) fn manifest_default(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.default)
        .unwrap_or(0.0)
}

/// Read a shape descriptor exactly as the node's `eval` does — override-else-
/// manifest-default, the SAME door ([`ShapeParams::read`]). `source.shape` has no
/// inputs, so `ctx.param` has no *driven* layer this could disagree with; that is
/// what makes the shell's key and the node's key the same string (gated).
pub(crate) fn read_params(overrides: Option<&BTreeMap<String, f32>>) -> ShapeParams {
    ShapeParams::read(|name| {
        overrides
            .and_then(|m| m.get(name).copied())
            .unwrap_or_else(|| manifest_default(name))
    })
}

/// Build the `VecPath` for a shape descriptor (ADR-0154). World-unit geometry,
/// centred at the origin; the instance transform places, rotates and scales it.
/// `corner` is absolute world units. Eight FILLABLE closed shapes (the draw entry
/// fills; open curves like arc/spiral are follow-ups).
pub(crate) fn build_shape_path(p: &ShapeParams) -> VecPath {
    let s = f64::from(p.size.max(0.01));
    let ry = s * f64::from(p.aspect.clamp(0.05, 20.0));
    let sides = p.sides.clamp(3, 32);
    // `corner` is a FRACTION of the size → a world-unit radius that scales.
    let corner = f64::from(p.corner.clamp(0.0, 1.0)) * s;
    match p.kind {
        ShapeKind::Circle => ellipse([0.0, 0.0], s, s),
        ShapeKind::Ellipse => ellipse([0.0, 0.0], s, ry),
        ShapeKind::Square => rounded_rect([-s, -s], [s, s], corner.min(s)),
        ShapeKind::Rectangle => rounded_rect([-s, -ry], [s, ry], corner.min(s.min(ry))),
        ShapeKind::Polygon => regular_polygon_rounded([0.0, 0.0], s, s, sides, corner),
        ShapeKind::Star => {
            let ratio = f64::from(p.star_depth.clamp(0.05, 0.95));
            star_rounded([0.0, 0.0], s, s, sides, ratio, corner, corner)
        }
        ShapeKind::Heart => heart([-s, -s], [s, s], f64::from(p.cleft.clamp(0.02, 0.45))),
        ShapeKind::Gear => {
            // `sides` = teeth; `tooth_depth` = fraction of radius; `hole` = centre bore.
            let depth = f64::from(p.tooth_depth.clamp(0.05, 0.6));
            let hole = f64::from(p.hole.clamp(0.0, 1.0));
            gear([-s, -s], [s, s], f64::from(sides), depth, hole)
        }
    }
}

/// Publish every `source.shape` node's geometry into the cook (ADR-0154).
///
/// ⚠️ **Called from the bridge POST-drain, pre-cook** — after `apply_graph_intents`
/// has applied this frame's param edits and before the pump cooks. Publishing it
/// earlier (before the drain) would set the PRE-edit key while the cook reads the
/// POST-edit key ⇒ the node clones an empty external for one frame ⇒ the shape
/// vanishes = **flicker on edit**. It only ADDS (the drawn-curve publish already
/// ran `clear_externals`), under the `shape:` key namespace the node reads.
pub(crate) fn publish(motion: &mut MotionState) {
    // Collect the (key, params) jobs first so the graph borrow drops before we
    // mutate the store and the cook (three disjoint fields of `MotionState`).
    let graph = &motion.doc.graph;
    let jobs: Vec<(String, ShapeParams)> = graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == MANIFEST.name)
        .map(|n| {
            let p = read_params(graph.node_param_overrides(n.id));
            (shape_key(&p), p)
        })
        .collect();
    for (key, p) in jobs {
        let handle = motion.shape_store.intern(&key, || build_shape_path(&p));
        // One-row instance at the origin, carrying the geometry handle. The
        // duplicator/deformers move, rotate, scale and tint it downstream; a bare
        // shape is white at the origin. `size`/`rot`/`tint` are absent ⇒ their
        // stream defaults (unit / 0 / white).
        let stream = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("geometry_id", Column::Scalar(vec![handle as f32]));
        motion.pump.cook.set_external(key, stream);
    }
}

/// Draw each cooked shape instance into the shared vector scene (ADR-0154). The
/// instance's `geometry_id` names a stored `VecPath`; its world pose (`P`, `basis`,
/// `size`) composes with the world→screen `cam` into the draw transform, and its
/// `tint` paints it — so one stored path serves N differently-tinted copies.
pub(crate) fn encode(
    insts: &[VectorInstance],
    store: &VecPathStore,
    cam: Affine,
    scene: &mut VectorScene,
) {
    for inst in insts {
        let Some(path) = store.get(inst.geometry_id) else {
            continue; // a handle with no stored geometry (a forward cook) draws nothing
        };
        let [b0, b1, b2, b3] = inst.basis;
        let [sx, sy] = inst.size;
        let [px, py] = inst.world_pos;
        // world pose = translate(P) · R(basis) · scale(size); kurbo Affine coeffs
        // [a,b,c,d,e,f] = matrix [[a,c,e],[b,d,f]] ⇒ linear = R·S, translation = P.
        let pose = Affine::new([
            f64::from(b0 * sx),
            f64::from(b1 * sx),
            f64::from(b2 * sy),
            f64::from(b3 * sy),
            f64::from(px),
            f64::from(py),
        ]);
        ph2d_vec_render::draw_shape_instance(path, cam * pose, inst.tint, scene);
    }
}

#[cfg(test)]
#[path = "motion_shape_gen_tests.rs"]
mod tests;
