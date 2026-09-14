//! Testes de `picking.rs` — irmão por tecto de LOC (`architecture_workspace_file_loc_cap`), cortado
//! quando o picking aprendeu a malha de uma sprite (plano `docs/Skeleton/03`, W3, 2026-09-13).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim (menos um nível de recuo), do ficheiro que o
//! continha.

use super::*;
use bevy_ecs::entity::Entity;
use ph2d_core::Vec2;
use ph2d_ecs::PresentWorld;

/// Spawn a present-side mirror entity for `sim_entity` at `(x,
/// y)` with `size`. Returns the bits the renderer's picking
/// surfaces back to callers.
fn spawn_at(present: &mut PresentWorld, sim_entity: Entity, x: f32, y: f32, size: [f32; 2]) -> u64 {
    let gt =
        GlobalTransform::from_transform(ph2d_ecs::Transform::from_translation(Vec2::new(x, y)));
    let ri = RenderInstance {
        world_pos: [x, y],
        size,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        basis: RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        z_order: 0,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    };
    present.world_mut().spawn((SimRef(sim_entity), gt, ri));
    sim_entity.to_bits()
}

/// Like [`spawn_at`] but with a non-zero pivot offset: the pivot
/// (transform/world_pos) is at `(x, y)`, while the quad CENTER sits
/// at `(x + anchor.0, y + anchor.1)`.
fn spawn_at_with_anchor(
    present: &mut PresentWorld,
    sim_entity: Entity,
    x: f32,
    y: f32,
    size: [f32; 2],
    anchor: [f32; 2],
) -> u64 {
    let gt =
        GlobalTransform::from_transform(ph2d_ecs::Transform::from_translation(Vec2::new(x, y)));
    let ri = RenderInstance {
        world_pos: [x, y],
        size,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        basis: RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor,
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        z_order: 0,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    };
    present.world_mut().spawn((SimRef(sim_entity), gt, ri));
    sim_entity.to_bits()
}

/// Allocate a fresh Entity from a SimWorld, then immediately drop
/// the world to keep the test pure-PresentWorld. Used to get
/// realistic `Entity::to_bits()` values where index+generation
/// reflect actual bevy_ecs allocator state.
fn fresh_sim_entity(sim: &mut ph2d_ecs::SimWorld) -> Entity {
    sim.world_mut().spawn_empty().id()
}

#[test]
fn world_bbox_contains_center() {
    let b = WorldBbox {
        min: [-1.0, -1.0],
        max: [1.0, 1.0],
    };
    assert!(b.contains([0.0, 0.0]));
    assert!(b.contains([1.0, 1.0]));
    assert!(b.contains([-1.0, -1.0]));
    assert!(!b.contains([1.001, 0.0]));
}

#[test]
fn world_bbox_center_half_derives_correctly() {
    let b = WorldBbox {
        min: [2.0, 3.0],
        max: [4.0, 9.0],
    };
    let (c, h) = b.center_half();
    assert_eq!(c, [3.0, 6.0]);
    assert_eq!(h, [1.0, 3.0]);
}

#[test]
fn anchor_offsets_pick_region_away_from_pivot() {
    // Pivot at origin, 2×2 quad shifted +5 on X by the anchor →
    // quad spans x∈[4,6], y∈[-1,1]. A click on the pivot must MISS
    // (no quad there); a click on the shifted quad center must HIT.
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let sim_e = fresh_sim_entity(&mut sim);
    let bits = spawn_at_with_anchor(&mut present, sim_e, 0.0, 0.0, [2.0, 2.0], [5.0, 0.0]);
    assert_eq!(
        pick_sprite_at_world(present.world_mut(), [0.0, 0.0]),
        None,
        "pivot is empty once the quad is anchored away"
    );
    assert_eq!(
        pick_sprite_at_world(present.world_mut(), [5.0, 0.0]),
        Some(bits),
        "the shifted quad center is pickable"
    );
}

#[test]
fn anchor_offsets_selection_bbox_to_quad_center() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let sim_e = fresh_sim_entity(&mut sim);
    let bits = spawn_at_with_anchor(&mut present, sim_e, 0.0, 0.0, [2.0, 2.0], [5.0, 0.0]);
    let bbox = selection_bbox_world(present.world_mut(), bits).expect("entity present");
    let (center, half) = bbox.center_half();
    assert_eq!(
        center,
        [5.0, 0.0],
        "gizmo box tracks the quad, not the pivot"
    );
    assert_eq!(half, [1.0, 1.0]);
}

