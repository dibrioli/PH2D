//! Gates for the Flip bake (doc 86 §2/§6 A3). FILHO of `motion_flip_bake` via
//! `#[path]` (so `use super::*` reaches the private `content_key`/`bake_one`), split
//! off to keep the parent under the shell LOC cap.

use super::*;
use ph2d_flip::{Fill, FlipStroke, Hold, KeyKind, Point, Rgba};

/// A two-layer named Flip object, mirroring the smoke: a filled rectangle (BG) + a
/// square (FG) of half-size `fg_half`. The parameter lets the edit case build a
/// genuinely different geometry.
fn two_layer_object(fg_half: f32) -> (FlipDoc, FlipObjectId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Object");
    let obj = doc.object_mut(oid).expect("object");
    obj.fps = 12.0;
    let bg = obj.add_layer("BG");
    if let Some(d) = obj.insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("bg drawing")
            .strokes
            .push(filled_rect(
                [-0.6, -0.4],
                [0.6, 0.4],
                Rgba::new(0.2, 0.5, 0.9, 1.0),
            ));
    }
    let fg = obj.add_layer("FG");
    if let Some(d) = obj.insert_frame(fg, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("fg drawing")
            .strokes
            .push(filled_rect(
                [-fg_half, -fg_half],
                [fg_half, fg_half],
                Rgba::new(0.95, 0.7, 0.2, 1.0),
            ));
    }
    (doc, oid)
}

