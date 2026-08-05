//! Gates for the group membrane (`motion_bridge_objects`, doc 86 §2/§6/§9.6). FILHO of
//! `objects` via `#[path]` (so `use super::*` reaches the private `resolve_drawing_leaf`,
//! `walk_group_transforms`, `entity_is_in_a_named_group`, `LeafInstance`), split off to
//! keep the parent under the shell LOC cap.

use super::*;

#[test]
fn the_membrane_publishes_exactly_the_columns_the_sink_reads() {
    // doc 86 §6 gate 3 (`bake_source_is_the_same_columns_the_sink_reads`):
    // the tile the membrane emits, lowered by the render sink, carries the
    // appearance back to the `RenderInstance` — the publish side and the
    // read side cannot diverge (the two-doors bug this repo hunts). A DUMMY
    // texture_id 7 that is neither the atlas sentinel (0) nor a size/tint
    // value, so a lowering that ignored the column could not pass.
    let tile = appearance_tile([2.0, 3.0], [1.0, 0.0, 0.0, 1.0], [0.1, 0.2, 0.3, 0.4], 7);
    let inst = ph2d_eval_motion::lower_to_instances(&tile);
    assert_eq!(inst.len(), 1);
    assert_eq!(inst[0].texture_id, 7, "which texture");
    assert_eq!(inst[0].size, [2.0, 3.0]);
    assert_eq!(inst[0].tint, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(inst[0].atlas_uv, [0.1, 0.2, 0.3, 0.4], "the atlas cell");
    assert_eq!(inst[0].world_pos, [0.0, 0.0], "at the origin — a template");
}

#[test]
fn a_group_lays_its_children_out_relative_to_the_group() {
    // doc 86 §2 A4: a named group emits its subtree's leaves at their transform
    // RELATIVE to the group — the group's OWN pose is EXCLUDED (the tile is a
    // template stamped at each point's `P`). The walk composes down the chain,
    // so a nested subgroup lays out correctly. This is the load-bearing layout;
    // the per-medium appearance is the column gate above + the smoke.
    use ph2d_core::Vec2;
    use ph2d_ecs::ChildOf;
    let at = |x: f32, y: f32| Transform {
        translation: Vec2::new(x, y),
        ..Transform::IDENTITY
    };
    let mut sim = SimWorld::new();
    // The group carries its OWN translation (100,50) — it must NOT leak into
    // the children's layout.
    let group = sim
        .world_mut()
        .spawn((Name::new("Group"), GroupedChildren, at(100.0, 50.0)))
        .id();
    let c1 = sim.world_mut().spawn((at(2.0, 0.0), ChildOf(group))).id();
    let c2 = sim.world_mut().spawn((at(-2.0, 1.0), ChildOf(group))).id();
    // A nested subgroup at (5,0) with a grandchild at (1,0) ⇒ the grandchild
    // relative to the TOP group is (6,0).
    let sub = sim
        .world_mut()
        .spawn((GroupedChildren, at(5.0, 0.0), ChildOf(group)))
        .id();
    let gc = sim.world_mut().spawn((at(1.0, 0.0), ChildOf(sub))).id();

    let mut out: Vec<(Entity, Transform)> = Vec::new();
    walk_group_transforms(sim.world(), group, Transform::IDENTITY, &mut out);

    let pos = |e: Entity| {
        out.iter()
            .find(|(x, _)| *x == e)
            .map(|(_, t)| [t.translation.x, t.translation.y])
    };
    // ⚠️ If the walk folded the group's own transform in, the group would be at
    // (100,50) and c1 at (102,50) — these assertions are the mutation guard.
    assert_eq!(
        pos(group),
        Some([0.0, 0.0]),
        "the group's OWN pose is excluded"
    );
    assert_eq!(
        pos(c1),
        Some([2.0, 0.0]),
        "a direct child at its local position"
    );
    assert_eq!(pos(c2), Some([-2.0, 1.0]), "the second direct child");
    assert_eq!(
        pos(gc),
        Some([6.0, 0.0]),
        "a grandchild composed down the chain (5+1)"
    );
}

#[test]
fn the_group_stream_lowers_to_one_instance_per_child() {
    // doc 86 §2 A4: the N-instance stream a group publishes, lowered by the
    // sink, yields ONE `RenderInstance` per child at its position + texture —
    // the two-doors coverage for the group path (the single-object version is
    // the column gate above). DISTINCT texture_ids (3, 9) that are neither the
    // atlas sentinel nor a coordinate, so a lowering that dropped the column
    // could not pass.
    let leaf = |p: [f32; 2], size: [f32; 2], tid: u32| LeafInstance {
        p,
        rot_deg: 0.0,
        size,
        tint: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0, 1.0, 1.0],
        tid,
        gid: 0, // raster leaves (this test); the live-vector column is exercised below
    };
    let leaves = vec![
        leaf([-1.0, 0.0], [0.5, 0.5], 3),
        leaf([1.0, 0.5], [0.7, 0.7], 9),
    ];
    let inst = ph2d_eval_motion::lower_to_instances(&group_stream(&leaves));
    assert_eq!(inst.len(), 2, "one instance per child");
    assert_eq!(inst[0].world_pos, [-1.0, 0.0]);
    assert_eq!(inst[0].texture_id, 3);
    assert_eq!(inst[1].world_pos, [1.0, 0.5]);
    assert_eq!(inst[1].texture_id, 9);
    assert_eq!(inst[1].size, [0.7, 0.7], "each child keeps its own size");
}