#[test]
fn pick_returns_none_when_no_sprites_overlap() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let sim_e = fresh_sim_entity(&mut sim);
    spawn_at(&mut present, sim_e, 10.0, 10.0, [1.0, 1.0]);
    let hit = pick_sprite_at_world(present.world_mut(), [0.0, 0.0]);
    assert_eq!(hit, None);
}

#[test]
fn pick_returns_entity_when_point_inside_bbox() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let sim_e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, sim_e, 0.0, 0.0, [2.0, 2.0]);
    // Inside the unit-half bbox.
    let hit = pick_sprite_at_world(present.world_mut(), [0.5, -0.5]);
    assert_eq!(hit, Some(bits));
}

#[test]
fn pick_topmost_when_two_sprites_overlap() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    // Spawn order = present-side iteration order within the
    // shared archetype (Transform + RenderInstance + SimRef).
    // Last spawned wins per the documented "topmost = last hit"
    // heuristic.
    let lower = fresh_sim_entity(&mut sim);
    let upper = fresh_sim_entity(&mut sim);
    spawn_at(&mut present, lower, 0.0, 0.0, [4.0, 4.0]);
    let upper_bits = spawn_at(&mut present, upper, 0.0, 0.0, [2.0, 2.0]);
    let hit = pick_sprite_at_world(present.world_mut(), [0.0, 0.0]);
    assert_eq!(hit, Some(upper_bits));
}

#[test]
fn selection_bbox_recovers_size_from_render_instance() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let sim_e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, sim_e, 5.0, -3.0, [3.0, 2.0]);
    let b = selection_bbox_world(present.world_mut(), bits).unwrap();
    // 3×2 centered on (5, -3): min=(5-1.5, -3-1), max=(5+1.5, -3+1)
    assert!((b.min[0] - 3.5).abs() < 1e-5);
    assert!((b.min[1] - (-4.0)).abs() < 1e-5);
    assert!((b.max[0] - 6.5).abs() < 1e-5);
    assert!((b.max[1] - (-2.0)).abs() < 1e-5);
}

#[test]
fn selection_bbox_none_when_entity_absent() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let known = fresh_sim_entity(&mut sim);
    spawn_at(&mut present, known, 0.0, 0.0, [1.0, 1.0]);
    let missing = fresh_sim_entity(&mut sim);
    let b = selection_bbox_world(present.world_mut(), missing.to_bits());
    assert!(b.is_none());
}

/// Spawn a sprite with an explicit 2×2 `basis` (rotation/scale/skew) at the origin.
fn spawn_with_basis(
    present: &mut PresentWorld,
    sim_entity: Entity,
    size: [f32; 2],
    basis: [f32; 4],
) -> u64 {
    let gt =
        GlobalTransform::from_transform(ph2d_ecs::Transform::from_translation(Vec2::new(0.0, 0.0)));
    let mut ri = RenderInstance {
        world_pos: [0.0, 0.0],
        size,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        basis,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        z_order: 0,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    };
    ri.world_pos = [0.0, 0.0];
    present.world_mut().spawn((SimRef(sim_entity), gt, ri));
    sim_entity.to_bits()
}

#[test]
fn sprite_world_to_uv_identity_maps_center_and_corners() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, e, 0.0, 0.0, [2.0, 2.0]); // identity basis
    // Centre → (0.5, 0.5).
    let (u, v) = sprite_world_to_uv(present.world_mut(), bits, [0.0, 0.0]).unwrap();
    assert!(
        (u - 0.5).abs() < 1e-5 && (v - 0.5).abs() < 1e-5,
        "centre: {u},{v}"
    );
    // World +X (right) → u>0.5; world +Y (UP) → v<0.5 (texture TOP).
    let (u, v) = sprite_world_to_uv(present.world_mut(), bits, [0.5, 0.5]).unwrap();
    assert!((u - 0.75).abs() < 1e-5, "u={u}");
    assert!((v - 0.25).abs() < 1e-5, "v={v}");
    // Outside the quad → None (gates the click).
    assert!(sprite_world_to_uv(present.world_mut(), bits, [2.0, 0.0]).is_none());
}

#[test]
fn sprite_world_to_uv_honours_rotation() {
    // 90° CCW basis [0,1,-1,0]: the texture's local +X (right edge, u→1) now points to
    // world +Y (up). So a click ABOVE the centre must land on the texture's RIGHT edge —
    // which the old translation-only AABB mapping could never do.
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_with_basis(&mut present, e, [2.0, 2.0], [0.0, 1.0, -1.0, 0.0]);
    let (u, v) = sprite_world_to_uv(present.world_mut(), bits, [0.0, 0.9]).unwrap();
    assert!(u > 0.9, "world-up maps to the texture right edge: u={u}");
    assert!((v - 0.5).abs() < 1e-5, "v centred: {v}");
    // World +X (right) → texture BOTTOM (local −Y) → v>0.5.
    let (_, v2) = sprite_world_to_uv(present.world_mut(), bits, [0.9, 0.0]).unwrap();
    assert!(v2 > 0.9, "world-right maps to the texture bottom: v={v2}");
}

