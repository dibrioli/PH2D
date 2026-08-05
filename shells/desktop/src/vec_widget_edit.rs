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
            sim.world_mut().entity_mut(e).remove::<VecWidget>();
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
    let state = ph2d_panel_vector::state::WidgetSkinState {
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
