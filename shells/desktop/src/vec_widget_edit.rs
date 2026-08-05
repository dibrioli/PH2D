//! **Os verbos da PELE por-widget** (plano UI/UX W6.2) — irmão do [`crate::vec_component_edit`],
//! mesma divisão: o painel PEDE, a shell FAZ, e o que muda é o ECS.
//!
//! ⚠️ **Três verbos e só eles**, porque o modelo tem um campo só: vestir, trocar de tipo,
//! despir. Não há um quarto porque não há um segundo número — o rótulo é o `Name` e a aparência
//! é dos tokens.

use ph2d_ecs::{Entity, SimWorld, VecWidget};
use ph2d_editor::widget::WidgetKind;
use ph2d_vec_scene::VecPathId;

use crate::vec_entities::VecEntityMap;

/// O que um clique na seção WIDGET SKIN pede.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WidgetEdit {
    /// **Wear a Widget** — a forma passa a vestir (nasce Button, o tipo mais comum).
    Wear,
    /// **Back to Drawing** — a forma volta a ser vetor.
    Remove,
    /// O chip do tipo `i` em `WidgetKind::ALL`.
    Kind(usize),
    /// **Bind Shape** — ARMA o conta-gotas; quem resolve é o clique seguinte (W8b.3).
    ///
    /// ⚠️ Ele não carrega alvo: a fonte é a forma selecionada AGORA e o alvo é o que o clique
    /// acertar. É o mesmo gesto de duas mãos do `PathPick`, e por isso reusa aquele modal em vez
    /// de um segundo — o segundo é o que esquece o Escape.
    Bind,
    /// **Unbind** — solta a forma dirigida.
    Unbind,
}

/// Este id é um verbo de pele? **Porta única** do roteador.
///
/// ⚠️ A varredura dos chips é sobre `MAX_WIDGET_KINDS` — o mesmo intervalo que o `populate`
/// regista. As duas pontas leem a MESMA constante: um teto que o roteador conhecesse e o registro
/// não deixaria os últimos chips mortos sob o rato.
#[must_use]
pub(crate) fn widget_edit_for_id(id: ph2d_editor::NodeId) -> Option<WidgetEdit> {
    match id {
        _ if id == ph2d_editor::ids::VECTOR_WIDGET_WEAR => Some(WidgetEdit::Wear),
        _ if id == ph2d_editor::ids::VECTOR_WIDGET_REMOVE => Some(WidgetEdit::Remove),
        _ if id == ph2d_editor::ids::VECTOR_WIDGET_BIND => Some(WidgetEdit::Bind),
        _ if id == ph2d_editor::ids::VECTOR_WIDGET_UNBIND => Some(WidgetEdit::Unbind),
        _ => (0..ph2d_editor::ids::MAX_WIDGET_KINDS)
            .find(|&i| ph2d_editor::ids::vector_widget_kind_id(i) == id)
            .map(WidgetEdit::Kind),
    }
}

/// A forma selecionada (uma só) e a entidade dela.
fn subject(sim: &SimWorld, map: &VecEntityMap, selected: &[VecPathId]) -> Option<Entity> {
    let [only] = selected else { return None };
    let &bits = map.get(only)?;
    let e = Entity::from_bits(bits);
    sim.world().get_entity(e).ok().map(|_| e)
}

/// Aplica o verbo. Sem seleção única, não faz nada — a seção nem é oferecida nesse caso.
pub(crate) fn apply(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
    verb: WidgetEdit,
) {
    let Some(e) = subject(sim, map, selected) else {
        return;
    };
    match verb {
        WidgetEdit::Wear => {
            sim.world_mut().entity_mut(e).insert(VecWidget {
                kind: WidgetKind::Button.code(),
            });
        }
        WidgetEdit::Remove => {
            // ⚠️ Despir leva o VÍNCULO junto: um `VecWidgetBind` sobre uma forma que já não veste
            // é um fio pendurado em nada — ele viajaria no save, entraria no diff do undo e
            // ressuscitaria dirigindo no dia em que alguém a vestisse outra vez.
            sim.world_mut()
                .entity_mut(e)
                .remove::<VecWidget>()
                .remove::<ph2d_ecs::VecWidgetBind>();
        }
        // O ARM mora no `render_loop` (é ele que tem o modal na mão); aqui não há o que escrever.
        WidgetEdit::Bind => {}
        WidgetEdit::Unbind => {
            sim.world_mut()
                .entity_mut(e)
                .remove::<ph2d_ecs::VecWidgetBind>();
        }
        WidgetEdit::Kind(i) => {
            // ⚠️ Um índice fora do catálogo é um chip que o painel não pintou — ignorar é a
            // resposta certa, e escrever um `kind` inventado faria a forma desaparecer.
            if let Some(&kind) = WidgetKind::ALL.get(i) {
                sim.world_mut()
                    .entity_mut(e)
                    .insert(VecWidget { kind: kind.code() });
            }
        }
    }
}

