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
    // vector carries a LIVE handle (geometry_id 70) AND a LOD tile (texture_id 700 —
    // used only above the count threshold, post-cook), the Flip a baked texture_id 90.
    let mut vbake = crate::motion_object_bake::ObjectBake::default();
    vbake.seed_for_test(7, 70, 700, [2.0, 1.0]);
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

// ── LOD (the freeze fix, ADR-0154 follow-up): a live vector stamped past the count
// knee renders as a GPU-instanced tile instead of a crisp per-instance vector ──────

/// A `VectorInstance` at `(x,0)`, identity pose, white, for geometry `gid`.
#[cfg(test)]
fn lod_vi(gid: u32, x: f32) -> VectorInstance {
    VectorInstance {
        geometry_id: gid,
        world_pos: [x, 0.0],
        size: [1.0, 1.0],
        basis: [1.0, 0.0, 0.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
    }
}

/// **The freeze-killer, per-geometry.** A geometry stamped MORE than the threshold
/// (with a baked tile) is moved to `instances` as GPU-instanced tile quads; a
/// geometry at/below stays crisp in `vector_instances`. RED-first: with no LOD the
/// high-count geometry stays a crisp vector (the 160k freeze). Two geometries in one
/// buffer prove the decision is PER-GEOMETRY, not per-buffer.
#[test]
fn a_high_count_geometry_becomes_tiles_a_low_count_one_stays_crisp() {
    let mut bake = crate::motion_object_bake::ObjectBake::default();
    bake.seed_for_test(1, 5, 500, [1.0, 1.0]); // gid 5 → tile 500
    bake.seed_for_test(2, 6, 600, [1.0, 1.0]); // gid 6 → tile 600
    // gid 5 stamped 6×, gid 6 stamped 2×; threshold 3.
    let mut vectors = vec![
        lod_vi(5, 0.0),
        lod_vi(6, 1.0),
        lod_vi(5, 2.0),
        lod_vi(5, 3.0),
        lod_vi(5, 4.0),
        lod_vi(5, 5.0),
        lod_vi(5, 6.0),
        lod_vi(6, 7.0),
    ];
    let mut instances: Vec<RenderInstance> = Vec::new();
    apply_object_lod(&mut instances, &mut vectors, &bake, 3);
    assert_eq!(instances.len(), 6, "gid 5 (6 > 3) became GPU tiles");
    assert!(
        instances.iter().all(|i| i.texture_id == 500),
        "the tiles sample gid 5's baked LOD texture"
    );
    assert_eq!(vectors.len(), 2, "only the low-count geometry stays crisp");
    assert!(
        vectors.iter().all(|v| v.geometry_id == 6),
        "gid 6 (2 <= 3) stayed a crisp vector"
    );
}

/// **The swap is FAITHFUL** — a tile lands EXACTLY where the crisp vector would. The
/// LOD keeps `world_pos`/`size`/`basis` (rotation) / `tint` and only adds the tile
/// texture + the individual-texture unit UV; every other field is the identity value
/// `lower_to_instances_onto`'s `make` uses, so a converted tile is indistinguishable
/// from a membrane-lowered one.
#[test]
fn the_lod_tile_lands_exactly_where_the_crisp_vector_would() {
    let vi = VectorInstance {
        geometry_id: 9,
        world_pos: [3.5, -2.0],
        size: [2.0, 0.5],
        basis: [0.0, 1.0, -1.0, 0.0], // a 90° rotation — carried, not dropped
        tint: [0.2, 0.4, 0.6, 0.8],
    };
    let tile = vector_instance_as_tile(&vi, 42);
    assert_eq!(tile.world_pos, vi.world_pos, "same position");
    assert_eq!(tile.size, vi.size, "same size");
    assert_eq!(tile.basis, vi.basis, "same rotation basis");
    assert_eq!(tile.tint, vi.tint, "same tint");
    assert_eq!(tile.texture_id, 42, "sampling the object's LOD tile");
    assert_eq!(
        tile.atlas_uv,
        [0.0, 0.0, 1.0, 1.0],
        "the whole individual texture"
    );
    assert_eq!(tile.opacity, 1.0, "identity default (no opacity authoring)");
    assert_eq!(tile.premultiplied, 0.0, "straight alpha (not pre-bake)");
    assert_eq!(tile.clip_group, RenderInstance::CLIP_GROUP_NONE);
}

/// **Correctness before speed:** below the threshold, OR when no tile was baked, every
/// instance stays crisp — the LOD never blanks a shape it cannot tile. The empty-bake
/// arm is the guard that a swap-without-a-texture can never happen.
#[test]
fn below_threshold_or_without_a_tile_everything_stays_crisp() {
    // (a) below threshold: 2 instances, threshold 3 → untouched.
    let mut bake = crate::motion_object_bake::ObjectBake::default();
    bake.seed_for_test(1, 5, 500, [1.0, 1.0]);
    let mut vectors = vec![lod_vi(5, 0.0), lod_vi(5, 1.0)];
    let mut instances: Vec<RenderInstance> = Vec::new();
    apply_object_lod(&mut instances, &mut vectors, &bake, 3);
    assert!(
        instances.is_empty() && vectors.len() == 2,
        "2 <= 3 stays crisp"
    );
    // (b) over the threshold but NO tile baked → must NOT blank; stays crisp.
    let empty = crate::motion_object_bake::ObjectBake::default();
    let mut vectors = vec![
        lod_vi(5, 0.0),
        lod_vi(5, 1.0),
        lod_vi(5, 2.0),
        lod_vi(5, 3.0),
    ];
    let mut instances: Vec<RenderInstance> = Vec::new();
    apply_object_lod(&mut instances, &mut vectors, &empty, 3);
    assert!(
        instances.is_empty() && vectors.len() == 4,
        "no tile -> correctness before speed, all crisp"
    );
}

/// The LOD's tile lookup: a seeded geometry resolves to its baked `texture_id`, and an
/// unknown geometry to `None` — the wire the partition reads.
#[test]
fn tile_texture_for_gid_finds_the_baked_tile() {
    let mut bake = crate::motion_object_bake::ObjectBake::default();
    bake.seed_for_test(1, 5, 500, [1.0, 1.0]);
    assert_eq!(bake.tile_texture_for_gid(5), Some(500));
    assert_eq!(bake.tile_texture_for_gid(99), None, "an unbaked geometry");
}

/// **Sonda: o CUSTO da PARTIÇÃO do LOD** (report 2026-08-05) — o passo de CPU entre o cook e o
/// desenho, agora que o render está resolvido (crisp cacheado abaixo do joelho, tile instanciado
/// acima). `apply_object_lod` conta por `geometry_id` (BTreeMap), resolve a tile e MOVE as
/// instâncias acima do joelho para `instances` como quads. É O(N); esta sonda mede se ele é um
/// segundo freeze escondido (160k) ou custo insignificante — decide se há próxima alavanca.
/// `cargo test -p ph2d-host-desktop --release the_lod_partition_cost -- --ignored --nocapture`
#[test]
#[ignore = "sonda manual de escala; rode em --release --nocapture"]
fn the_lod_partition_cost_at_scale() {
    use std::time::Instant;
    let mut bake = crate::motion_object_bake::ObjectBake::default();
    bake.seed_for_test(1, 5, 500, [1.0, 1.0]); // gid 5 → tile 500 (acima do joelho ⇒ tiles)
    println!("\n=== PARTICAO DO LOD: apply_object_lod (ms/frame CPU) ===");
    for &n in &[10_000usize, 40_000, 160_000] {
        let iters = if n >= 100_000 { 10 } else { 40 };
        // O caso do freeze: N instâncias de UMA geometria, todas acima do joelho ⇒ viram tiles.
        let base: Vec<VectorInstance> = (0..n).map(|i| lod_vi(5, (i % 400) as f32 * 1.3)).collect();
        let t = Instant::now();
        for _ in 0..iters {
            let mut vectors = base.clone();
            let mut instances: Vec<RenderInstance> = Vec::with_capacity(n);
            apply_object_lod(&mut instances, &mut vectors, &bake, 3);
            std::hint::black_box(&instances);
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        // O `.clone()` da entrada faz parte do laço; a partição pura é menor. Mede o teto.
        println!("N={n:>7}  particao(+clone da entrada)={ms:>7.3} ms/frame");
    }
}

/// Uma cena vetorial com UM caminho aberto nomeado, e o mapa/xform que o frame
/// entrega aos dois publicadores.
#[cfg(test)]
fn one_named_path(
    sim: &mut SimWorld,
    name: &str,
) -> (
    ph2d_vec_scene::VecScene,
    crate::vec_entities::VecEntityMap,
    ph2d_vec_scene::VecXforms,
) {
    use ph2d_ecs::{Name, Transform, VecPathRef};
    let mut scene = ph2d_vec_scene::VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::VecPath {
        verts: vec![
            ph2d_vec_scene::VecVertex::corner([0.0, 0.0]),
            ph2d_vec_scene::VecVertex::corner([4.0, 0.0]),
            ph2d_vec_scene::VecVertex::corner([4.0, 3.0]),
        ],
        closed: false,
        ..ph2d_vec_scene::VecPath::default()
    });
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Name::new(name), VecPathRef(id)))
        .id();
    let mut map = crate::vec_entities::VecEntityMap::default();
    map.insert(id, e.to_bits());
    (scene, map, ph2d_vec_scene::VecXforms::default())
}

