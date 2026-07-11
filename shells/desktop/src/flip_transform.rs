//! O `Transform` dos objetos Flip (ADR-0111 parity, espelho de [`crate::vec_transform`]).
//!
//! Desde o ADR-0113 cada objeto Flip é uma entidade ([`crate::flip_entities`]).
//! Aqui ela ganha **pose**: a geometria dos traços passa a ser LOCAL, e o afim que
//! a leva ao mundo é `parent_world_transform ∘ Transform` — a MESMA cadeia de um
//! sprite. Consequência: um objeto Flip é movido/girado/escalado pelo **gizmo de
//! sprite**, individualmente ou numa multi-seleção mista, sem gizmo próprio.
//!
//! Identidade ⇒ local é mundo. Todo objeto recém-desenhado nasce assim (a geometria
//! é escrita em coordenadas de mundo pela mão de desenho); o [`settle_origins`] põe
//! o pivô no centro dele assim que o gesto termina, e daí em diante o objeto tem a
//! pose que o gizmo lhe der.
//!
//! Reusa os helpers GENÉRICOS de `vec_transform` (`world_transform`/
//! `xform_of_transform`) — eles não tocam `VecScene`, só o `SimWorld`/`Transform`.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_flip::{FlipDoc, FlipObjectId};
use ph2d_vec_scene::Xform;

use crate::flip_entities::FlipEntityMap;
use crate::vec_transform::{world_transform, xform_of_transform};

/// O afim local→mundo do objeto de `entity` (a cadeia de pais inclusa). É o `model`
/// que o render pré-multiplica no `world_to_clip` para rasterizar a geometria LOCAL
/// na pose certa.
#[must_use]
pub(crate) fn object_xform(sim: &SimWorld, entity: Entity) -> Xform {
    xform_of_transform(world_transform(sim, entity))
}

/// O afim de cada objeto Flip, uma vez por frame. Um objeto na identidade **não
/// entra no mapa** — o render trata o ausente como identidade e não paga lookup.
#[must_use]
pub(crate) fn build(sim: &SimWorld, map: &FlipEntityMap) -> Vec<(FlipObjectId, Xform)> {
    let mut out = Vec::new();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let x = object_xform(sim, e);
        if !x.is_identity() {
            out.push((id, x));
        }
    }
    out
}

/// Move a ORIGEM (o pivô) do objeto de `entity` para `target_world`, **sem mover a
/// arte**: a translação vai para lá e a geometria local recua o mesmo tanto. Igual
/// ao `vec_transform::move_origin_to`, mas o bake é sobre TODOS os desenhos do
/// objeto ([`ph2d_flip::FlipObject::bake_affine`]).
///
/// `false` se a entidade sumiu, se o objeto sumiu, ou se o afim é degenerado.
pub(crate) fn move_origin_to(
    sim: &mut SimWorld,
    doc: &mut FlipDoc,
    entity: Entity,
    object: FlipObjectId,
    target_world: [f32; 2],
) -> bool {
    if sim.world().get_entity(entity).is_err() {
        return false;
    }
    // A translação vive no espaço do PAI; a geometria, no espaço local (pós R·S).
    let parent = ph2d_ecs::parent_world_transform(sim.world(), entity);
    let Some(parent_inv) = xform_of_transform(parent).inverse() else {
        return false;
    };
    let target_parent = parent_inv.apply([f64::from(target_world[0]), f64::from(target_world[1])]);
    let Some(local) = sim.world().get::<ph2d_ecs::Transform>(entity).copied() else {
        return false;
    };
    // Quanto a origem andou, no espaço do pai.
    let delta_parent = [
        target_parent[0] - f64::from(local.translation.x),
        target_parent[1] - f64::from(local.translation.y),
    ];
    // O mesmo deslocamento no espaço LOCAL da geometria: desfaz a rotação/escala
    // próprias (a translação não entra — delta é um vetor).
    let rs = xform_of_transform(ph2d_ecs::Transform {
        translation: ph2d_core::Vec2::new(0.0, 0.0),
        ..local
    });
    let Some(rs_inv) = rs.inverse() else {
        return false;
    };
    let delta_local = rs_inv.apply_vec(delta_parent);
    let Some(obj) = doc.object_mut(object) else {
        return false;
    };
    // A geometria recua exatamente o que a origem avançou ⇒ a arte não se move.
    obj.bake_affine([1.0, 0.0, 0.0, 1.0, -delta_local[0], -delta_local[1]]);
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
        t.translation = ph2d_core::Vec2::new(target_parent[0] as f32, target_parent[1] as f32);
    }
    true
}