#[test]
fn sprite_world_to_uv_unclamped_returns_off_quad_uv() {
    // The clamped wrapper gates a click off the quad to `None`; the unclamped
    // variant the Painter uses must instead return the (out-of-range) UV so the
    // whole sprite stays paintable past the edge. A 2×2 sprite at origin: world
    // x=2 maps to u=1.5 (one full sprite-width right of centre).
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_with_basis(&mut present, e, [2.0, 2.0], [1.0, 0.0, 0.0, 1.0]);
    assert!(
        sprite_world_to_uv(present.world_mut(), bits, [2.0, 0.0]).is_none(),
        "clamped wrapper still gates off-quad"
    );
    let (u, v) = sprite_world_to_uv_unclamped(present.world_mut(), bits, [2.0, 0.0]).unwrap();
    assert!((u - 1.5).abs() < 1e-5, "off-quad u not clamped: {u}");
    assert!((v - 0.5).abs() < 1e-5, "v centred: {v}");
    // A degenerate basis is still unpaintable (None) on the unclamped path.
    let de = fresh_sim_entity(&mut sim);
    let dbits = spawn_with_basis(&mut present, de, [2.0, 2.0], [0.0, 0.0, 0.0, 0.0]);
    assert!(
        sprite_world_to_uv_unclamped(present.world_mut(), dbits, [0.0, 0.0]).is_none(),
        "degenerate basis stays None"
    );
}

#[test]
fn pick_respects_size_asymmetry() {
    // A wide-thin sprite — the picking algorithm should honor the
    // size aspect, not just the larger dimension.
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let sim_e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, sim_e, 0.0, 0.0, [10.0, 1.0]);
    // Inside the wide bbox but well past the thin Y range.
    let outside_y = pick_sprite_at_world(present.world_mut(), [4.0, 2.0]);
    assert_eq!(outside_y, None);
    let inside = pick_sprite_at_world(present.world_mut(), [4.0, 0.3]);
    assert_eq!(inside, Some(bits));
}

/// Put `mesh` on the present mirror of `sim_entity` — what `attach_skin_meshes` does after the
/// extract for an image bound to a skeleton.
fn give_mesh(present: &mut PresentWorld, sim_entity: Entity, mesh: crate::SpriteMesh) {
    let mut q = present.world_mut().query::<(Entity, &SimRef)>();
    let alvo = q
        .iter(present.world())
        .find(|(_, r)| r.0 == sim_entity)
        .map(|(e, _)| e)
        .expect("the mirror was spawned");
    present.world_mut().entity_mut(alvo).insert(mesh);
}

/// A 2×2 quad's mesh POSED out to `x = 3..5` — the case in which the quad and the mesh disagree,
/// and the only one that tells which of the two a consumer reads.
fn posed_arm() -> crate::SpriteMesh {
    crate::SpriteMesh {
        local: vec![[3.0, 0.0], [5.0, 0.0], [3.0, 2.0]],
        uv: vec![[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
        tris: vec![[0, 1, 2]],
    }
}

/// ⭐⭐⭐ **A skinned sprite is picked where its MESH is drawn, not where its quad rests** (plan
/// `docs/Skeleton/03`, W3). The control is the same instance before the mesh arrives.
///
/// (Mutation: `covers_local` ignoring the mesh ⇒ RED on both points.)
#[test]
fn a_skinned_sprite_is_picked_where_its_mesh_is_drawn_not_where_its_quad_rests() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, e, 0.0, 0.0, [2.0, 2.0]);
    assert_eq!(
        pick_sprite_at_world(present.world_mut(), [0.0, 0.0]),
        Some(bits),
        "control: the quad picks at its centre"
    );
    assert_eq!(pick_sprite_at_world(present.world_mut(), [4.0, 0.5]), None);
    give_mesh(&mut present, e, posed_arm());
    assert_eq!(
        pick_sprite_at_world(present.world_mut(), [4.0, 0.5]),
        Some(bits),
        "the posed arm is drawn at x = 4 and does not pick there"
    );
    assert_eq!(
        pick_sprites_at_world(present.world_mut(), [4.0, 0.5]),
        vec![bits]
    );
    assert_eq!(
        pick_sprite_at_world(present.world_mut(), [0.0, 0.0]),
        None,
        "the rest quad is empty canvas once the image is drawn somewhere else"
    );
}

