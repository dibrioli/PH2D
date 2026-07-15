//! **Blend Objects vivos** (ADR-0122) — o objeto único que interpola 2..=5 formas e as SEGUE.
//!
//! Espelho exato do padrão do [`crate::connector_live`]: o componente [`VecBlend`] guarda a
//! **relação** (quais formas, na ordem, e quantos passos) e os passos intermediários são uma
//! **função pura** dela, re-cozidos a cada frame. Ninguém "desenha" um passo: move-se uma forma
//! fonte, e a transição se refaz.
//!
//! Consequência de graça (a mesma do conector): **undo e save cobrem o blend sem uma linha a
//! mais** — os dois capturam o mundo ECS + a cena vetorial, e o `VecBlend` está registrado no
//! `ComponentRegistry`.
//!
//! # Os passos são VIRTUAIS — o que está na cena é só o SPINE
//!
//! A entidade do blend carrega um `VecPathRef` como qualquer forma; o `VecPath` dela é o **spine**
//! (a linha que une as fontes). Os N passos NÃO entram na cena — a shell os coze aqui, num
//! `Vec<VecPath>` de MUNDO, e um passe de render ([`ph2d_vec_render::draw_blend_overlay`]) os
//! desenha. É o que torna o blend **um objeto**, e não N formas (o pedido do Enio). Consequência:
//! os passos não são pickáveis (igual ao Illustrator — pega-se o objeto, não um passo).
//!
//! # O blend vive na IDENTIDADE (como o conector)
//!
//! O spine e os passos são geometria de MUNDO; uma pose na entidade os deslocaria. Por isso
//! `vec_transform::settle_origins` o **pula** e este módulo devolve o `Transform` à identidade —
//! o que o torna (corretamente) não-arrastável pelo gizmo: mover o blend não quer dizer nada; o
//! que se move são as formas-fonte, e a transição as segue (ADR-0122, o idioma do Illustrator).

use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecBlend};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// A forma assada no MUNDO (ADR-0111: as fontes podem ter poses diferentes, e a transição vive
/// num frame só — como na booleana e no blend destrutivo). `None` se a forma sumiu.
fn world(scene: &VecScene, xforms: &VecXforms, id: VecPathId) -> Option<VecPath> {
    let mut p = scene.paths().iter().find(|p| p.id == id)?.clone();
    bake_xform(&mut p, &xform_of(xforms, id));
    Some(p)
}