/// Põe a origem de cada objeto Flip recém-desenhado no **centro da bbox dele**.
///
/// Um objeto nasce com a geometria em coordenadas de mundo e a entidade na
/// identidade — o pivô cai no centro do mundo. Isto conserta isso assim que a arte
/// pára de crescer (`gesturing` = o objeto que a mão de desenho/borracha ainda está
/// escrevendo em MUNDO a cada frame; assentá-lo no meio somaria geometria +
/// `Transform` e deslocaria a arte do cursor).
///
/// Só toca quem está na **identidade e sem pai** — um objeto já movido tem a origem
/// que o usuário lhe deu. Idempotente: depois de centrado, o delta é zero.
pub(crate) fn settle_origins(
    sim: &mut SimWorld,
    doc: &mut FlipDoc,
    map: &FlipEntityMap,
    gesturing: Option<FlipObjectId>,
) {
    let pending: Vec<(FlipObjectId, Entity)> = map
        .iter()
        .filter(|(id, _)| Some(**id) != gesturing)
        .map(|(&id, &bits)| (id, Entity::from_bits(bits)))
        .filter(|&(_, e)| {
            sim.world().get_entity(e).is_ok()
                && sim.world().get::<ph2d_ecs::ChildOf>(e).is_none()
                && sim
                    .world()
                    .get::<ph2d_ecs::Transform>(e)
                    .is_some_and(|t| *t == ph2d_ecs::Transform::IDENTITY)
        })
        .collect();
    for (id, e) in pending {
        let Some((lo, hi)) = doc.object(id).and_then(|o| o.geometry_bbox()) else {
            continue;
        };
        let center = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
        if center[0] == 0.0 && center[1] == 0.0 {
            continue; // já centrado — nada a fazer
        }
        move_origin_to(sim, doc, e, id, center);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{ChildOf, FlipObjectRef, Name, Transform};
    use ph2d_flip::{FlipStroke, Hold, KeyKind};

    /// Um doc com um objeto (1 camada, 1 desenho) cujo traço vai de `a` a `b`, e a
    /// entidade que o referencia (identidade). Devolve `(doc, sim, map, id, entity)`.
    fn doc_with_segment(
        a: [f32; 2],
        b: [f32; 2],
    ) -> (FlipDoc, SimWorld, FlipEntityMap, FlipObjectId, Entity) {
        let mut doc = FlipDoc::new();
        let oid = doc.push_object("Obj");
        let obj = doc.object_mut(oid).unwrap();
        let l = obj.add_layer("L");
        let d = obj
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let mut s = FlipStroke::new();
        s.push_default(ph2d_core::Vec2::new(a[0], a[1]));
        s.push_default(ph2d_core::Vec2::new(b[0], b[1]));
        obj.drawing_mut(d).unwrap().strokes.push(s);

        let mut sim = SimWorld::default();
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Obj"), FlipObjectRef(oid.0)))
            .id();
        let mut map = FlipEntityMap::new();
        map.insert(oid, e.to_bits());
        (doc, sim, map, oid, e)
    }

    /// O objeto na identidade não entra no mapa de afins (caminho comum: nada a pagar).
    #[test]
    fn an_untransformed_object_is_absent_from_the_map() {
        let (doc, sim, map, _, _) = doc_with_segment([0.0, 0.0], [1.0, 1.0]);
        assert!(build(&sim, &map).is_empty());
    }

    /// Assentar põe o pivô no CENTRO da arte sem mover um pixel dela, e a bbox de
    /// MUNDO (afim ∘ local) não muda. Idempotente.
    #[test]
    fn settling_centers_the_pivot_without_moving_the_art() {
        // Traço de (10,20) a (30,40): centro (20,30).
        let (mut doc, mut sim, map, oid, e) = doc_with_segment([10.0, 20.0], [30.0, 40.0]);
        settle_origins(&mut sim, &mut doc, &map, None);

        let t = sim.world().get::<Transform>(e).copied().unwrap();
        assert!(
            (t.translation.x - 20.0).abs() < 1e-4 && (t.translation.y - 30.0).abs() < 1e-4,
            "pivô no centro: {:?}",
            t.translation
        );
        // A geometria recuou o mesmo tanto: bbox local centrada na origem.
        let (lo, hi) = doc.object(oid).unwrap().geometry_bbox().unwrap();
        assert!((lo[0] + hi[0]).abs() < 1e-4 && (lo[1] + hi[1]).abs() < 1e-4);
        // Afim ∘ local reconstrói o mundo: o canto local sobe para (30,40).
        let x = object_xform(&sim, e);
        let w = x.apply([f64::from(hi[0]), f64::from(hi[1])]);
        assert!(
            (w[0] - 30.0).abs() < 1e-4 && (w[1] - 40.0).abs() < 1e-4,
            "{w:?}"
        );

        // Idempotente.
        settle_origins(&mut sim, &mut doc, &map, None);
        let t2 = sim.world().get::<Transform>(e).copied().unwrap();
        assert_eq!(t2.translation, t.translation);
    }

    /// O objeto EM GESTO não é assentado — a origem pularia a cada amostra e a arte
    /// sairia deslocada do cursor (a mão escreve MUNDO a cada frame).
    #[test]
    fn a_gesturing_object_is_never_settled() {
        let (mut doc, mut sim, map, oid, e) = doc_with_segment([40.0, 40.0], [50.0, 50.0]);
        settle_origins(&mut sim, &mut doc, &map, Some(oid));
        assert_eq!(
            sim.world().get::<Transform>(e).copied().unwrap(),
            Transform::IDENTITY,
            "o gesto continua em mundo: identidade"
        );
        assert!(build(&sim, &map).is_empty(), "afim identidade: sem offset");
        // Terminado o gesto, assenta.
        settle_origins(&mut sim, &mut doc, &map, None);
        assert!((sim.world().get::<Transform>(e).unwrap().translation.x - 45.0).abs() < 1e-4);
    }

    /// Um objeto parentado NÃO é assentado (a origem é herança do pai, não nossa).
    #[test]
    fn a_parented_object_is_left_alone() {
        let (mut doc, mut sim, map, _oid, e) = doc_with_segment([10.0, 20.0], [30.0, 40.0]);
        let parent = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("G")))
            .id();
        sim.world_mut().entity_mut(e).insert(ChildOf(parent));
        settle_origins(&mut sim, &mut doc, &map, None);
        assert_eq!(
            sim.world().get::<Transform>(e).copied().unwrap(),
            Transform::IDENTITY,
            "parentado: a origem é do pai"
        );
    }
}