#[test]
fn a_live_vector_object_lowers_to_a_vector_instance_not_a_quad() {
    // Part 1 (Vetor Vivo): a `source.object` that names a VECTOR emits `geometry_id`,
    // and the render sink lowers it to a crisp VECTOR instance (drawn by the vector
    // pass), NOT a textured atlas quad. The two-doors gate for the live path — the
    // publish side (`appearance_vector`) and the read side cannot diverge. A DUMMY
    // geometry_id 5 (not the sentinel 0). RED-FIRST: emitting `texture_id` (the old
    // baked-tile behaviour) lowers to a RenderInstance here and NOTHING to the vector
    // lowering — the mutation that reverts the membrane to a raster tile.
    let obj = appearance_vector([2.0, 3.0], [1.0, 1.0, 1.0, 1.0], 5);
    let mut vecs = Vec::new();
    ph2d_eval_motion::lower_to_vector_instances_onto(&obj, &mut vecs);
    assert_eq!(vecs.len(), 1, "one live-vector instance");
    assert_eq!(vecs[0].geometry_id, 5, "carrying the live handle");
    assert_eq!(vecs[0].size, [2.0, 3.0]);
    // ...and the SPRITE lowering SKIPS it (geometry_id > 0.5) — no blank atlas quad.
    let quads = ph2d_eval_motion::lower_to_instances(&obj);
    assert!(
        quads.is_empty(),
        "a live vector is not ALSO stamped as an atlas quad"
    );
}

#[test]
fn a_mixed_media_group_draws_each_child_once() {
    // A composed group (Raster + Vector, doc 86 §2 A4 — Enio's mixed-media groups)
    // publishes BOTH columns: a sprite row (`tid > 0`, `gid = 0`) and a vector row
    // (`tid = 0`, `gid > 0`). The lowering's `geometry_id > 0.5` split routes each to
    // exactly ONE pass — the sprite to a quad, the vector to a live path — so each
    // child draws once (never a blank quad, never a double-draw). RED-FIRST if a leaf
    // set both ids, or if `group_stream` dropped the `geometry_id` column.
    let leaf = |p: [f32; 2], tid: u32, gid: u32| LeafInstance {
        p,
        rot_deg: 0.0,
        size: [0.6, 0.6],
        tint: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0, 1.0, 1.0],
        tid,
        gid,
    };
    let stream = group_stream(&[leaf([-1.0, 0.0], 3, 0), leaf([1.0, 0.0], 0, 5)]);
    // Only the sprite row lowers to a quad.
    let quads = ph2d_eval_motion::lower_to_instances(&stream);
    assert_eq!(quads.len(), 1, "only the sprite lowers to a quad");
    assert_eq!(quads[0].texture_id, 3);
    assert_eq!(quads[0].world_pos, [-1.0, 0.0]);
    // Only the vector row lowers to a live path.
    let mut vecs = Vec::new();
    ph2d_eval_motion::lower_to_vector_instances_onto(&stream, &mut vecs);
    assert_eq!(vecs.len(), 1, "only the vector lowers to a live path");
    assert_eq!(vecs[0].geometry_id, 5);
    assert_eq!(vecs[0].world_pos, [1.0, 0.0]);
}

