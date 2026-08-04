//! **OS VERBOS de componente** — o que os quatro botões fazem (plano UI/UX W5).
//!
//! Irmão do [`crate::vec_anchor_edit`], com a mesma divisão de donos: a verdade mora no ECS
//! (`VecComponentMain` / `VecInstance`), isto é a shell a honrar o clique, e o painel só mostra.
//!
//! # As quatro operações, e a que cada uma NÃO faz
//!
//! - **Create** marca a seleção como mestre. Não move nada, não cria cópia nenhuma: o mestre
//!   continua a ser a arte que o artista desenhou, no sítio onde ela está.
//! - **Place** põe uma INSTÂNCIA ao lado. O que ela guarda é o **suporte** — um retângulo da caixa
//!   do CONTEÚDO do mestre ([`main_content_box`]) — e o vínculo; o que se vê é derivado por frame.
//! - **Detach** materializa: o que estava na tela vira geometria da própria instância, e o
//!   componente sai. ⚠️ A geometria vem do **produtor**, nunca de uma segunda derivação — uma
//!   segunda porta faria a arte SALTAR no clique, que é o defeito que o ADR-0128 pagou cinco
//!   vezes. ⚠️ E a RAIZ do resultado é a raiz do mestre por IDENTIDADE, nunca a primeira peça
//!   desenhada: ver [`detach`], onde os dois defeitos do smoke estão medidos.
//! - **Reset** limpa os overrides.
//!
//! # O suporte não é uma cópia
//!
//! Ele é quatro pontos — a caixa do que o mestre DESENHA, sub-árvore e pose incluídas. É o que dá
//! à instância uma caixa de gizmo (`vec_gizmo_view::anchor_half` lê a bbox GUARDADA), um alvo de
//! clique quando o vínculo quebra, e um lugar na ordem de z. Guardar a árvore do mestre ali seria a
//! herança por CÓPIA que o plano recusa — `O(N × tamanho)` em memória, e sem propagação.

use ph2d_ecs::{Entity, SimWorld, VecComponentMain, VecInstance};
use ph2d_vec_scene::{VecPathId, VecScene, rectangle};

use crate::vec_entities::VecEntityMap;

/// O que um clique num verbo de componente PEDE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ComponentEdit {
    Create,
    Place,
    Detach,
    Reset,
    /// **Update Main** (W5b) — as diferenças desta instância passam a ser o mestre.
    UpdateMain,
    /// **Swap Main** (W5b) — arma o conta-gotas; o clique seguinte no canvas escolhe o mestre.
    Swap,
    /// O interruptor de visibilidade da PEÇA `row` desta instância (W5b).
    PieceVisible(usize),
}

/// Este id é um verbo de componente? Porta única do roteador.
///
/// ⚠️ A varredura das linhas de peça é sobre [`ph2d_editor::ids::MAX_INSTANCE_PIECES`] — o mesmo
/// intervalo que o `populate` regista. As duas pontas leem a MESMA constante: um teto que o
/// roteador conhecesse e o registro não deixaria as últimas linhas mortas sob o rato.
#[must_use]
pub(crate) fn component_edit_for_id(id: ph2d_editor::NodeId) -> Option<ComponentEdit> {
    match id {
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_CREATE => Some(ComponentEdit::Create),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_PLACE => Some(ComponentEdit::Place),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_DETACH => Some(ComponentEdit::Detach),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_RESET => Some(ComponentEdit::Reset),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_UPDATE_MAIN => {
            Some(ComponentEdit::UpdateMain)
        }
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_SWAP => Some(ComponentEdit::Swap),
        _ => (0..ph2d_editor::ids::MAX_INSTANCE_PIECES)
            .find(|&r| ph2d_editor::ids::vector_instance_piece_show_id(r) == id)
            .map(ComponentEdit::PieceVisible),
    }
}