fn filled_rect(a: [f32; 2], b: [f32; 2], color: Rgba) -> FlipStroke {
    let mut s = FlipStroke::new();
    for corner in [
        Vec2::new(a[0], a[1]),
        Vec2::new(b[0], a[1]),
        Vec2::new(b[0], b[1]),
        Vec2::new(a[0], b[1]),
    ] {
        s.push_point(Point {
            pos: corner,
            width: 0.04,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s.fill = Some(Fill {
        color,
        opacity: 1.0,
    });
    s
}

fn key_of(doc: &FlipDoc, oid: FlipObjectId, model: Xform) -> u64 {
    let obj = doc.objects().iter().find(|o| o.id == oid).expect("object");
    content_key(obj, &model, obj.frame_at(&Playhead::default()))
}

/// The design decision the cache stands on (doc 86 §2, A3): the tile is the object's
/// composed drawing, bbox-normalized. So a MOVE (object translation only) is a cache
/// HIT — the linear transform, the layer poses and the geometry are unchanged, and
/// the tile does not carry the translation. A ROTATE (linear changes) or an EDIT
/// (geometry changes) re-bakes. A key that folded translation in would re-bake on
/// every drag (A2's rule, in the Flip axis).
#[test]
fn moving_the_object_does_not_rebake_but_rotating_and_editing_do() {
    let (doc, oid) = two_layer_object(0.2);

    let base = key_of(&doc, oid, Xform::IDENTITY);
    // A move: translation only (Xform e,f) — the linear part is unchanged.
    let moved = key_of(&doc, oid, Xform([1.0, 0.0, 0.0, 1.0, 3.7, -2.1]));
    assert_eq!(base, moved, "a move is a cache hit — no re-bake");
    // A 90° rotation: the linear part changes.
    let rotated = key_of(&doc, oid, Xform([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]));
    assert_ne!(base, rotated, "a rotate re-bakes");

    // MEASURED — the LIVE cache's whole point: a STATIC hold (this object has
    // drawings only at frame 0, held forever) must yield ONE content key across a
    // sweep of frames, so it bakes ONCE and does not re-rasterize every frame of
    // playback. A key that hashed the raw frame would re-bake 25 times here.
    let mut ph = Playhead::new(1.0 / 60.0);
    let mut distinct = std::collections::BTreeSet::new();
    for f in 0..25 {
        ph.seek_frame(f, 12.0);
        let obj = doc.objects().iter().find(|o| o.id == oid).expect("object");
        distinct.insert(content_key(obj, &Xform::IDENTITY, obj.frame_at(&ph)));
    }
    assert_eq!(
        distinct.len(),
        1,
        "a static hold bakes ONCE, not per-frame (25 frames -> 1 key)"
    );

    // An edit: a LARGER FG square (different geometry) -> a different content key.
    let (edited, eid) = two_layer_object(0.35);
    let key_e = key_of(&edited, eid, Xform::IDENTITY);
    assert_ne!(base, key_e, "editing the geometry re-bakes");
}

/// **gate-5-flip (doc 86 §6):** the baked tile carries the object's COMPOSED
/// silhouette — an APPEARANCE oracle (coverage + BOTH layers' colours present),
/// not bytes (the bake is GPU). It drives the real Flip raster + compositor +
/// readback on a device, so it also proves the GPU path doesn't panic.
///
/// `#[ignore]`: needs a GPU adapter (none on CI); run on a dev machine with
/// `--ignored`.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn a_baked_flip_object_carries_the_composed_two_layer_silhouette() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let (doc, oid) = two_layer_object(0.2);
    let obj = doc.objects().iter().find(|o| o.id == oid).expect("object");
    let mut bake = FlipObjectBake::default();

    // ⚠️ Warm-up bake, DISCARDED: a fresh process pays cold GPU init + the first
    // pipeline/kernel COMPILE (the walk kernel, the compositor). In the running app
    // the device + pipelines are already warm (the frame pass compiled them), so the
    // representative bake cost is the SECOND one — the first would let the cold path
    // define the product (§0).
    let warm = bake
        .bake_one(
            obj,
            &Xform::IDENTITY,
            obj.frame_at(&Playhead::default()),
            &gpu,
            &mut renderer,
        )
        .expect("the warm-up bake produced a tile");
    renderer.individual_mut().release(warm.0);

    let t0 = std::time::Instant::now();
    let (tid, wsize, _thumb) = bake
        .bake_one(
            obj,
            &Xform::IDENTITY,
            obj.frame_at(&Playhead::default()),
            &gpu,
            &mut renderer,
        )
        .expect("the bake produced a tile");
    let dt = t0.elapsed();

    let (w, h, rgba) = renderer
        .readback_individual(tid)
        .expect("readback the tile");
    let (mut opaque, mut blue, mut orange) = (0usize, 0usize, 0usize);
    for px in rgba.chunks_exact(4) {
        let (r, g, b, a) = (i32::from(px[0]), i32::from(px[1]), i32::from(px[2]), px[3]);
        if a > 128 {
            opaque += 1;
            if b > r + 20 && b > g {
                blue += 1; // BG
            }
            if r > b + 20 && r > g - 20 {
                orange += 1; // FG
            }
        }
    }
    let coverage = opaque as f64 / (w * h).max(1) as f64;
    eprintln!(
        "[flip-bake gate-5] tile {w}x{h} px (world {:.2}x{:.2}) | coverage {:.1}% | \
         BG-blue {blue} px + FG-orange {orange} px | bake {:.2} ms",
        wsize[0],
        wsize[1],
        coverage * 100.0,
        dt.as_secs_f64() * 1000.0
    );
    assert!(
        coverage > 0.5,
        "the composed object fills its bbox (coverage {coverage:.2})"
    );
    assert!(blue > 0, "the BG (blue) layer is present in the tile");
    assert!(
        orange > 0,
        "the FG (orange) layer is present — the two layers COMPOSED"
    );

    renderer.individual_mut().release(tid);
}

/// **The Flip twin of `motion_object_bake::select_present`** (doc 86 §9.6): a Flip object
/// is tiled iff it is NAMED or sits inside a named group — and nothing else, so unnamed
/// canvas art never wastes a tile. The three rows are the whole decision table; two
/// mutations each break a distinct row (drop the group check ⇒ the group child vanishes;
/// drop the `continue` ⇒ loose art is tiled).
#[test]
fn select_present_bakes_named_and_group_flip_objects_but_not_loose_art() {
    use ph2d_ecs::{ChildOf, GroupedChildren};
    let mut sim = SimWorld::new();
    let named = sim.world_mut().spawn((Name::new("Named"),)).id();
    let group = sim
        .world_mut()
        .spawn((Name::new("Group"), GroupedChildren))
        .id();
    let child = sim.world_mut().spawn((ChildOf(group),)).id(); // UNNAMED group child
    let loose = sim.world_mut().spawn(()).id(); // unnamed, no group

    // The map is FlipObjectId -> entity bits (what `flip_entities::sync` builds).
    let mut map = FlipEntityMap::new();
    map.insert(FlipObjectId(10), named.to_bits());
    map.insert(FlipObjectId(20), child.to_bits());
    map.insert(FlipObjectId(30), loose.to_bits());

    let present = select_present(sim.world(), &map);

    assert_eq!(
        present.get(&FlipObjectId(10)),
        Some(&Some("Named".to_string())),
        "a named Flip object is tiled, carrying its name"
    );
    assert_eq!(
        present.get(&FlipObjectId(20)),
        Some(&None),
        "an UNNAMED group child is tiled by its id, with no name"
    );
    assert!(
        !present.contains_key(&FlipObjectId(30)),
        "unnamed canvas Flip art no group references is NOT tiled"
    );
}
