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

/// Move a ORIGEM da entidade de `path` para `target_world`, **sem mover a forma**:
/// a translação vai para lá e a geometria local desloca o mesmo tanto para trás.
///
/// É o que dá sentido ao pivô de uma forma vetorial. Um sprite tem a origem no
/// centro do quad por construção (`anchor = 0`); um path nasce com a geometria em
/// coordenadas de mundo e a origem em (0,0) — o centro do MUNDO, não o da forma.
///
/// Usado em dois lugares, com o mesmo código: ao assentar um path recém-criado
/// (`target` = centro da bbox) e no botão "Set Center" (`target` = o clique).
///
/// `false` se a entidade sumiu, se o path sumiu, ou se o afim é degenerado.
pub(crate) fn move_origin_to(
    sim: &mut SimWorld,
    scene: &mut ph2d_vec_scene::VecScene,
    entity: Entity,
    path: ph2d_vec_scene::VecPathId,
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
    let Some(local) = sim.world().get::<Transform>(entity).copied() else {
        return false;
    };
    // Quanto a origem andou, no espaço do pai.
    let delta_parent = [
        target_parent[0] - f64::from(local.translation.x),
        target_parent[1] - f64::from(local.translation.y),
    ];
    // O mesmo deslocamento, no espaço LOCAL da geometria: desfaz a rotação/escala
    // próprias (a translação não entra — delta é um vetor).
    let rs = xform_of_transform(Transform {
        translation: ph2d_core::Vec2::new(0.0, 0.0),
        ..local
    });
    let Some(rs_inv) = rs.inverse() else {
        return false;
    };
    let delta_local = rs_inv.apply_vec(delta_parent);
    let Some(p) = scene.path_mut(path) else {
        return false;
    };
    // A geometria recua exatamente o que a origem avançou ⇒ a forma não se move.
    ph2d_vec_scene::bake_xform(
        p,
        &Xform([1.0, 0.0, 0.0, 1.0, -delta_local[0], -delta_local[1]]),
    );
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
        t.translation = ph2d_core::Vec2::new(target_parent[0] as f32, target_parent[1] as f32);
    }
    true
}

/// **Translada a FORMA** de `entity` por `delta_world` (um vetor em MUNDO) — o gizmo-move reduzido
/// a uma translação, sem escala/rotação, e o OPOSTO do [`move_origin_to`] (a geometria NÃO recua,
/// então a forma SE MOVE). Por ser uma DELTA (e não um alvo absoluto), não presume nada sobre onde
/// a origem está — vale settled ou não.
///
/// É o que faz uma forma-fonte do blend SEGUIR a ponta do spine arrastada no modo Node (ADR-0122
/// C2b): arrastar a ponta é o MESMO que mover a forma pelo gizmo. `false` se a entidade sumiu ou o
/// afim do pai é degenerado.
pub(crate) fn translate_shape_world(
    sim: &mut SimWorld,
    entity: Entity,
    delta_world: [f64; 2],
) -> bool {
    if sim.world().get_entity(entity).is_err() {
        return false;
    }
    // A translação vive no espaço do PAI; a delta é um VETOR, então só a parte linear do afim do
    // pai a converte (a translação do pai não entra).
    let parent = ph2d_ecs::parent_world_transform(sim.world(), entity);
    let Some(parent_inv) = xform_of_transform(parent).inverse() else {
        return false;
    };
    let dp = parent_inv.apply_vec(delta_world);
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) else {
        return false;
    };
    t.translation = ph2d_core::Vec2::new(
        t.translation.x + dp[0] as f32,
        t.translation.y + dp[1] as f32,
    );
    true
}