/// A forma selecionada (uma só) e a entidade dela.
fn subject(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<(VecPathId, Entity)> {
    let [only] = selected else { return None };
    let &bits = map.get(only)?;
    let e = Entity::from_bits(bits);
    sim.world().get_entity(e).ok().map(|_| (*only, e))
}

/// **O estado que o painel lê** — que verbos fazem sentido para esta seleção.
///
/// `None` = não oferecer a seção (nenhuma forma selecionada, ou mais de uma: um prefab é sobre UMA
/// coisa, e "criar componente" a partir de duas seleções é outra operação — agrupar primeiro).
#[must_use]
pub(crate) fn selected_component(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
    orphans: &[VecPathId],
    swap_armed: bool,
) -> Option<ph2d_panel_vector::state::ComponentState> {
    let (id, e) = subject(sim, map, selected)?;
    let inst = sim.world().get::<VecInstance>(e);
    Some(ph2d_panel_vector::state::ComponentState {
        is_main: sim.world().get::<VecComponentMain>(e).is_some(),
        is_instance: inst.is_some(),
        has_overrides: inst.is_some_and(|i| !i.overrides.is_empty()),
        swap_armed,
        // ⚠️ A órfã é decidida pelo PRODUTOR, e o painel lê a resposta dele. Re-perguntar aqui
        // (*"o mestre existe?"*) seria a segunda resposta, e ela divergiria no frame em que o
        // produtor recusasse por outra razão — pose degenerada, laço — e o painel dissesse que
        // está tudo bem.
        main_missing: orphans.contains(&id),
    })
}

/// **Cria um mestre** a partir da seleção. `true` se o mundo mudou.
pub(crate) fn create_main(sim: &mut SimWorld, map: &VecEntityMap, selected: &[VecPathId]) -> bool {
    let Some((_, e)) = subject(sim, map, selected) else {
        return false;
    };
    // Uma instância não vira mestre: o que ela mostra é derivado, então um componente feito dela
    // seria um componente de um desenho que não é dela. Faça Detach primeiro.
    if sim.world().get::<VecInstance>(e).is_some()
        || sim.world().get::<VecComponentMain>(e).is_some()
    {
        return false;
    }
    sim.world_mut().entity_mut(e).insert(VecComponentMain);
    true
}

/// **Quantas instâncias VIVAS deste mestre já existem** — o degrau da cascata.
///
/// A varredura é sobre a CENA e não sobre o mundo ECS, pelo mesmo motivo do produtor: é a cena que
/// dá a ordem estável, e é ela que o `sync` mantém de acordo com as entidades.
#[must_use]
pub(crate) fn instance_count(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    main: VecPathId,
) -> usize {
    scene
        .paths()
        .iter()
        .filter_map(|p| map.get(&p.id))
        .filter(|&&bits| {
            sim.world()
                .get::<VecInstance>(Entity::from_bits(bits))
                .is_some_and(|i| i.main == main)
        })
        .count()
}

/// **Onde a cópia nasce**: `already + 1` degraus de tela a partir do mestre.
///
/// # Duas coisas estavam erradas aqui, e a segunda escondia-se atrás da primeira
///
/// O `step` chega em MUNDO, mas nasce de um número em **pixels de TELA**
/// ([`crate::input_dispatch::PASTE_OFFSET_PX`], convertido por
/// [`crate::input_dispatch::screen_offset_world`]). A v1 tinha um `PLACE_OFFSET: f32 = 24.0` em
/// unidades de MUNDO com um doc-comment a afirmar que era *"a mesma folga do Duplicate"* — e não
/// era: a folga do Duplicate são 12 px de tela. Medido na cena `=53`, a ~29 px por unidade, a
/// cópia nascia a **~700 px do mestre — 58× um paste, e sete larguras do próprio botão**. É a
/// classe das unidades misturadas: o número parecia pixels e foi consumido como mundo.
///
/// E a cascata: a v1 era `const`, então os três cliques do roteiro punham as três cópias **no
/// mesmo sítio**. O Ctrl+D não sofre disto porque **seleciona a cópia** (a duplicação seguinte
/// parte dela); o *Place* mantém o MESTRE selecionado — senão o botão deixaria de ser *Place* no
/// clique seguinte —, então o degrau tem de vir de outro lado: de quantas cópias já existem.
///
/// ⚠️ **Sem teto, de propósito.** Um teto faria a cópia N+1 voltar a pousar em cima de uma
/// anterior, que é exactamente o defeito que esta função existe para remover; e o que limita a
/// cascata é o gesto — uma cópia por clique. Uma cópia arrastada para longe deixa o degrau dela
/// vago (a seguinte nasce um degrau adiante do que precisava): fica **um vão adjacente ao mestre**,
/// sempre visível, e é o preço de contar em vez de procurar sítio livre — procurar exigiria um
/// raio de *"este sítio está ocupado?"*, um número que nada mede.
#[must_use]
pub(crate) fn cascade_offset(step: [f32; 2], already: usize) -> [f32; 2] {
    let n = already.saturating_add(1) as f32;
    [step[0] * n, step[1] * n]
}

/// **Põe uma instância** do mestre selecionado. Devolve o caminho novo.
pub(crate) fn place_instance(
    sim: &SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<VecPathId> {
    let (main_id, main_e) = subject(sim, map, selected)?;
    sim.world().get::<VecComponentMain>(main_e)?;
    // O SUPORTE: um retângulo da caixa do mestre. Ele não é a arte — é o que dá à instância uma
    // caixa de gizmo e um alvo de clique (ver o § do cabeçalho).
    let (lo, hi) = main_content_box(sim, scene, map, main_id, main_e)?;
    let id = scene.push_path(rectangle(lo, hi));
    // ⚠️ A entidade nasce no `sync`, que corre DEPOIS da malha de ações neste mesmo frame — não
    // aqui. Quem pendura o vínculo é o [`arm_instance`], logo a seguir a ele; é o protocolo do
    // `make_committed_shape_live`, e a razão é a mesma: o mapa path↔entidade é construído lá.
    Some(id)
}

/// **A caixa do CONTEÚDO do mestre**, no referencial em que a instância o vai desenhar.
///
/// ⚠️ Duas coisas que a v1 ignorava, e as duas apareciam como um gizmo do tamanho errado:
/// a caixa era a do caminho-RAIZ (um botão cujo rótulo transborda ficava com o suporte a menos) e
/// era **local**, cega à pose do mestre — e desde que a forma do mestre passou a propagar
/// ([`crate::instance_live::place_delta`]) um mestre escalado 2× daria uma cópia do dobro do
/// tamanho dentro de uma caixa de gizmo do tamanho antigo.
///
/// A caixa é medida no MESMO referencial do desenho: cada peça em mundo, menos a translação do
/// mestre. ⚠️ Sob rotação ela é uma **sobre-aproximação** (as quinas do bbox local giram e o
/// resultado é re-alinhado aos eixos) — a mesma limitação honesta que a caixa do gizmo já carrega.
///
/// ⚠️ **E ela é um instantâneo:** redimensionar o mestre DEPOIS move o desenho da cópia e **não**
/// move o suporte dela. Corrigir isso é re-derivar geometria do documento a cada frame, que é a
/// classe que o ADR-0128 recusou cinco vezes; fica NOMEADO em vez de contrabandeado.
fn main_content_box(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    main_id: VecPathId,
    main_e: Entity,
) -> Option<([f64; 2], [f64; 2])> {
    let m = crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(
        sim, main_e,
    ))
    .0;
    let un_place = ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, -m[4], -m[5]]);
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    // A MESMA porta que decide o que a instância desenha — uma segunda travessia daria um suporte
    // que descreve um conjunto de peças diferente do que aparece.
    let probe = VecInstance::new(main_id);
    for piece in crate::instance_live::visible_pieces(scene, sim, map, main_e, &probe) {
        let Some((plo, phi)) = scene.path_curve_bbox(piece) else {
            continue;
        };
        let Some(&bits) = map.get(&piece) else {
            continue;
        };
        let w = crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(
            sim,
            Entity::from_bits(bits),
        ));
        for corner in [
            [plo[0], plo[1]],
            [phi[0], plo[1]],
            [plo[0], phi[1]],
            [phi[0], phi[1]],
        ] {
            let p = un_place.apply(w.apply(corner));
            for a in 0..2 {
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
        }
    }
    (lo[0] <= hi[0]).then_some((lo, hi))
}

