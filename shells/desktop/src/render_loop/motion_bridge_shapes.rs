//! **Publishing the drawn shapes into the cook** (doc 65) — the shell half of `motion.path`.
//!
//! Every vector path that has a **name** goes into the `Cook`'s external table under that name, as a
//! polyline in **world space**. The node reads it by name (`ctx.external("Track")`), and the memo
//! sees the curve's revision, so dragging the shape re-cooks the set that walks it.
//!
//! ## Three decisions, all of them about where a boundary is
//!
//! **1. The graph gets a POLYLINE, not a Bézier.** Flattening is the shell's job because the shell
//! is the one that already owns the curve — `ph2d-nodegraph` is a leaf substrate with zero deps, and
//! teaching it about `VecPath` would make it the vector module's downstream. The node walks arc-
//! length, and arc-length on a polyline is `sqrt` and addition; on a cubic it is an integral. The
//! cheap thing is also the honest one.
//!
//! **2. The name is the artist's, and it is the WHOLE reference.** A shape with no name is not
//! published — there is nothing to type into the node. Draw it, name it in the Hierarchy, and it is
//! visible to the graph. Rename it and the node's `Shape` field stops matching, which is exactly
//! what renaming a thing you referred to by name means. (An id would have been a second identity to
//! keep in sync with the one the artist can see. There is no picker to build and nowhere for the two
//! to disagree.)
//!
//! **3. Two shapes with the same name is the artist's business, not ours.** The last one in scene
//! order wins, deterministically. Refusing to publish either would punish a typo with silence.
//!
//! The world affine matters: the geometry a path stores is LOCAL and its pose lives on the ECS
//! `Transform` (ADR-0111). A polyline published in local space would put the instances where the
//! shape *was authored*, not where it *is* — and it would look right until the artist moved it.

use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_vec_scene::{VecScene, VecXforms, xform_of};

/// How finely a curve is flattened, in world units. A segment is split until its chord is within
/// this of the true curve — which is what "flatten to tolerance" means everywhere else in this repo
/// (`paint/curve_offset.rs` does the same thing for the same reason).
const FLATTEN_TOL: f64 = 0.01;

/// The deepest a segment subdivides. A cubic needs ~2^5 pieces for the tolerance above at any sane
/// scale, and the cap is what stops a degenerate control net (a cusp, a NaN from a corrupt file)
/// from subdividing until the heap gives out.
const MAX_DEPTH: u32 = 5;

/// Is this cubic flat enough to be its own chord? The distance from each control point to the chord
/// — the standard flatness test, and the one the offset code already uses.
fn flat_enough(p: [[f64; 2]; 4]) -> bool {
    let (dx, dy) = (p[3][0] - p[0][0], p[3][1] - p[0][1]);
    let len = (dx * dx + dy * dy).sqrt();
    if len < f64::EPSILON {
        // A degenerate chord (the ends coincide): flat only if the handles are on top of them too,
        // or a loop would be flattened away to a point.
        return [p[1], p[2]]
            .iter()
            .all(|q| (q[0] - p[0][0]).hypot(q[1] - p[0][1]) < FLATTEN_TOL);
    }
    [p[1], p[2]].iter().all(|q| {
        let d = ((q[0] - p[0][0]) * dy - (q[1] - p[0][1]) * dx).abs() / len;
        d < FLATTEN_TOL
    })
}

/// Append the flattened cubic (excluding its first point — the caller already pushed it).
fn flatten(p: [[f64; 2]; 4], depth: u32, out: &mut Vec<[f32; 2]>) {
    if depth >= MAX_DEPTH || flat_enough(p) {
        out.push([p[3][0] as f32, p[3][1] as f32]);
        return;
    }
    // Split at the midpoint (de Casteljau), and recurse on both halves.
    let mid = |a: [f64; 2], b: [f64; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let (ab, bc, cd) = (mid(p[0], p[1]), mid(p[1], p[2]), mid(p[2], p[3]));
    let (abc, bcd) = (mid(ab, bc), mid(bc, cd));
    let m = mid(abc, bcd);
    flatten([p[0], ab, abc, m], depth + 1, out);
    flatten([m, bcd, cd, p[3]], depth + 1, out);
}

/// One path, flattened into a **world-space** polyline.
///
/// It walks the **COOKED** geometry (`VecPath::cooked()`, ADR-0121) — the same thing the renderer,
/// the hit-test and the boolean walk. The document keeps the sharp corner and the radius; the world
/// consumes the rounded one, and a node that walked the *authored* verts would put instances on a
/// corner the artist cannot see.
pub(super) fn polyline(
    scene: &VecScene,
    xforms: &VecXforms,
    id: ph2d_vec_scene::VecPathId,
) -> Vec<[f32; 2]> {
    let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
        return Vec::new();
    };
    let cooked = path.cooked();
    if cooked.verts.len() < 2 {
        return Vec::new();
    }
    let x = xform_of(xforms, id);
    let w = |p: [f64; 2]| x.apply(p);

    let mut out: Vec<[f32; 2]> = Vec::new();
    let first = w(cooked.verts[0].anchor);
    out.push([first[0] as f32, first[1] as f32]);

    let n = cooked.verts.len();
    let last = if cooked.closed { n } else { n - 1 };
    for i in 0..last {
        let a = &cooked.verts[i];
        let b = &cooked.verts[(i + 1) % n];
        flatten(
            [w(a.anchor), w(a.out_handle), w(b.in_handle), w(b.anchor)],
            0,
            &mut out,
        );
    }
    out
}

