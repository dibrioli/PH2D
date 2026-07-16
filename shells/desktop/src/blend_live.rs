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

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecBlend};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// As operações de edição/interação do blend (painel + canvas) — módulo irmão pelo teto de 600 LOC.
#[path = "blend_live_edit.rs"]
mod edit;
pub(crate) use edit::{
    drag_spine_anchors_move_sources, expand, pick_preview, release, reset_spine,
    selected_closed_in_z, set_selected_steps,
};

/// A memória do spine AUTOMÁTICO que a shell escreveu por último, por blend. É como a shell
/// detecta que o artista editou a curva (modo Node): se o spine ATUAL difere do último auto, a
/// mão mexeu → o blend vira `spine_authored`. Runtime-only (o flag em si viaja no componente).
pub(crate) type BlendSpines = BTreeMap<VecPathId, Vec<VecVertex>>;

/// Translada TODOS os pontos de um path (âncora + as duas alças) por `off`. É como um passo é
/// movido do seu lugar do lerp para o lugar dele no spine.
fn translate_verts(path: &mut VecPath, off: [f64; 2]) {
    for v in &mut path.verts {
        for p in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
            p[0] += off[0];
            p[1] += off[1];
        }
    }
}

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
/// posição-base dos passos, e o que o modo Node torna editável (ADR-0122).
fn spine_verts(centers: &[[f64; 2]]) -> Vec<VecVertex> {
    centers.iter().map(|&c| VecVertex::corner(c)).collect()
}

/// O traço FINO do spine — é o que o torna visível e selecionável/clicável no canvas (para editar
/// no modo Node). É **dado de documento** (um `StrokeSpec` de um path, como o fill de uma forma),
/// não chrome de UI. Cinza sutil, largura pequena em MUNDO. (Um guia por-overlay ancorado à
/// seleção, com token e largura em px, é um refinamento — ADR-0122.)
fn spine_stroke() -> ph2d_vec_scene::StrokeSpec {
    ph2d_vec_scene::StrokeSpec::new(ph2d_vec_scene::Rgba8::new(150, 150, 165, 190), 0.03)
}

/// Fixa as ÂNCORAS do spine autorado aos centros das fontes — cada âncora pertence a uma forma (o
/// Illustrator: a curva se edita pelas ALÇAS, não movendo a âncora para fora da forma). As alças
/// acompanham (a tangente é preservada, `shift_vertex_to`), então uma fonte que se move **arrasta a
/// âncora e a curva junto** — inclusive as fontes do MEIO da cadeia ([`edit::anchor_source_pairs`]).
/// Sem isto, uma fonte movida no Select descolaria os passos da sua âncora.
///
/// `live` são as formas vivas na ordem da cadeia (uma por âncora, no caso normal). Passar os centros
/// direto não bastava: com pontos de dobra extras, âncora ≠ fonte por índice.
fn pin_spine_anchors(
    scene: &mut VecScene,
    id: VecPathId,
    live: &[VecPathId],
    centers: &[[f64; 2]],
) {
    let Some(p) = scene.path_mut(id) else {
        return;
    };
    // O mapa usa o Nº de fontes; os centros vêm na MESMA ordem de `live`, então o índice em `live`
    // indexa `centers`. O mapa mora no módulo irmão `edit` (junto com quem mais o usa).
    for (vi, src_id) in edit::anchor_source_pairs(p.verts.len(), live) {
        let Some(li) = live.iter().position(|&s| s == src_id) else {
            continue;
        };
        let (Some(v), Some(&c)) = (p.verts.get_mut(vi), centers.get(li)) else {
            continue;
        };
        shift_vertex_to(v, c);
    }
}

/// Move a âncora do vértice para `anchor`, arrastando as duas alças pelo mesmo delta (a tangente
/// fica igual — a ponta translada inteira).
fn shift_vertex_to(v: &mut VecVertex, anchor: [f64; 2]) {
    let d = [anchor[0] - v.anchor[0], anchor[1] - v.anchor[1]];
    v.anchor = anchor;
    for h in [&mut v.in_handle, &mut v.out_handle] {
        h[0] += d[0];
        h[1] += d[1];
    }
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
        // Invisível na cena: a linha só aparece no modo Node, elevada ao overlay (`elevate_spines`).
        // No Select ela é Node-only. O `recook` mantém o traço em `None` todo frame.
        stroke: None,
        ..VecPath::default()
    };
    let spine_id = scene.push_path(spine);
    Some((spine_id, VecBlend::new(sources.to_vec(), steps)))
}