/// Quantos pontos há na coluna `P` do externo `key` (0 se não há externo).
#[cfg(test)]
fn points_under(cook: &ph2d_nodegraph::cook::Cook, key: &str) -> usize {
    use ph2d_nodegraph::attr::Column;
    match cook.externals().get(key).map(|e| e.value.get("P")) {
        Some(Some(Column::Vec2(v))) => v.len(),
        _ => 0,
    }
}

/// O id do único caminho da fixture.
#[cfg(test)]
fn path_id_of(scene: &ph2d_vec_scene::VecScene) -> ph2d_vec_scene::VecPathId {
    scene.paths()[0].id
}

/// **A CURVA DESENHADA SOBREVIVE AO PUBLICADOR DE OBJETOS** (smoke de 2026-08-12:
/// *"escolhi a curva mas não funcionou"*).
///
/// ⚠️ **O defeito era um NOME com três donos.** Toda forma vetorial nomeada é
/// publicada como polilinha por `shapes::publish` **e** assada como objeto, e o
/// `objects::publish` roda DEPOIS e escreve, sob o mesmo nome, uma aparência de
/// UMA instância na origem. Os dois publicadores documentam a colisão — *"objects
/// publish after curves; the last write on a name clash wins"* — e o que se perdia
/// era exactamente a resposta que o `motion.path` e o `motion.spline_wrap` pedem.
/// O leitor achava um stream de um ponto, não achava arco, e caía no fallback: sem
/// erro, sem aviso, com o painel mostrando a forma escolhida.
///
/// A cura é a do irmão `position_of`, uma pergunta adiante: **a geometria ganha
/// canal próprio**, e o nome cru continua a querer dizer *aparência* (que é o que
/// o `source.object` precisa). Este gate afirma as DUAS metades — a curva chega
/// inteira, e a aparência continua onde estava.
#[test]
fn the_drawn_curve_survives_the_object_publisher() {
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let mut sim = SimWorld::new();
    let (scene, map, xforms) = one_named_path(&mut sim, "Path 0");

    // 1) O publicador de CURVAS, como o frame o chama.
    super::super::shapes::publish(&mut cook, &sim, &scene, &map, &xforms);
    let curve_key = ph2d_nodegraph::external::curve_of("Path 0");
    let n_before = points_under(&cook, &curve_key);
    assert!(
        n_before >= 2,
        "a polilinha da forma desenhada tem de estar publicada no canal dela, deu {n_before}"
    );

    // 2) O publicador de OBJETOS, que roda depois no MESMO frame.
    let mut vbake = crate::motion_object_bake::ObjectBake::default();
    vbake.seed_named_for_test(path_id_of(&scene), Some("Path 0"), 70, 700, [2.0, 1.0]);
    super::publish_vector_bakes(&mut cook, &vbake);

    // A CURVA sobrevive, ponto por ponto.
    assert_eq!(
        points_under(&cook, &curve_key),
        n_before,
        "o publicador de objetos nao pode levar a curva embora -- e o defeito do smoke"
    );
    // E a APARÊNCIA continua no nome cru, que e o que o `source.object` le.
    assert_eq!(
        points_under(&cook, "Path 0"),
        1,
        "o nome cru continua sendo a aparencia: UMA instancia"
    );
}