/// O centro (da bbox de contorno em MUNDO) de uma forma-fonte. `None` se ela sumiu.
fn center_of(scene: &VecScene, xforms: &VecXforms, id: VecPathId) -> Option<[f64; 2]> {
    let (lo, hi) = scene.path_world_curve_bbox(xforms, id)?;
    Some([(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5])
}

/// O spine default: a polilinha (aberta) que passa pelos centros das fontes, na ordem. É a
/// posição-base dos passos, e o que a Fase C torna editável no modo Node.
fn spine_verts(centers: &[[f64; 2]]) -> Vec<VecVertex> {
    centers.iter().map(|&c| VecVertex::corner(c)).collect()
}

/// Escreve o spine **em lugar** no path `id` — id, estilo (invisível na Fase B) e entidade
/// preservados, como o `write_route` do conector. `centers` vazio ⇒ o spine some (blend sem
/// fontes vivas).
fn write_spine(scene: &mut VecScene, id: VecPathId, centers: &[[f64; 2]]) {
    let Some(p) = scene.path_mut(id) else {
        return;
    };
    p.verts = spine_verts(centers);
    p.closed = false;
    p.subpaths.clear();
}

/// **Cria** um Blend Object sobre `sources` (na ordem de z), com `steps` passos por elo.
///
/// Empurra o spine (a polilinha entre os centros das fontes) na cena e devolve `(spine_id,
/// VecBlend)` para a fila `pending` — a entidade nasce no `vec_entities::sync` do frame, e o
/// [`upkeep`] pendura o componente nela. `None` se não houver 2 fontes que resolvam.
///
/// O spine nasce **invisível** (sem fill nem stroke): na Fase B os PASSOS carregam o visual; o
/// spine visível/editável é a Fase C. A entidade aparece na Hierarquia pelo `Name` ("Blend N").
pub(crate) fn create(
    scene: &mut VecScene,
    xforms: &VecXforms,
    sources: &[VecPathId],
    steps: u32,
) -> Option<(VecPathId, VecBlend)> {
    if sources.len() < 2 {
        return None;
    }
    let centers: Vec<[f64; 2]> = sources
        .iter()
        .filter_map(|&id| center_of(scene, xforms, id))
        .collect();
    if centers.len() < 2 {
        return None;
    }
    let spine = VecPath {
        verts: spine_verts(&centers),
        closed: false,
        ..VecPath::default()
    };
    let spine_id = scene.push_path(spine);
    Some((spine_id, VecBlend::new(sources.to_vec(), steps)))
}

/// **O re-cook de todo frame.** Para cada entidade com um [`VecBlend`]: resolve as fontes no
/// MUNDO, coze os passos (cor interpolada em OKLab pelo motor) para `out`, e atualiza o spine.
///
/// Roda DEPOIS de `vec_entities::sync` (a entidade existe) e depois de `vec_transform::build`
/// (os afins das fontes já são os deste frame), e ANTES do render — o mesmo lugar do
/// `connector_live::recook`.
///
/// `out` é ZERADO aqui e preenchido com o **overlay ordenado** de TODOS os blends, em MUNDO — os
/// passos de cada elo INTERCALADOS com a fonte de cima dele (a pilha de z: fonte0 embaixo → passos
/// → fonte1 → …). É o que o passe [`ph2d_vec_render::draw_blend_overlay`] desenha, nessa ordem.
pub(crate) fn recook(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    out: &mut Vec<VecPath>,
) {
    out.clear();
    let blends: Vec<(VecPathId, Entity, VecBlend)> = map
        .iter()
        .filter_map(|(&id, &bits)| {
            let e = Entity::from_bits(bits);
            let b = sim.world().get::<VecBlend>(e)?.clone();
            Some((id, e, b))
        })
        .collect();
    if blends.is_empty() {
        return;
    }

    for (spine_id, entity, blend) in blends {
        // As fontes que ainda resolvem, na ordem da cadeia. Um elo morto (forma apagada) é
        // PULADO — a cadeia não quebra por causa de um id que sumiu.
        let live: Vec<VecPathId> = blend
            .sources
            .iter()
            .copied()
            .filter(|id| scene.paths().iter().any(|p| p.id == *id))
            .collect();
        if live.len() < 2 {
            write_spine(scene, spine_id, &[]); // sem transição: o spine some
            continue;
        }

        // As fontes assadas no MUNDO. A cadeia é pairwise (fonte[i]→fonte[i+1]); os passos de
        // cada elo entram no overlay INTERCALADOS com a fonte "de cima" dele (`pair[1]`),
        // redesenhada por cima deles. É a pilha de z do Illustrator: fonte0 (que o `dispatch`
        // desenha, embaixo) → passos → fonte1 → passos → fonte2 … Sem intercalar, o passe
        // desenharia TODOS os passos por cima de TODAS as fontes, e a última forma ficaria
        // enterrada sob o último passo (o smoke do Enio: "a última devia ficar acima da
        // penúltima"). A fonte é REDESENHADA (o `dispatch` já a pôs embaixo) porque não dá, na
        // Fase B, para reordenar UM item no meio do `dispatch`; o overdraw de uma forma opaca é
        // barato. A fonte0 fica no z da cena; o interleaving fino contra o resto é da Fase C.
        let worlds: Vec<VecPath> = live
            .iter()
            .filter_map(|&id| world(scene, xforms, id))
            .collect();
        let n = blend.steps as usize;
        for pair in worlds.windows(2) {
            if let Some(plan) = ph2d_vec_blend::Plan::new(&pair[0], &pair[1]) {
                out.extend((1..=n).map(|i| plan.at(i as f64 / (n + 1) as f64)));
            }
            out.push(pair[1].clone()); // a fonte de cima do elo, por cima dos passos dele
        }

        // O spine = a polilinha entre os centros das fontes vivas.
        let centers: Vec<[f64; 2]> = live
            .iter()
            .filter_map(|&id| center_of(scene, xforms, id))
            .collect();
        write_spine(scene, spine_id, &centers);

        // O blend vive na IDENTIDADE (a geometria acima é MUNDO): devolvê-la é o que torna o
        // gizmo inócuo sobre ele — mover o blend não quer dizer nada.
        if sim
            .world()
            .get::<Transform>(entity)
            .is_some_and(|t| *t != Transform::IDENTITY)
            && let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity)
        {
            *t = Transform::IDENTITY;
        }
    }
}

/// Pendura (ou atualiza) o [`VecBlend`] na entidade do path `id` — espelho de
/// `connector_live::attach`. Idempotente (não marca a entidade suja se o componente já é igual).
///
/// `true` se a entidade existia e o componente está lá.
pub(crate) fn attach(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    blend: &VecBlend,
) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let entity = Entity::from_bits(bits);
    if sim.world().get::<VecBlend>(entity) == Some(blend) {
        return true;
    }
    let first = sim.world().get::<VecBlend>(entity).is_none();
    let Ok(mut e) = sim.world_mut().get_entity_mut(entity) else {
        return false;
    };
    e.insert(blend.clone());
    if first {
        // O nome que a Hierarquia mostra — é por ele que o usuário acha o blend na árvore (o
        // spine é invisível na Fase B).
        e.insert(Name::new(format!("Blend {id}")));
    }
    true
}

/// Drena a fila `pending` (o blend recém-criado, esperando a entidade dele nascer no `sync`) —
/// espelho de `connector_live::upkeep`. Roda entre o `sync` e o [`recook`].
///
/// O `pending` é de um item: ou a entidade chegou (attach), ou o path sumiu (undo/delete no
/// mesmo frame) — nos dois casos a fila esvazia.
pub(crate) fn upkeep(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    pending: &mut Option<(VecPathId, VecBlend)>,
) {
    if let Some((id, blend)) = pending.as_ref() {
        let gone = !scene.paths().iter().any(|p| p.id == *id);
        if gone || attach(sim, map, *id, blend) {
            *pending = None;
        }
    }
}

#[cfg(test)]
#[path = "blend_live_tests.rs"]
mod tests;