/// Pendura o vínculo numa instância recém-criada (chamado depois do `sync`).
pub(crate) fn arm_instance(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    at: VecPathId,
    main: VecPathId,
    place_offset: [f32; 2],
) -> bool {
    let Some(&bits) = map.get(&at) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
        return false;
    };
    let mut t = em.get::<ph2d_ecs::Transform>().copied().unwrap_or_default();
    t.translation.x += place_offset[0];
    t.translation.y += place_offset[1];
    em.insert((t, VecInstance::new(main)));
    true
}

/// O que um [`detach`] produziu: o caminho que ficou, os que nasceram, e quem é pai de quem.
pub(crate) struct Detached {
    /// Os caminhos empurrados, na ordem de z em que o produtor os desenhou.
    pub(crate) extra: Vec<VecPathId>,
    /// `(filho, pai)` em ids de CAMINHO — o espelho da árvore do mestre.
    pub(crate) parents: Vec<(VecPathId, VecPathId)>,
}

/// **Detach**: o que está na tela vira geometria da instância, e o vínculo sai.
///
/// `drawn` é o que o PRODUTOR desenhou neste frame e `pieces` são os ids das peças do mestre que
/// o produziram, **na mesma ordem** ([`crate::instance_live::visible_pieces`], a porta única).
/// Passá-los em vez de re-derivar é o que faz a arte não saltar no clique.
///
/// # Dois defeitos que o smoke expôs, e os dois eram sobre a mesma preguiça
///
/// ⚠️ **(1) A raiz era a PRIMEIRA peça desenhada, não a raiz do mestre.** A lista vem na ordem de
/// **z do documento**, e z é autorável: basta o filho estar à frente do pai para o Detach trocar os
/// dois — *"quem era pai virou filho, quem era filho virou pai"*. A raiz agora é escolhida por
/// IDENTIDADE (`inst.main`), que não depende de ordem nenhuma.
///
/// ⚠️ **(2) A arte SALTAVA.** A geometria derivada está em MUNDO e um caminho guarda LOCAL; a v1
/// gravava mundo num caminho cuja entidade ainda carrega a pose da instância, então a pose era
/// aplicada **duas vezes** — medido, uma peça desenhada em `x = 100` aterrava em `x = 200`. O
/// doc-comment que justificava isso citava o *Apply* da booleana — e a citação falhava por uma
/// PREMISSA: lá o resultado nasce num caminho NOVO cuja entidade *"nasce na identidade"*
/// (`input_dispatch::apply_vec_boolean`), então mundo **é** local; aqui a geometria pousa num
/// caminho que já existe e cuja entidade carrega a pose da instância. Cada peça é agora escrita no
/// referencial de quem a vai guardar (`I⁻¹`) — e como todo caminho novo nasce com `Transform`
/// identidade sob a raiz, **um** inverso serve a árvore inteira.
///
/// ⚠️ **E o parentesco é o do MESTRE**, não *"tudo sob a raiz"*: um componente de três níveis
/// achatava para dois, silenciosamente.
pub(crate) fn detach(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
    drawn: Option<&[ph2d_vec_scene::VecPath]>,
    pieces: Option<&[VecPathId]>,
) -> Option<Detached> {
    let (id, e) = subject(sim, map, selected)?;
    let main_id = sim.world().get::<VecInstance>(e)?.main;
    let mut out = Detached {
        extra: Vec::new(),
        parents: Vec::new(),
    };
    if let (Some(items), Some(src)) = (drawn, pieces)
        && items.len() == src.len()
        && !items.is_empty()
    {
        // Mundo → local do caminho que a vai guardar. Degenerado (escala 0) recusa: gravar
        // geometria que não tem volta seria destruir a instância para não mostrar nada.
        let inv =
            crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(sim, e))
                .inverse()?;
        // A RAIZ por identidade; se ela estiver escondida nesta instância (hoje inalcançável — o
        // override não tem gesto), a primeira desenhada assume, para o resultado nunca ficar sem
        // um caminho que segure o grupo.
        let root_i = src.iter().position(|p| *p == main_id).unwrap_or(0);
        // `peça do mestre -> caminho que a recebeu`, para o parentesco sair da árvore do MESTRE.
        let mut landed: Vec<(VecPathId, VecPathId)> = Vec::new();
        if let Some(p) = scene.path_mut(id) {
            let keep = p.id;
            *p = items[root_i].clone();
            p.id = keep;
            ph2d_vec_scene::bake_xform(p, &inv);
        }
        landed.push((src[root_i], id));
        for (i, piece) in items.iter().enumerate() {
            if i == root_i {
                continue;
            }
            let mut g = piece.clone();
            ph2d_vec_scene::bake_xform(&mut g, &inv);
            let new_id = scene.push_path(g);
            out.extra.push(new_id);
            landed.push((src[i], new_id));
        }
        for (i, master_piece) in src.iter().enumerate() {
            if i == root_i {
                continue;
            }
            let child = landed.iter().find(|(m, _)| m == master_piece)?.1;
            let parent = master_parent_path(sim, map, *master_piece, &landed).unwrap_or(id);
            out.parents.push((child, parent));
        }
    }
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
        em.remove::<VecInstance>();
    }
    Some(out)
}