/// **O QUE O GLOW ALCANÇA MUDA DE LADO NO LIMIAR DO LOD** — a metade cruel do bug
/// que o Enio relatou em 2026-08-20 (*"Glow não funciona com shape"*).
///
/// ⚠️ **O passe do glow re-renderiza `instances`, e só ela.** Uma forma abaixo do
/// limiar fica em `vector_instances` (desenhada pela cena vetorial, depois do
/// tonemap) e **não brilha**; a MESMA forma acima do limiar é movida para
/// `instances` como tile e **brilha**. Com `LOD_COUNT = 16_000` isso quer dizer,
/// literalmente: *não brilha com 16 000 cópias, brilha com 16 001*.
///
/// ⚠️ **Este gate não CURA nada — ele fixa o degrau para ele não mudar em silêncio**,
/// e é a metade medida do aviso `Deficit::BlindPass` (que mora no diagnoser, porque
/// a cura real é um passe antes do tonemap: decisão de renderer).
///
/// FALSIFICADO por qualquer mudança que faça a partição mover — ou deixar de mover —
/// no limiar, que é exactamente onde a aparência do glow vira.
#[test]
fn the_lod_threshold_is_where_a_shape_starts_being_visible_to_the_glow() {
    let mut bake = crate::motion_object_bake::ObjectBake::default();
    bake.seed_for_test(1, 7, 700, [1.0, 1.0]);
    let stamps = |n: usize| -> Vec<VectorInstance> {
        #[expect(clippy::cast_precision_loss, reason = "uma fixture pequena")]
        let v = (0..n).map(|i| lod_vi(7, i as f32)).collect();
        v
    };
    // NO limiar: nada se move — a forma continua invisível ao passe.
    let (mut inst, mut vecs) = (Vec::new(), stamps(4));
    apply_object_lod(&mut inst, &mut vecs, &bake, 4);
    assert!(
        inst.is_empty() && vecs.len() == 4,
        "no limiar a forma fica crisp, e o glow não a vê"
    );
    // UM acima: todas viram tiles — e passam a ser exactamente o que o glow lê.
    let (mut inst, mut vecs) = (Vec::new(), stamps(5));
    apply_object_lod(&mut inst, &mut vecs, &bake, 4);
    assert!(
        vecs.is_empty() && inst.len() == 5,
        "um acima do limiar a MESMA forma vira sprite, e passa a brilhar"
    );
    // ⚠️ E o degrau do app é este número, não o `4` da fixture.
    assert_eq!(super::LOD_COUNT, 16_000, "o degrau que o artista encontra");
}