/// ⭐⭐ **The box of a skinned sprite — the gizmo's, the rubber band's, View All's — is the box of its
/// mesh.** And View All unions every DRAWN box: the anchor of a plain sprite counts too (the old
/// View All rebuilt a pivot-centred quad by hand and ignored it).
///
/// (Mutation: `drawn_world_bbox` ignoring the mesh ⇒ RED on the selection box.)
#[test]
fn the_box_of_a_skinned_sprite_is_the_box_of_its_mesh() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, e, 0.0, 0.0, [2.0, 2.0]);
    give_mesh(&mut present, e, posed_arm());
    let b = selection_bbox_world(present.world_mut(), bits).expect("selected");
    assert_eq!((b.min, b.max), ([3.0, 0.0], [5.0, 2.0]));
    assert_eq!(
        pick_sprites_in_world_rect(present.world_mut(), [4.5, 1.0], [6.0, 3.0]),
        vec![bits]
    );
    assert!(
        pick_sprites_in_world_rect(present.world_mut(), [-1.0, -1.0], [-0.5, -0.5]).is_empty(),
        "a corner of the REST quad is not inside the drawn box"
    );
    let plain = fresh_sim_entity(&mut sim);
    spawn_at(&mut present, plain, -10.0, 0.0, [2.0, 2.0]);
    let anchored = fresh_sim_entity(&mut sim);
    spawn_at_with_anchor(&mut present, anchored, 0.0, 10.0, [2.0, 2.0], [0.0, 5.0]);
    let all = scene_sprites_bbox_world(present.world_mut()).expect("three sprites");
    assert_eq!(
        (all.min, all.max),
        ([-11.0, -1.0], [5.0, 16.0]),
        "View All: the mesh, the plain quad and the ANCHORED quad"
    );
}

/// ⭐⭐ **A point on a skinned sprite reads the REST UV under it** — the painter and the eyedropper
/// land on the texel that is drawn there. Off the mesh, the quad law (the rest image) still answers
/// the unclamped query — a named limit: a stroke that leaves the posed silhouette is mapped as if
/// the image rested.
///
/// (Mutation: `sprite_world_to_uv_unclamped` without the mesh branch ⇒ RED.)
#[test]
fn a_point_on_a_skinned_sprite_reads_the_rest_uv_under_it() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, e, 0.0, 0.0, [2.0, 2.0]);
    give_mesh(&mut present, e, posed_arm());
    let (u, v) = sprite_world_to_uv(present.world_mut(), bits, [3.5, 0.5]).expect("on the mesh");
    assert!(
        (u - 0.25).abs() < 1e-5 && (v - 0.75).abs() < 1e-5,
        "the posed point (3.5, 0.5) read ({u}, {v}) — the rest UV under it is (0.25, 0.75)"
    );
    let (u0, v0) =
        sprite_world_to_uv_unclamped(present.world_mut(), bits, [0.0, 0.0]).expect("quad law");
    assert!((u0 - 0.5).abs() < 1e-5 && (v0 - 0.5).abs() < 1e-5);
    assert_eq!(
        sprite_world_to_uv(present.world_mut(), bits, [0.0, 0.0]),
        None,
        "the rest quad of a skinned sprite is not drawn — a click there is not on the image"
    );
}

/// ⚠️ **A mesh the renderer would NOT draw is not a mesh for picking either** — the sprite draws its
/// quad, and it must pick there.
#[test]
fn a_mesh_the_renderer_would_not_draw_picks_like_the_quad() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, e, 0.0, 0.0, [2.0, 2.0]);
    let mut torta = posed_arm();
    torta.uv.pop();
    give_mesh(&mut present, e, torta);
    assert_eq!(
        pick_sprite_at_world(present.world_mut(), [0.0, 0.0]),
        Some(bits)
    );
    assert_eq!(pick_sprite_at_world(present.world_mut(), [4.0, 0.5]), None);
}