/// Publish every **named** shape into the cook, and forget the ones that are gone.
///
/// Called once a frame, before the pump. Republishing is free: the external's revision is a hash of
/// its content, so a shape nobody touched invalidates nothing (`ph2d_nodegraph::external`).
pub(super) fn publish(
    cook: &mut ph2d_nodegraph::cook::Cook,
    sim: &SimWorld,
    scene: &VecScene,
    map: &crate::vec_entities::VecEntityMap,
    xforms: &VecXforms,
) {
    // Clear first: a shape the artist deleted (or un-named) must stop being visible to the graph,
    // and a stale entry would keep a `motion.path` walking a curve that is not on the canvas.
    cook.clear_externals();
    let w = sim.world();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        let Ok(entity) = w.get_entity(e) else {
            continue;
        };
        let Some(name) = entity.get::<Name>().map(|n| n.0.clone()) else {
            continue; // unnamed: there is nothing for the artist to type into the node
        };
        if name.trim().is_empty() || is_reserved(&name) {
            continue;
        }
        let pts = polyline(scene, xforms, id);
        if pts.len() < 2 {
            continue; // not a curve: a single point has no arc to walk
        }
        // Where it IS, on the same channel a sprite uses — so a node asking for the
        // position of "X" never has to know whether X is a drawing or an object.
        let n = pts.len();
        #[expect(
            clippy::cast_precision_loss,
            reason = "a count of polyline points; the mean is a world position"
        )]
        let mean = pts
            .iter()
            .fold([0.0f32, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]])
            .map(|v| v / n as f32);
        cook.set_external(
            ph2d_nodegraph::external::position_of(&name),
            Stream::new(1).with("P", Column::Vec2(vec![mean])),
        );
        // ⚠️ **A GEOMETRIA vai no canal DELA, e é a correção do smoke de
        // 2026-08-12** (*"escolhi a curva mas não funcionou"*). O nome cru é
        // publicado aqui como polilinha **e** logo depois pelo publicador de
        // objetos como a APARÊNCIA da forma assada (uma instância na origem) —
        // os dois dizem, cada um no seu comentário, que *"o último a escrever
        // num nome vence"*. O que se perdia era a resposta que o `motion.path` e
        // o `motion.spline_wrap` pedem: eles achavam um stream de UM ponto, não
        // achavam arco, e caíam no fallback **em silêncio**, com o painel
        // mostrando a forma escolhida.
        //
        // A cura é a do irmão `position_of`, uma pergunta adiante: *a aparência,
        // a posição e a FORMA de X são três perguntas*, e uma coluna com o nome
        // certo carregando a resposta de outra é o jeito mais quieto de errar.
        cook.set_external(
            ph2d_nodegraph::external::curve_of(&name),
            Stream::new(n).with("P", Column::Vec2(pts.clone())),
        );
        // O nome cru continua publicado como antes: quem não foi assado como
        // objeto (o frame em que a forma nasce) ainda o encontra, e nada que já
        // lia daqui muda de comportamento.
        cook.set_external(name, Stream::new(n).with("P", Column::Vec2(pts)));
    }
}

/// **The `$` namespace belongs to the EDITOR, never to an artist's object.**
///
/// Every external is keyed by a name the artist typed, and the editor now publishes
/// values of its own into the same table (the cursor, and whatever follows it). With
/// one flat namespace, the day somebody names a sprite `$cursor` their sprite silently
/// BECOMES the mouse — no error, no warning, and a `motion.look_at` aiming at a thing
/// that moves when the artist moves the pointer.
///
/// So the shell refuses to publish an artist name inside the namespace instead of
/// hoping the collision never happens. Refusing is visible (the object stops appearing
/// in the picker); the collision is not.
pub(super) fn is_reserved(name: &str) -> bool {
    ph2d_nodegraph::external::is_reserved(name)
}