#[test]
fn an_unnamed_group_child_resolves_by_its_drawing_id() {
    // doc 86 §9.6 follow-up: a vector/flip child of a group with NO name still
    // stamps, because `resolve_drawing_leaf` looks the tile up by the child's
    // DRAWING id (`VecPathRef`/`FlipObjectRef`), not by `Name`. A mutation reverting
    // the lookup to the name (empty ⇒ None) skips the unnamed child — the exact bug
    // this fix closes.
    use ph2d_ecs::ChildOf;
    let mut sim = SimWorld::new();
    let group = sim
        .world_mut()
        .spawn((Name::new("Group"), GroupedChildren))
        .id();
    // Two UNNAMED children — one vector (drawing id 7), one Flip (drawing id 9).
    let v = sim.world_mut().spawn((VecPathRef(7), ChildOf(group))).id();
    let f = sim
        .world_mut()
        .spawn((FlipObjectRef(9), ChildOf(group)))
        .id();

    // Bakes seeded under those drawing ids, as a real (id-keyed) bake does: the
    // vector carries a LIVE handle (geometry_id 70), the Flip a baked texture_id 90.
    let mut vbake = crate::motion_object_bake::ObjectBake::default();
    vbake.seed_for_test(7, 70, [2.0, 1.0]);
    let mut fbake = crate::motion_flip_bake::FlipObjectBake::default();
    fbake.seed_for_test(9, 90, [3.0, 4.0]);

    let world = sim.world();
    // ⚠️ A VECTOR child resolves to `geometry_id` (live), a FLIP child to `texture_id`
    // (baked) — the media split that lets a mixed group draw each part once.
    let rv = resolve_drawing_leaf(world, v, &Transform::IDENTITY, &vbake, &fbake)
        .expect("the unnamed vector child resolves by its drawing id");
    assert_eq!(
        rv.gid, 70,
        "the vector child's LIVE handle is looked up by id 7"
    );
    assert_eq!(rv.tid, 0, "a live vector carries no texture_id");
    assert_eq!(rv.size, [2.0, 1.0], "carrying its drawing's world size");
    let rf = resolve_drawing_leaf(world, f, &Transform::IDENTITY, &vbake, &fbake)
        .expect("the unnamed Flip child resolves by its drawing id");
    assert_eq!(rf.tid, 90, "the Flip child's tile is looked up by id 9");
    assert_eq!(rf.gid, 0, "a baked Flip carries no geometry_id");
}

#[test]
fn the_named_group_predicate_matches_the_group_walk() {
    // doc 86 §9.6: `entity_is_in_a_named_group` (the up-walk the bake uses to decide
    // WHICH drawings to tile) is the SAME tree relation `group_externals` descends
    // DOWN — so the set the bake tiles == the set the membrane stamps. It counts ONLY
    // a `GroupedChildren` entity with a NON-EMPTY `Name` (exactly what `group_externals`
    // starts from). The fixture makes each near-miss wrong: an unnamed group, a named
    // non-group, and a deep grandchild.
    use ph2d_ecs::ChildOf;
    let mut sim = SimWorld::new();
    // Named group G → V1 (unnamed child)  and  → S (UNNAMED subgroup) → V2 (grandchild).
    let g = sim
        .world_mut()
        .spawn((Name::new("G"), GroupedChildren))
        .id();
    let v1 = sim.world_mut().spawn((ChildOf(g),)).id();
    let s = sim.world_mut().spawn((GroupedChildren, ChildOf(g))).id();
    let v2 = sim.world_mut().spawn((ChildOf(s),)).id();
    // UNNAMED root group U → C: a child of an unnamed group is NOT stamped.
    let u = sim.world_mut().spawn((GroupedChildren,)).id();
    let c = sim.world_mut().spawn((ChildOf(u),)).id();
    // NAMED sprite P (Name but NOT a group) → C2: a child of a non-group is NOT stamped.
    let p = sim.world_mut().spawn((Name::new("P"),)).id();
    let c2 = sim.world_mut().spawn((ChildOf(p),)).id();
    // A root entity with no parent.
    let r = sim.world_mut().spawn(()).id();

    let w = sim.world();
    for (e, why) in [
        (g, "the named group itself"),
        (v1, "a direct child"),
        (s, "an unnamed subgroup under a named group"),
        (v2, "a grandchild through the subgroup"),
    ] {
        assert!(
            entity_is_in_a_named_group(w, e),
            "{why} is in a named group"
        );
    }
    for (e, why) in [
        (c, "a child of an UNNAMED group"),
        (u, "an unnamed group"),
        (c2, "a child of a NAMED non-group"),
        (r, "a root, no group"),
    ] {
        assert!(
            !entity_is_in_a_named_group(w, e),
            "{why} is NOT in a named group"
        );
    }

    // Single door: the predicate's TRUE set == the down-walk subtree of the (only)
    // named group G — {G, V1, S, V2}. A predicate that descends differently diverges.
    let mut subtree: Vec<(Entity, Transform)> = Vec::new();
    walk_group_transforms(w, g, Transform::IDENTITY, &mut subtree);
    let down: std::collections::BTreeSet<Entity> = subtree.iter().map(|(e, _)| *e).collect();
    let up: std::collections::BTreeSet<Entity> = [g, v1, s, v2, u, c, p, c2, r]
        .into_iter()
        .filter(|&e| entity_is_in_a_named_group(w, e))
        .collect();
    assert_eq!(
        up, down,
        "the up-walk == the down-walk subtree (single door)"
    );
}