/// ⭐⭐⭐ **A porta de canvas: os TRÊS estados** ([`MeshUv`]), e o controlo da sprite SEM malha.
///
/// ⛔ O defeito que ela fecha (medido 2026-09-14): o Painter mapeia o ponteiro pelo afim do QUAD DE
/// REPOUSO, então numa arte presa e DOBRADA a pincelada cai deslocada pela deformação — e as duas
/// portas que sabiam da malha não tinham chamador nenhum de produto.
#[test]
fn the_canvas_port_answers_the_art_the_quad_and_the_refusal() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, e, 0.0, 0.0, [2.0, 2.0]);
    give_mesh(&mut present, e, posed_arm());

    // Sobre a arte: a UV de repouso do texel desenhado ali — a começar ou a meio, é a mesma.
    for starting in [true, false] {
        assert!(
            matches!(
                crate::mesh_uv(present.world_mut(), bits, [3.5, 0.5], starting),
                crate::MeshUv::Use { u, v, .. } if (u - 0.25).abs() < 1e-5 && (v - 0.75).abs() < 1e-5
            ),
            "sobre a arte a porta devolve a UV de repouso (starting = {starting})"
        );
    }
    // Fora dela: um gesto que COMEÇA é recusado — um traço não nasce sobre um quad que não se
    // desenha —, e um gesto ABERTO segue pela lei do quad, para a pincelada não se partir.
    assert_eq!(
        crate::mesh_uv(present.world_mut(), bits, [0.0, 0.0], true),
        crate::MeshUv::Refuse
    );
    assert!(matches!(
        crate::mesh_uv(present.world_mut(), bits, [0.0, 0.0], false),
        crate::MeshUv::Use { u, v, warp } if (u - 0.5).abs() < 1e-5 && (v - 0.5).abs() < 1e-5
            && warp == [[1.0, 0.0], [0.0, 1.0]]
    ));

    // ⛔ O CONTROLO: uma sprite SEM malha devolve `Quad` nos dois casos — a lei do chamador fica
    // intocada (no Painter ela carrega a grelha da folha e o *Repeat Image*, que esta porta não
    // conhece). Sem ele, uma porta que respondesse `Use` a toda gente passaria nas asserções acima.
    let plain = fresh_sim_entity(&mut sim);
    let pbits = spawn_at(&mut present, plain, 20.0, 0.0, [2.0, 2.0]);
    for (p, starting) in [([20.0, 0.0], true), ([99.0, 0.0], false)] {
        assert_eq!(
            crate::mesh_uv(present.world_mut(), pbits, p, starting),
            crate::MeshUv::Quad,
            "uma sprite sem malha nao passa por esta porta"
        );
    }
}

/// ⭐⭐⭐ **A porta de canvas devolve a DEFORMAÇÃO LOCAL, e ela é adimensional** — report do dono com
/// foto (2026-09-14): *«o pincel é redondo mas pinta como se os polígonos não estivessem
/// deformados»*. Aqui o triângulo comprime `x` a metade ⇒ a `warp` diz `0,5` ali, e é disso que o
/// pincel tira a elipse que sai redonda no ecrã.
///
/// ⛔ **O CONTROLO é o `posed_arm`**, que só TRANSLADA a arte (a mesma escala do quad): ali a
/// deformação é a **identidade**, e uma porta que devolvesse sempre a matriz do triângulo
/// responderia `[[2,0],[0,2]]` — o tamanho da sprite, não a deformação.
#[test]
fn the_canvas_port_reports_the_local_deformation_and_it_is_dimensionless() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();

    let rigido = fresh_sim_entity(&mut sim);
    let rb = spawn_at(&mut present, rigido, 0.0, 0.0, [2.0, 2.0]);
    give_mesh(&mut present, rigido, posed_arm());
    assert!(
        matches!(
            crate::mesh_uv(present.world_mut(), rb, [3.5, 0.5], true),
            crate::MeshUv::Use { warp, .. } if warp == [[1.0, 0.0], [0.0, 1.0]]
        ),
        "uma malha que so' TRANSLADA a arte nao a deforma"
    );

    let dobrado = fresh_sim_entity(&mut sim);
    let db = spawn_at(&mut present, dobrado, 20.0, 0.0, [2.0, 2.0]);
    give_mesh(
        &mut present,
        dobrado,
        crate::SpriteMesh {
            local: vec![[3.0, 0.0], [4.0, 0.0], [3.0, 2.0]],
            uv: vec![[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
            tris: vec![[0, 1, 2]],
        },
    );
    let crate::MeshUv::Use { warp, .. } =
        crate::mesh_uv(present.world_mut(), db, [23.2, 0.2], true)
    else {
        panic!("o ponto tinha de cair sobre o triangulo");
    };
    assert!(
        (warp[0][0] - 0.5).abs() < 1e-5
            && warp[0][1].abs() < 1e-5
            && warp[1][0].abs() < 1e-5
            && (warp[1][1] - 1.0).abs() < 1e-5,
        "o triangulo comprime x a metade: {warp:?}"
    );
}