/// Publish the **cursor** as an external, so a node can aim at the mouse.
///
/// The cursor is not in the graph and cannot be: it is not a document value, it is an
/// editor input that changes every frame. Publishing it into the table the cook already
/// reads is what lets `motion.look_at` name it as a target WITHOUT the node learning
/// what a window or a camera is.
///
/// ⚠️ Runs AFTER `publish` (which clears) and after the objects, for the same reason
/// they run in that order — and it is a one-point stream, so the node's `centroid`
/// reads it as the point it is.
pub(super) fn publish_cursor(
    cook: &mut ph2d_nodegraph::cook::Cook,
    camera: &ph2d_render::Camera2d,
    cursor: (f32, f32),
    split: ph2d_editor::screens::layout::CenterSplit,
    window: ph2d_host::WindowSize,
) {
    // ⚠️ The SCENE window, never the raw one — `scene_camera_window` is the porta única
    // (`field_gizmo`), and its doc names this exact failure: with the Motion tool active
    // the scene renders into the top-left sub-rectangle of the split, so a cursor mapped
    // through the FULL window lands shifted and shrunk. That is the chronic Motion drift
    // of 2026-07-25, and the aim reads as "the node does not follow the mouse" rather
    // than as a projection error, because nothing on screen shows the wrong number.
    //
    // The sub-rectangle is anchored at `(0,0)`, so only the DIMS change: the cursor pixel
    // needs no offset, and off the split this is byte-identical to the window.
    let scene = crate::field_gizmo::scene_camera_window(split, window);
    let [x, y] = camera.screen_to_world(cursor, scene);
    cook.set_external(
        ph2d_nodegraph::external::CURSOR.to_string(),
        Stream::new(1).with("P", Column::Vec2(vec![[x, y]])),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The editor's namespace is refused to artist names, at the publisher.**
    ///
    /// Without this an object called `$cursor` overwrites the mouse — or, depending on
    /// publish order, the mouse overwrites the object — and a `motion.look_at` aims at
    /// whichever won. Nothing errors, nothing warns; the artist sees a sprite that
    /// follows the pointer and has no way to ask why.
    ///
    /// The refusal is the SHELL's, not the node's, because the shell is the only side
    /// that knows a name came from a human.
    #[test]
    fn the_shell_refuses_to_publish_an_artist_name_in_the_reserved_namespace() {
        assert!(is_reserved("$cursor"));
        assert!(is_reserved(" $anything"));
        assert!(!is_reserved("Sun"));
        // And it is the SUBSTRATE's rule, not a second copy: the two answers cannot
        // drift, because there is only one.
        for n in ["$cursor", " $x", "Sun", "", "a$b"] {
            assert_eq!(is_reserved(n), ph2d_nodegraph::external::is_reserved(n));
        }
    }

    /// **The cursor is mapped through the SCENE, not the window.**
    ///
    /// Enio, on the first smoke of the Cursor mode: *"parece não olhar para o mouse"*.
    /// It was not the node — it was this publish projecting through the FULL window
    /// while the Motion tool renders the scene into the top-left sub-rectangle of the
    /// split. A world point then lands in two places, which is the chronic Motion drift
    /// of 2026-07-25, and `field_gizmo::scene_camera_window` exists BECAUSE of it, with
    /// a doc-comment that says every world↔screen mapping of the scene chrome has to go
    /// through it. This publish did not, and the aim reads as a broken node rather than
    /// as a projection error, because nothing on screen shows the wrong number.
    ///
    /// The gate is a RATIO, not a coordinate: under a half-height split the scene camera
    /// sees half the height, so the same pixel maps to a world `y` twice as far from
    /// centre. Asserting the mapping MOVED (and by how much) is the property; asserting
    /// a literal world point would pin the camera's own arithmetic instead.
    #[test]
    fn the_cursor_is_mapped_through_the_scene_viewport_not_the_window() {
        use ph2d_editor::screens::layout::CenterSplit;
        use ph2d_host::WindowSize;

        let win = WindowSize::new(800, 600);
        let camera = ph2d_render::Camera2d::default();
        // A pixel ABOVE centre, so the split's height change is visible in `y`.
        let cursor = (400.0, 150.0);

        let world_of = |split: CenterSplit| -> [f32; 2] {
            let mut cook = ph2d_nodegraph::cook::Cook::new();
            publish_cursor(&mut cook, &camera, cursor, split, win);
            match cook.externals()[ph2d_nodegraph::external::CURSOR]
                .value
                .get("P")
            {
                Some(Column::Vec2(v)) => v[0],
                other => panic!("the cursor must publish a point: {other:?}"),
            }
        };

        let full = world_of(CenterSplit::None);
        let half = world_of(CenterSplit::Horizontal { t: 0.5 });
        // Off the split: byte-identical to the plain window mapping — the control, and
        // the half that proves the door is not silently rescaling everybody.
        assert_eq!(full, camera.screen_to_world(cursor, win));
        // Under the split the SAME pixel means a different world point, and the numbers
        // are measured rather than reasoned: y=150 is a quarter down an 800×600 window
        // (world +2.5, above centre) and the exact CENTRE of the 800×300 the scene gets
        // (world 0). ⚠️ Halving the height moves the viewport's centre, it does not just
        // scale — the first version of this gate asserted "twice as far" and was wrong
        // about the product, not the other way round.
        assert!(
            (full[1] - 2.5).abs() < 1e-4,
            "the full window puts this pixel above centre: {full:?}"
        );
        assert!(
            half[1].abs() < 1e-4,
            "the scene viewport puts the same pixel AT its centre: {half:?}"
        );
        assert!(
            (half[0] - full[0]).abs() < 1e-4,
            "a horizontal split does not change the width: {full:?} -> {half:?}"
        );
        // And it is the DOOR's answer, not a second derivation of it: the defect was
        // projecting through `win` here, which makes these two equal.
        assert_ne!(
            half,
            camera.screen_to_world(cursor, win),
            "mapping through the full window is the drift this gate exists for"
        );
    }
}
