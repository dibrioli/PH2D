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
/// É o que faz uma forma-fonte do blend SEGUIR a ponta do spine arrastada no modo Node (ADR-0128
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

/// **Os paths EM GESTO neste frame** — a porta ÚNICA da pergunta *"quem está escrevendo
/// geometria de MUNDO agora?"*.
///
/// Todo gesto de autoria à mão (a caneta, a ferramenta de forma, o **lápis**) reescreve a
/// geometria do seu path em coordenadas de MUNDO a cada frame. Assentar um deles no meio do
/// gesto soma geometria + `Transform` e a tinta sai deslocada do cursor exatamente pelo
/// ponto onde o arrasto começou — **medido: 1,5897 unidades de mundo (≈353 px) num arrasto
/// que começou a 1,5 do centro**, e o erro cresce com a distância à origem do mundo.
///
/// ⚠️ **Por que uma porta e não uma lista no chamador.** Esta lista nasceu enumerando *"a
/// caneta e a ferramenta de forma"* — as duas que existiam. O lápis chegou como o TERCEIRO
/// e a enumeração não o conhecia: a condição envelheceu em silêncio, com o gesto novo
/// funcionando no primeiro frame (o `settle` corre antes do render) e deslocado em todos os
/// seguintes ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
///
/// E os gestos entram por **PARÂMETRO**, não numa lista montada no chamador: o gesto nº 4
/// acrescenta um parâmetro, o que é um **erro de compilação** no único sítio de chamada —
/// enquanto um `Vec` literal aceita em silêncio a lista incompleta, que é exactamente como
/// esta nasceu.
///
/// ⚠️ **O Offset saiu desta lista de propósito** (2026-07-21): o preview dele deixou de
/// tocar a cena — a forma que está no documento é a AUTORADA, com a pose dela, e o
/// resultado é geometria derivada que nunca entra aqui. Enquanto o preview churnava a cena,
/// o resultado renascia com o mesmo id todo frame e precisava ser pulado, senão mundo ×
/// centro dobrava a pose (o *"pula pro canto direito"*). Sem churn, não há o que pular.
#[must_use]
pub(crate) fn gesture_paths(
    pen: &ph2d_vec_edit::PenTool,
    shape: &ph2d_vec_edit::ShapeTool,
    pencil: &ph2d_vec_edit::Pencil,
) -> Vec<ph2d_vec_scene::VecPathId> {
    [pen.active_path(), shape.active_path(), pencil.active_path()]
        .into_iter()
        .flatten()
        .collect()
}

/// **Pendura `child` em `parent` SEM mover a forma.**
///
/// ⚠️ Um `ChildOf` cru desloca o filho pela pose do pai, e o modo de falha é silencioso e grande:
/// depois do [`settle_origins`] toda forma-raiz carrega a própria translação, então prender uma a
/// outra soma as duas e o filho salta o centro do pai. Medido numa cena de smoke desta linha: uma
/// barra desenhada dentro de um corpo de 8 unidades aterrava **5 unidades à esquerda**, fora dele.
///
/// A pose local que devolve o filho ao sítio é `inverse_compose(pose_do_pai, pose_de_mundo_dele)` —
/// a mesma porta que a física usa para ler um corpo-filho ([`ph2d_ecs::Transform::inverse_compose`],
/// exacta sob rotação, escala não-uniforme e cisalhamento). `false` quando alguma das entidades
/// sumiu ou o afim do pai é degenerado — e aí **nada é escrito**, porque prender sem saber pôr de
/// volta deixaria a forma num sítio que ninguém autorou.
pub(crate) fn reparent_keeping_world(sim: &mut SimWorld, child: Entity, parent: Entity) -> bool {
    if sim.world().get_entity(child).is_err() || sim.world().get_entity(parent).is_err() {
        return false;
    }
    let world = world_transform(sim, child);
    let parent_world = world_transform(sim, parent);
    let Some(local) = Transform::inverse_compose(parent_world, world) else {
        return false;
    };
    if let Ok(mut e) = sim.world_mut().get_entity_mut(child) {
        e.insert((local, ph2d_ecs::ChildOf(parent)));
        return true;
    }
    false
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
                // BLEND OBJECT (ADR-0128): pela MESMA razão do conector. O spine é geometria de
                // MUNDO, reescrita a cada frame (`blend_live`) a partir dos centros das fontes —
                // assentar somaria geometria + `Transform` e o deslocaria. Ele vive na
                // identidade, e é isso que o torna não-arrastável pelo gizmo: mover um blend não
                // quer dizer nada; o que se move são as formas-fonte.
                && sim.world().get::<ph2d_ecs::VecBlend>(e).is_none()
                // MORPH OBJECT: idem — a forma morfada é reescrita em MUNDO a cada frame
                // (`morph_live`) a partir das duas fontes.
                && sim.world().get::<ph2d_ecs::VecMorph>(e).is_none()
                // ENVELOPE OBJECT (ADR-0129): idem — a forma é a fonte autorada deformada pela
                // gaiola, reescrita em MUNDO a cada frame (`envelope_live`). Vive na identidade.
                && sim.world().get::<ph2d_ecs::VecEnvelope>(e).is_none()
                // ⭐ PELE (estudo 42 item 5): idem — a forma é a fonte autorada deformada pelos
                // ossos, reescrita a cada quadro (`skin_live`). Assentar o pivô no meio da
                // DEFORMADA faria este sistema escrever no documento num quadro que o artista não
                // provocou, e o centro de um objecto é propriedade da identidade dele, não da pose
                // em que ele está agora.
                && sim.world().get::<ph2d_ecs::VecSkin>(e).is_none()
                && sim
                    .world()
                    .get::<Transform>(e)
                    .is_some_and(|t| *t == Transform::IDENTITY)
        })
        .collect();
    for (id, e) in pending {
        // ⚠️ A bbox **AUTORADA** (`path_bbox`), não a cozida.
        //
        // O `path_curve_bbox` passou a medir a geometria COZIDA quando os Live Path Effects
        // entraram (ADR-0132) — o que está certo para o GIZMO, que tem de abraçar o que se vê.
        // Para o PIVÔ está errado: o centro de um objeto é uma propriedade da identidade dele,
        // não da aparência de hoje. Com o cozido, acrescentar um Trim ou um Repeater desloca a
        // bbox e faz este sistema **escrever no documento** num frame que o utilizador não
        // provocou — um escritor por-frame que reage a efeitos, que é exatamente a forma de um
        // passo de undo espúrio (a classe que o `vec_zorder_fixpoint_tests` já apanhou uma vez).
        let Some((lo, hi)) = scene.path_bbox(id) else {
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
#[path = "vec_transform_tests.rs"]
mod tests;

/// O lápis contra a ordem REAL do frame — o gate do defeito de POSE (o "offset do mouse").
/// Arquivo irmão porque este já hospeda a suíte do assentamento, e as duas medem coisas
/// diferentes: aquela o pivô, esta a tinta sob o dedo.
#[cfg(test)]
#[path = "vec_pencil_frame_tests.rs"]
mod pencil_frame_tests;

#[cfg(test)]
#[path = "vec_transform_reparent_tests.rs"]
mod reparent_tests;