/// Sobe a árvore ECS do mestre a partir de `piece` até achar uma peça que TENHA aterrado, e devolve
/// o caminho dela. `None` quando nenhuma aterrou (a peça pendura na raiz).
fn master_parent_path(
    sim: &SimWorld,
    map: &VecEntityMap,
    piece: VecPathId,
    landed: &[(VecPathId, VecPathId)],
) -> Option<VecPathId> {
    let &bits = map.get(&piece)?;
    let mut cur = Entity::from_bits(bits);
    for _ in 0..MAX_TREE_DEPTH {
        let parent = sim.world().get::<ph2d_ecs::ChildOf>(cur)?.parent();
        let pid = map
            .iter()
            .find(|(_, b)| **b == parent.to_bits())
            .map(|(k, _)| *k);
        if let Some(pid) = pid
            && let Some((_, landed_at)) = landed.iter().find(|(m, _)| *m == pid)
        {
            return Some(*landed_at);
        }
        cur = parent;
    }
    None
}

/// Profundidade máxima de subida na árvore do mestre — o mesmo guard do produtor.
const MAX_TREE_DEPTH: usize = 64;

/// Pendura as peças materializadas pelo [`detach`] segundo a árvore do mestre (pós-`sync`).
///
/// ⚠️ Parentear, e não deixá-las soltas na raiz: o que o artista tinha era **uma** coisa que se
/// move junta, e um Detach que a espalha em N objetos independentes desfaz o agrupamento que ele
/// nunca pediu para desfazer.
pub(crate) fn arm_detached(sim: &mut SimWorld, map: &VecEntityMap, plan: &Detached) -> bool {
    let mut any = false;
    for (child, parent) in &plan.parents {
        let (Some(&cb), Some(&pb)) = (map.get(child), map.get(parent)) else {
            continue;
        };
        if let Ok(mut em) = sim.world_mut().get_entity_mut(Entity::from_bits(cb)) {
            em.insert(ph2d_ecs::ChildOf(Entity::from_bits(pb)));
            any = true;
        }
    }
    any
}

/// **Reset**: a instância volta a ser exactamente o mestre.
pub(crate) fn reset_overrides(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> bool {
    let Some((_, e)) = subject(sim, map, selected) else {
        return false;
    };
    let Some(mut inst) = sim.world().get::<VecInstance>(e).cloned() else {
        return false;
    };
    if inst.overrides.is_empty() {
        return false; // no-op: o `post_frame_undo` regista por diff, e um passo vazio é ruído
    }
    inst.reset();
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
        em.insert(inst);
        return true;
    }
    false
}

#[cfg(test)]
#[path = "vec_component_edit_tests.rs"]
mod tests;
