//! O `Transform` das formas vetoriais (ADR-0111).
//!
//! Desde o ADR-0110 cada path é uma entidade. Aqui ela ganha o que faltava para ser
//! um objeto de verdade: **pose**. A geometria em `VecScene` passa a ser LOCAL, e o
//! afim que a leva ao mundo é `parent_world_transform ∘ Transform` — a mesma cadeia
//! de um sprite, computada pelo mesmo helper.
//!
//! Consequência que vale o preço: um path pode ser filho de qualquer coisa e é
//! movido/girado/escalado pelo **gizmo de sprite**, individualmente ou dentro de
//! uma multi-seleção mista. Não há gizmo vetorial próprio — havia, e foi removido.
//!
//! Identidade ⇒ local é mundo. Todo path recém-desenhado nasce assim, então nada
//! muda para quem só desenha.

use ph2d_ecs::{Entity, GlobalTransform, SimWorld, Transform};
use ph2d_vec_scene::{VecXforms, Xform};

/// A pose de `entity` no mundo: a cadeia de pais, depois a local.
///
/// Reusa `parent_world_transform` — o mesmo caminho que o drag do gizmo de sprite
/// já percorre, então um path e um sprite irmãos concordam por construção.
#[must_use]
pub(crate) fn world_transform(sim: &SimWorld, entity: Entity) -> Transform {
    let local = sim
        .world()
        .get::<Transform>(entity)
        .copied()
        .unwrap_or(Transform::IDENTITY);
    Transform::compose(ph2d_ecs::parent_world_transform(sim.world(), entity), local)
}

/// O afim local→mundo de uma pose. Passa por `GlobalTransform` para herdar a MESMA
/// matemática dos sprites — incluindo skew e o `libm::sincosf` que mantém o
/// resultado bit-idêntico entre sistemas (HR-5).
#[must_use]
pub(crate) fn xform_of_transform(t: Transform) -> Xform {
    let a = GlobalTransform::from_transform(t).affine();
    Xform([
        f64::from(a[0]),
        f64::from(a[1]),
        f64::from(a[2]),
        f64::from(a[3]),
        f64::from(a[4]),
        f64::from(a[5]),
    ])
}

/// O afim de cada path do documento, uma vez por frame. Um path cuja entidade está
/// na identidade **não entra no mapa** — `xform_of` devolve identidade e o caminho
/// comum não paga nem um lookup.
#[must_use]
pub(crate) fn build(sim: &SimWorld, map: &crate::vec_entities::VecEntityMap) -> VecXforms {
    let mut out = VecXforms::new();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let x = xform_of_transform(world_transform(sim, e));
        if !x.is_identity() {
            out.insert(id, x);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{ChildOf, Name, VecPathRef};

    fn t(tx: f32, ty: f32, scale: f32) -> Transform {
        Transform {
            translation: ph2d_core::Vec2::new(tx, ty),
            scale: ph2d_core::Vec2::new(scale, scale),
            ..Transform::IDENTITY
        }
    }

    /// Identidade não entra no mapa: o caminho comum (todo path recém-desenhado)
    /// não paga nada, e `xform_of` já devolve identidade para o ausente.
    #[test]
    fn an_untransformed_path_is_absent_from_the_map() {
        let mut sim = SimWorld::default();
        let mut map = crate::vec_entities::VecEntityMap::new();
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("P"), VecPathRef(7)))
            .id();
        map.insert(7, e.to_bits());
        assert!(build(&sim, &map).is_empty());
        assert_eq!(
            ph2d_vec_scene::xform_of(&build(&sim, &map), 7),
            Xform::IDENTITY
        );
    }

    /// A pose de mundo COMPÕE a cadeia de pais — é o que faz um path parentado a um
    /// sprite andar junto com ele.
    #[test]
    fn a_child_path_inherits_the_pose_of_its_parent() {
        let mut sim = SimWorld::default();
        let mut map = crate::vec_entities::VecEntityMap::new();
        let parent = sim
            .world_mut()
            .spawn((t(10.0, 0.0, 2.0), Name::new("S")))
            .id();
        let child = sim
            .world_mut()
            .spawn((
                t(1.0, 1.0, 1.0),
                Name::new("P"),
                VecPathRef(7),
                ChildOf(parent),
            ))
            .id();
        map.insert(7, child.to_bits());

        let x = ph2d_vec_scene::xform_of(&build(&sim, &map), 7);
        // pai escala 2× e translada +10 em x; o filho translada (1,1) no espaço do pai.
        // A origem local do filho cai em (10 + 2·1, 0 + 2·1) = (12, 2).
        let o = x.apply([0.0, 0.0]);
        assert!(
            (o[0] - 12.0).abs() < 1e-6 && (o[1] - 2.0).abs() < 1e-6,
            "{o:?}"
        );
        // E um ponto local [1,0] anda 2 world-units (a escala do pai).
        let p = x.apply([1.0, 0.0]);
        assert!((p[0] - 14.0).abs() < 1e-6, "{p:?}");
    }

    /// Uma entidade morta não produz afim (a `sync` a removeria, mas o mapa pode
    /// estar um frame atrasado).
    #[test]
    fn a_dead_entity_contributes_nothing() {
        let mut sim = SimWorld::default();
        let mut map = crate::vec_entities::VecEntityMap::new();
        let e = sim
            .world_mut()
            .spawn((t(5.0, 5.0, 1.0), VecPathRef(1)))
            .id();
        map.insert(1, e.to_bits());
        assert_eq!(build(&sim, &map).len(), 1);
        sim.world_mut().despawn(e);
        assert!(build(&sim, &map).is_empty());
    }
}
