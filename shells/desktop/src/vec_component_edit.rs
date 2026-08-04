//! **OS VERBOS de componente** — o que os quatro botões fazem (plano UI/UX W5).
//!
//! Irmão do [`crate::vec_anchor_edit`], com a mesma divisão de donos: a verdade mora no ECS
//! (`VecComponentMain` / `VecInstance`), isto é a shell a honrar o clique, e o painel só mostra.
//!
//! # As quatro operações, e a que cada uma NÃO faz
//!
//! - **Create** marca a seleção como mestre. Não move nada, não cria cópia nenhuma: o mestre
//!   continua a ser a arte que o artista desenhou, no sítio onde ela está.
//! - **Place** põe uma INSTÂNCIA ao lado. O que ela guarda é o **suporte** — um retângulo do
//!   tamanho do mestre — e o vínculo; o que se vê é derivado por frame.
//! - **Detach** materializa: o que estava na tela vira geometria da própria instância, e o
//!   componente sai. ⚠️ A geometria vem do **produtor**, nunca de uma segunda derivação — uma
//!   segunda porta faria a arte SALTAR no clique, que é o defeito que o ADR-0128 pagou cinco
//!   vezes e que o `bool_live` documenta na cabeça do próprio Apply.
//! - **Reset** limpa os overrides.
//!
//! # O suporte não é uma cópia
//!
//! Ele é quatro pontos. É o que dá à instância uma caixa de gizmo (`vec_gizmo_view::anchor_half`
//! lê a bbox GUARDADA), um alvo de clique quando o vínculo quebra, e um lugar na ordem de z.
//! Guardar a árvore do mestre ali seria a herança por CÓPIA que o plano recusa — `O(N × tamanho)`
//! em memória, e sem propagação.

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
}

/// Este id é um verbo de componente? Porta única do roteador.
#[must_use]
pub(crate) fn component_edit_for_id(id: ph2d_editor::NodeId) -> Option<ComponentEdit> {
    match id {
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_CREATE => Some(ComponentEdit::Create),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_PLACE => Some(ComponentEdit::Place),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_DETACH => Some(ComponentEdit::Detach),
        _ if id == ph2d_editor::ids::VECTOR_COMPONENT_RESET => Some(ComponentEdit::Reset),
        _ => None,
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
) -> Option<ph2d_panel_vector::state::ComponentState> {
    let (id, e) = subject(sim, map, selected)?;
    let inst = sim.world().get::<VecInstance>(e);
    Some(ph2d_panel_vector::state::ComponentState {
        is_main: sim.world().get::<VecComponentMain>(e).is_some(),
        is_instance: inst.is_some(),
        has_overrides: inst.is_some_and(|i| !i.overrides.is_empty()),
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
    let (lo, hi) = scene.path_curve_bbox(main_id)?;
    let id = scene.push_path(rectangle(lo, hi));
    // ⚠️ A entidade nasce no `sync`, que corre DEPOIS da malha de ações neste mesmo frame — não
    // aqui. Quem pendura o vínculo é o [`arm_instance`], logo a seguir a ele; é o protocolo do
    // `make_committed_shape_live`, e a razão é a mesma: o mapa path↔entidade é construído lá.
    Some(id)
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

/// **Detach**: o que está na tela vira geometria da instância, e o vínculo sai.
///
/// `drawn` é o que o PRODUTOR desenhou neste frame — a mesma lista que o `dispatch` consumiu.
/// Passá-la em vez de a re-derivar é o que faz a arte não saltar no clique.
///
/// ⚠️ **Materializa TODAS as peças, não a primeira.** A raiz do mestre pousa no caminho da própria
/// instância (que assim mantém id, z e seleção) e cada peça restante vira um caminho NOVO,
/// empurrado logo a seguir — a ordem de z do documento é a ordem em que o produtor as desenhou, e
/// é por isso que empurrar em sequência a preserva. Escrever só a primeira teria dado um Detach
/// que **perde arte em silêncio** num componente de duas peças, que é o caso comum (uma caixa e um
/// rótulo).
///
/// Devolve os ids dos caminhos criados — o chamador os parenteia depois do `sync` ([`arm_detached`]).
pub(crate) fn detach(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
    drawn: Option<&[ph2d_vec_scene::VecPath]>,
) -> Option<(VecPathId, Vec<VecPathId>)> {
    let (id, e) = subject(sim, map, selected)?;
    sim.world().get::<VecInstance>(e)?;
    let mut extra = Vec::new();
    if let Some(items) = drawn {
        // A geometria derivada está em MUNDO; um caminho guarda LOCAL. Assumir o mundo (e deixar
        // a pose onde está) é o que o **Apply** da booleana faz com o resultado dela.
        if let Some(first) = items.first()
            && let Some(p) = scene.path_mut(id)
        {
            let keep = p.id;
            *p = first.clone();
            p.id = keep;
        }
        for piece in items.iter().skip(1) {
            extra.push(scene.push_path(piece.clone()));
        }
    }
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
        em.remove::<VecInstance>();
    }
    Some((id, extra))
}

/// Pendura as peças materializadas pelo [`detach`] sob o caminho que ficou (pós-`sync`).
///
/// ⚠️ Parentear, e não deixá-las soltas na raiz: o que o artista tinha era **uma** coisa que se
/// move junta, e um Detach que a espalha em N objetos independentes desfaz o agrupamento que ele
/// nunca pediu para desfazer.
pub(crate) fn arm_detached(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    root: VecPathId,
    pieces: &[VecPathId],
) -> bool {
    let Some(&root_bits) = map.get(&root) else {
        return false;
    };
    let root_e = Entity::from_bits(root_bits);
    let mut any = false;
    for p in pieces {
        let Some(&bits) = map.get(p) else { continue };
        let e = Entity::from_bits(bits);
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(ph2d_ecs::ChildOf(root_e));
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