/// Põe a origem de cada path recém-criado no **centro da bbox dele**.
///
/// Um path nasce com a geometria em coordenadas de mundo e a entidade na
/// identidade — a origem, e portanto o pivô, cai no centro do mundo. Isto conserta
/// isso assim que a forma pára de crescer (`drawing` = o path que a caneta ainda
/// está construindo, que ficaria pulando a cada vértice).
///
/// Só toca quem está na **identidade e sem pai** — um path já movido, escalado ou
/// parentado tem a origem que o usuário lhe deu, e não é da nossa conta. Idempotente:
/// depois de centrado, o delta é zero.
///
/// `drawing` são os paths em GESTO: o da caneta e o da ferramenta de forma. Ambos
/// escrevem geometria em coordenadas de MUNDO a cada frame; assentá-los no meio do
/// gesto faria a geometria e o `Transform` somarem, e a forma sairia deslocada do
/// cursor exatamente pelo ponto onde o arrasto começou.
pub(crate) fn settle_origins(
    sim: &mut SimWorld,
    scene: &mut ph2d_vec_scene::VecScene,
    map: &crate::vec_entities::VecEntityMap,
    drawing: &[ph2d_vec_scene::VecPathId],
) {
    let pending: Vec<(ph2d_vec_scene::VecPathId, Entity)> = map
        .iter()
        .filter(|(id, _)| !drawing.contains(id))
        .map(|(&id, &bits)| (id, Entity::from_bits(bits)))
        .filter(|&(_, e)| {
            sim.world().get_entity(e).is_ok()
                && sim.world().get::<ph2d_ecs::ChildOf>(e).is_none()
                // Live Shapes (texto/forma paramétrica): geometria DERIVADA dos
                // parâmetros e re-cozinhada — assentar o pivô no meio brigaria com o
                // re-cook (a origem fica onde a forma foi criada; "Set Center" move).
                && sim.world().get::<ph2d_ecs::VecShape>(e).is_none()
                // CONECTOR: pela MESMA razão, e ainda mais forte. A geometria dele é
                // reescrita em MUNDO a cada frame (`connector_live`), como a de um gesto
                // que nunca termina — assentar somaria geometria + `Transform` e a rota
                // sairia deslocada das formas que ela liga. Ele vive na identidade, e é
                // isso que o torna (corretamente) não-arrastável pelo gizmo: mover um
                // conector não quer dizer nada; o que se move são as pontas dele.
                && sim.world().get::<ph2d_ecs::VecConnector>(e).is_none()
                // BLEND OBJECT (ADR-0122): pela MESMA razão do conector. O spine é geometria de
                // MUNDO, reescrita a cada frame (`blend_live`) a partir dos centros das fontes —
                // assentar somaria geometria + `Transform` e o deslocaria. Ele vive na
                // identidade, e é isso que o torna não-arrastável pelo gizmo: mover um blend não
                // quer dizer nada; o que se move são as formas-fonte.
                && sim.world().get::<ph2d_ecs::VecBlend>(e).is_none()
                // MORPH OBJECT: idem — a forma morfada é reescrita em MUNDO a cada frame
                // (`morph_live`) a partir das duas fontes.
                && sim.world().get::<ph2d_ecs::VecMorph>(e).is_none()
                // ENVELOPE OBJECT (ADR-0123): idem — a forma é a fonte autorada deformada pela
                // gaiola, reescrita em MUNDO a cada frame (`envelope_live`). Vive na identidade.
                && sim.world().get::<ph2d_ecs::VecEnvelope>(e).is_none()
                && sim
                    .world()
                    .get::<Transform>(e)
                    .is_some_and(|t| *t == Transform::IDENTITY)
        })
        .collect();
    for (id, e) in pending {
        let Some((lo, hi)) = scene.path_curve_bbox(id) else {
            continue;
        };
        let center = [
            ((lo[0] + hi[0]) * 0.5) as f32,
            ((lo[1] + hi[1]) * 0.5) as f32,
        ];
        if center[0] == 0.0 && center[1] == 0.0 {
            continue; // já centrado — nada a fazer
        }
        move_origin_to(sim, scene, e, id, center);
    }
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

    /// Assentar a origem põe o pivô no CENTRO da forma sem mover um pixel dela.
    /// Era o bug: um path nascia com a geometria em coordenadas de mundo e a origem
    /// em (0,0) — o centro do mundo.
    #[test]
    fn settling_puts_the_origin_at_the_shape_center_without_moving_the_shape() {
        let mut sim = SimWorld::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let mut map = crate::vec_entities::VecEntityMap::new();
        // Quadrado de [10,20] a [30,40]: centro em (20, 30).
        let id = scene.push_path(ph2d_vec_scene::rectangle([10.0, 20.0], [30.0, 40.0]));
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, VecPathRef(id)))
            .id();
        map.insert(id, e.to_bits());

        let world_before = scene.path_curve_bbox(id).unwrap();
        settle_origins(&mut sim, &mut scene, &map, &[]);

        let t = sim.world().get::<Transform>(e).copied().unwrap();
        assert!(
            (t.translation.x - 20.0).abs() < 1e-4 && (t.translation.y - 30.0).abs() < 1e-4,
            "a origem foi para o centro da forma: {:?}",
            t.translation
        );
        // A geometria recuou o mesmo tanto ⇒ o bbox de MUNDO não mudou.
        let x = ph2d_vec_scene::xform_of(&build(&sim, &map), id);
        let (lo, hi) = scene.path_curve_bbox(id).unwrap();
        let (wlo, whi) = (x.apply(lo), x.apply(hi));
        assert!((wlo[0] - world_before.0[0]).abs() < 1e-4, "{wlo:?}");
        assert!((whi[1] - world_before.1[1]).abs() < 1e-4, "{whi:?}");
        // E a bbox local ficou centrada na origem.
        assert!((lo[0] + hi[0]).abs() < 1e-4 && (lo[1] + hi[1]).abs() < 1e-4);

        // Idempotente: rodar de novo não mexe em nada.
        settle_origins(&mut sim, &mut scene, &map, &[]);
        let t2 = sim.world().get::<Transform>(e).copied().unwrap();
        assert_eq!(t2.translation, t.translation);
    }

    /// REGRESSÃO (Enio 2026-07-09: "forma sendo desenhada com pivot no centro do mundo
    /// e com um offset bizarro em relação ao mouse").
    ///
    /// A ferramenta de forma empurra o path no Down e **reescreve a geometria em
    /// coordenadas de MUNDO** a cada Move. Se a origem for assentada no meio do gesto,
    /// geometria e `Transform` passam a somar, e a forma sai deslocada exatamente pelo
    /// ponto onde o arrasto começou.
    #[test]
    fn a_shape_still_being_dragged_is_never_settled() {
        let mut sim = SimWorld::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let mut map = crate::vec_entities::VecEntityMap::new();
        // O degenerado do Down: um quadrado minúsculo longe da origem.
        let id = scene.push_path(ph2d_vec_scene::rectangle([40.0, 40.0], [40.0, 40.0]));
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, VecPathRef(id)))
            .id();
        map.insert(id, e.to_bits());

        settle_origins(&mut sim, &mut scene, &map, &[id]);
        assert_eq!(
            sim.world().get::<Transform>(e).copied().unwrap(),
            Transform::IDENTITY,
            "o gesto continua em world-space: identidade"
        );
        assert!(build(&sim, &map).is_empty(), "afim identidade: sem offset");

        // Terminado o gesto, aí sim assenta.
        settle_origins(&mut sim, &mut scene, &map, &[]);
        let t = sim.world().get::<Transform>(e).copied().unwrap();
        assert!((t.translation.x - 40.0).abs() < 1e-4);
    }

    /// **Um CONECTOR nunca é assentado.** A geometria dele é reescrita em MUNDO a
    /// cada frame (`connector_live`) — assentar o pivô somaria geometria e
    /// `Transform`, e a rota sairia deslocada das duas formas que ela liga. Mesmo
    /// motivo do gesto em curso, e da forma viva: geometria DERIVADA não tem pivô a
    /// assentar. Ele fica na identidade — e é por isso que o gizmo não o arrasta.
    #[test]
    fn a_connector_is_never_settled() {
        let mut sim = SimWorld::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let mut map = crate::vec_entities::VecEntityMap::new();
        // Uma "rota" longe da origem: assentada, ela ganharia translação (20, 30).
        let id = scene.push_path(ph2d_vec_scene::line([10.0, 20.0], [30.0, 40.0]));
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                VecPathRef(id),
                ph2d_ecs::VecConnector::between(1, 2),
            ))
            .id();
        map.insert(id, e.to_bits());

        settle_origins(&mut sim, &mut scene, &map, &[]);

        assert_eq!(
            sim.world().get::<Transform>(e).copied().unwrap(),
            Transform::IDENTITY,
            "o conector tem de ficar na IDENTIDADE — a geometria dele ja e mundo"
        );
        assert!(build(&sim, &map).is_empty(), "afim identidade: sem offset");
    }

    /// O path que a caneta ainda constrói NÃO é assentado — a origem ficaria pulando
    /// a cada vértice novo.
    #[test]
    fn the_path_being_drawn_is_left_alone() {
        let mut sim = SimWorld::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let mut map = crate::vec_entities::VecEntityMap::new();
        let id = scene.push_path(ph2d_vec_scene::rectangle([10.0, 20.0], [30.0, 40.0]));
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, VecPathRef(id)))
            .id();
        map.insert(id, e.to_bits());
        settle_origins(&mut sim, &mut scene, &map, &[id]);
        assert_eq!(
            sim.world().get::<Transform>(e).copied().unwrap(),
            Transform::IDENTITY
        );
    }

    /// `move_origin_to` respeita a escala própria da forma: a geometria local recua
    /// o delta DIVIDIDO pela escala, senão a forma saltaria.
    #[test]
    fn moving_the_origin_of_a_scaled_shape_does_not_move_the_shape() {
        let mut sim = SimWorld::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let id = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -1.0], [1.0, 1.0]));
        let e = sim
            .world_mut()
            .spawn((t(0.0, 0.0, 3.0), VecPathRef(id)))
            .id();
        let mut map = crate::vec_entities::VecEntityMap::new();
        map.insert(id, e.to_bits());

        let x0 = ph2d_vec_scene::xform_of(&build(&sim, &map), id);
        let corner_before = x0.apply([1.0, 1.0]); // mundo (3,3)

        assert!(move_origin_to(&mut sim, &mut scene, e, id, [3.0, 3.0]));

        let x1 = ph2d_vec_scene::xform_of(&build(&sim, &map), id);
        let corner_after = x1.apply(scene.paths()[0].verts_all().nth(2).unwrap().anchor);
        assert!(
            (corner_after[0] - corner_before[0]).abs() < 1e-4
                && (corner_after[1] - corner_before[1]).abs() < 1e-4,
            "o canto não se moveu: {corner_before:?} -> {corner_after:?}"
        );
        let tr = sim
            .world()
            .get::<Transform>(e)
            .copied()
            .unwrap()
            .translation;
        assert!(
            (tr.x - 3.0).abs() < 1e-4 && (tr.y - 3.0).abs() < 1e-4,
            "{tr:?}"
        );
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