/// O teto de fontes por blend (o "até 5 formas" do Enio, ADR-0122). O motor aceita mais, mas o
/// idioma do Illustrator é uma cadeia curta.
pub(crate) const MAX_BLEND_SOURCES: usize = 5;

/// As fontes de um blend que ainda RESOLVEM, na ordem da cadeia. Um elo morto (forma apagada) é
/// PULADO — a cadeia não quebra por causa de um id que sumiu.
fn live_sources(scene: &VecScene, blend: &VecBlend) -> Vec<VecPathId> {
    blend
        .sources
        .iter()
        .copied()
        .filter(|id| scene.paths().iter().any(|p| p.id == *id))
        .collect()
}

/// Os deslocamentos dos passos ao longo do spine AUTORADO — a posição de cada passo deixa de ser o
/// lerp e passa a ser um ponto da CURVA, por comprimento de arco. Vazio (⇒ sem deslocamento) quando
/// o spine é automático: aí os passos seguem o lerp puro, byte-idêntico à Fase B.
///
/// **Só LÊ o spine** (não o reescreve nem o pina) — é o que permite ao [`edit::expand`] chamá-la
/// sobre o estado que o [`recook`] deste frame já assentou.
fn spine_offsets_of(
    scene: &VecScene,
    spine_id: VecPathId,
    centers: &[[f64; 2]],
    blend: &VecBlend,
) -> Vec<[f64; 2]> {
    if !blend.spine_authored {
        return Vec::new();
    }
    scene
        .paths()
        .iter()
        .find(|p| p.id == spine_id)
        .map(|sp| ph2d_vec_blend::spine_offsets(sp, centers, blend.steps as usize))
        .unwrap_or_default()
}