/// **Prende a row daquele widget a esta forma** (W8b.3). `false` = não deu, e o pick fica armado.
///
/// ⚠️ Recusa um alvo que **veste** — dirigir um controle com outro controle é uma cadeia que este
/// modelo não tem (quem lê o valor é a projeção da CENA, não outra row), e aceitá-la em silêncio
/// daria um vínculo que não faz nada. Recusar mantém o conta-gotas armado, que é o que diz ao
/// artista *"este não"* sem nenhuma mensagem.
pub(crate) fn bind(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    widget: VecPathId,
    target: VecPathId,
) -> bool {
    let Some(&bits) = map.get(&widget) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    if sim.world().get_entity(e).is_err() {
        return false;
    }
    // Só um widget que DIRIGE aceita alvo — o painel já não oferece o botão nos outros, e esta é
    // a metade que HONRA a mesma pergunta.
    let drives = sim
        .world()
        .get::<VecWidget>(e)
        .and_then(|w| WidgetKind::from_code(w.kind))
        .is_some_and(crate::vec_widget_drive::bindable);
    if !drives {
        return false;
    }
    let target_wears = map
        .get(&target)
        .map(|&b| Entity::from_bits(b))
        .is_some_and(|t| sim.world().get::<VecWidget>(t).is_some());
    if target_wears {
        return false;
    }
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_ecs::VecWidgetBind { target });
    true
}

/// O que o painel mostra para a seleção — `None` = não oferecer a seção.
///
/// ⚠️ Ela é oferecida para **qualquer forma única**, vestida ou não: uma seção que só existisse
/// onde já há pele tornaria a feature alcançável apenas onde ela já foi usada, ou seja em lugar
/// nenhum. É a mesma lei da seção de física, cuja face VAZIA é a importante.
#[must_use]
pub(crate) fn publish(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<(ph2d_panel_vector::state::WidgetSkinState, usize)> {
    let e = subject(sim, map, selected)?;
    let worn = sim.world().get::<VecWidget>(e).copied();
    let known = worn.and_then(|w| WidgetKind::from_code(w.kind));
    let cap = ph2d_editor::ids::MAX_WIDGET_KINDS;
    let kinds: Vec<String> = WidgetKind::ALL
        .iter()
        .take(cap)
        .map(|k| ph2d_i18n::tr(k.i18n_key()).to_string())
        .collect();
    // ⚠️ O nome do alvo sai do `Name` da ENTIDADE dele, e um alvo que já não existe mostra
    // `(none)` em vez de um id cru: o vínculo aponta um `VecPathId`, e uma forma apagada deixa o
    // fio a apontar para o vazio — dizer isso em palavras é o que evita o artista procurar uma
    // forma que ele mesmo apagou.
    let drives = known
        .filter(|&k| crate::vec_widget_drive::bindable(k))
        .map(|_| {
            sim.world()
                .get::<ph2d_ecs::VecWidgetBind>(e)
                .and_then(|b| map.get(&(b.target)).copied())
                .map(Entity::from_bits)
                .filter(|&t| sim.world().get_entity(t).is_ok())
                .and_then(|t| sim.world().get::<ph2d_ecs::Name>(t))
                .map(|n| n.0.to_string())
        });
    let state = ph2d_panel_vector::state::WidgetSkinState {
        drives,
        selected: known.and_then(|k| WidgetKind::ALL.iter().position(|&x| x == k)),
        // ⚠️ Vestida COM tipo desconhecido é um terceiro estado, e não *"não vestida"*: a forma
        // desenha como vetor mas o documento carrega uma pele, e o botão que faz sentido é
        // *despir*, não *vestir* (vestir apagaria o `kind` do futuro em silêncio).
        unknown: worn.is_some() && known.is_none(),
        kinds,
    };
    Some((state, WidgetKind::ALL.len().saturating_sub(cap)))
}

#[cfg(test)]
#[path = "vec_widget_edit_tests.rs"]
mod tests;