/// Os passos de um blend, **agrupados por elo** (`[[passo…], [passo…]]`), em MUNDO e já deslocados
/// para os seus lugares no spine.
///
/// É a **única porta que produz um passo**: o [`recook`] os intercala com as fontes para o overlay,
/// e o [`edit::expand`] os materializa na cena. Duas portas divergiriam — e aqui a divergência seria
/// VISÍVEL: as formas **saltariam** no instante do Expand, que é justamente a operação que promete
/// entregar o que estava na tela.
///
/// `offsets` vazio ⇒ sem deslocamento (spine automático). Um elo cujo `Plan` não resolve (forma
/// degenerada) devolve zero passos, sem derrubar os outros.
fn cook_links(worlds: &[VecPath], n: usize, offsets: &[[f64; 2]]) -> Vec<Vec<VecPath>> {
    worlds
        .windows(2)
        .enumerate()
        .map(|(i, pair)| {
            let Some(plan) = ph2d_vec_blend::Plan::new(&pair[0], &pair[1]) else {
                return Vec::new();
            };
            (1..=n)
                .map(|j| {
                    let mut step = plan.at(j as f64 / (n + 1) as f64);
                    if let Some(off) = offsets.get(i * n + (j - 1)) {
                        translate_verts(&mut step, *off);
                    }
                    step
                })
                .collect()
        })
        .collect()
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
///
/// # O SPINE: automático ou AUTORADO (ADR-0122)
///
/// Enquanto o artista não edita o spine, a shell o regenera (a reta pelos centros) e os passos
/// seguem o **lerp** (byte-idêntico à Fase B). Quando o artista edita a curva no modo Node, a
/// detecção (spine atual ≠ último auto escrito, em `spines`) marca `spine_authored`, a shell PARA
/// de sobrescrever, e os passos passam a **FLUIR ao longo do spine** por comprimento de arco
/// ([`ph2d_vec_blend::spine_offsets`]).
pub(crate) fn recook(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    spines: &mut BlendSpines,
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
    spines.retain(|id, _| blends.iter().any(|(b, _, _)| b == id));
    if blends.is_empty() {
        return;
    }

    for (spine_id, entity, mut blend) in blends {
        let live = live_sources(scene, &blend);
        if live.len() < 2 {
            write_spine(scene, spine_id, &[]); // sem transição: o spine some
            spines.remove(&spine_id);
            continue;
        }

        let worlds: Vec<VecPath> = live
            .iter()
            .filter_map(|&id| world(scene, xforms, id))
            .collect();
        let centers: Vec<[f64; 2]> = live
            .iter()
            .filter_map(|&id| center_of(scene, xforms, id))
            .collect();

        // O spine: detecta a edição (atual ≠ último auto) e, se autorado, os passos fluem por ele.
        let current: Vec<VecVertex> = scene
            .paths()
            .iter()
            .find(|p| p.id == spine_id)
            .map(|p| p.verts.clone())
            .unwrap_or_default();
        let mut authored = blend.spine_authored;
        if !authored && spines.get(&spine_id).is_some_and(|last| *last != current) {
            authored = true; // a mão mexeu na curva (modo Node)
            if let Some(mut b) = sim.world_mut().get_mut::<VecBlend>(entity) {
                b.spine_authored = true; // persiste (viaja no save/undo)
            }
        }
        blend.spine_authored = authored; // a cópia local acompanha o componente
        let offsets = if authored {
            // As âncoras seguem as fontes (a curva se edita pelas alças); depois os passos FLUEM ao
            // longo do spine editado — deslocamento por comprimento de arco.
            pin_spine_anchors(scene, spine_id, &live, &centers);
            spine_offsets_of(scene, spine_id, &centers, &blend)
        } else {
            // Spine automático (a reta pelos centros): escreve e MEMORIZA (para detectar a edição
            // no frame seguinte). Sem deslocamento — os passos seguem o lerp, byte-idêntico.
            write_spine(scene, spine_id, &centers);
            let auto = scene
                .paths()
                .iter()
                .find(|p| p.id == spine_id)
                .map(|p| p.verts.clone())
                .unwrap_or_default();
            spines.insert(spine_id, auto);
            Vec::new()
        };

        // O spine é INVISÍVEL na cena — ele só aparece no modo Node, elevado ao topo do overlay
        // (`elevate_spines`), que é o único modo em que a linha se toca (ADR-0122). No Select a linha
        // é Node-only e não deve aparecer: mantê-la traçada na cena a mostrava como um "fantasma" que
        // ainda dava drift ao mover as formas (Enio 2026-07-15). Zerar o traço todo frame é função
        // determinística do frame, não estado que gruda.
        if let Some(p) = scene.path_mut(spine_id) {
            p.stroke = None;
        }

        // Os passos de cada elo, INTERCALADOS com a fonte "de cima" dele, redesenhada por cima
        // deles. É a pilha de z do Illustrator: fonte0 (que o `dispatch` desenha, embaixo) → passos
        // → fonte1 → passos → fonte2 … Os passos saem de `cook_links` — a mesma porta que o
        // `expand` materializa, para o que se vê e o que se assa nunca divergirem.
        for (i, steps) in cook_links(&worlds, blend.steps as usize, &offsets)
            .into_iter()
            .enumerate()
        {
            out.extend(steps);
            out.push(worlds[i + 1].clone());
        }

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

/// **Modo Node: o SPINE aparece, elevado ao topo** (acima de TODAS as formas e passos) — é o path
/// que o artista edita, e tem de estar visível e clicável ali (ADR-0122). Na cena o spine é
/// INVISÍVEL (`recook` mantém o traço em `None`); aqui empurramos um clone TRAÇADO no fim de `out` —
/// o mesmo buffer que o [`recook`] encheu com os passos, desenhado por último
/// ([`ph2d_vec_render::draw_blend_overlay`]). Assim o spine só se vê no Node, por cima de tudo.
///
/// Roda DEPOIS de [`recook`] e ANTES do `dispatch`, e SÓ em modo Node. Em Select a linha não é
/// desenhada (é Node-only) — mantê-la visível a mostrava como um "fantasma" com drift ao mover as
/// formas (Enio 2026-07-15).
pub(crate) fn elevate_spines(
    sim: &SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    out: &mut Vec<VecPath>,
) {
    for (&id, &bits) in map.iter() {
        if sim
            .world()
            .get::<VecBlend>(Entity::from_bits(bits))
            .is_none()
        {
            continue;
        }
        let Some(p) = scene.path_mut(id) else {
            continue;
        };
        if p.verts.len() < 2 {
            continue; // spine vazio (blend sem 2 fontes vivas): não há linha a subir
        }
        let mut top = p.clone();
        top.stroke = Some(spine_stroke()); // o traço visível vai para o topo…
        p.stroke = None; // …e some da cena, para o `dispatch` não o desenhar embaixo (sem dobra)
        out.push(top);
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

/// Os testes do SPINE editável (ADR-0122 Fase C2) — arquivo irmão pelo teto de LOC; reusa os
/// helpers de `tests` (`pub(super)`).
#[cfg(test)]
#[path = "blend_live_spine_tests.rs"]
mod spine_tests;

/// Os testes do **Expand / Release** (ADR-0122 Fase D) — idem.
#[cfg(test)]
#[path = "blend_live_expand_tests.rs"]
mod expand_tests;
