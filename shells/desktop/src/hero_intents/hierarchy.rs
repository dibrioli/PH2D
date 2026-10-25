//! Hierarchy drag-reparent drain (`EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Reparent)`).
//!
//! A lei (quem viaja, ciclo, receita, ordem, pose de mundo) mora em
//! [`ph2d_app_components::hierarchy_reparent`]; aqui só a ponte `NodeId → Entity`, a folha e a voz.

use ph2d_ecs::SimWorld;
use ph2d_i18n::tr;

/// Drena um pedido de reparent da Hierarquia. `selected` = os bits da selecção do gizmo: se a linha
/// agarrada está nela, viaja a selecção inteira (F30 §F, report do dono 25/10); senão, só ela.
///
/// Returns `false` — o reparent é mudo, salvo as duas vozes:
///
/// * ⚠️ largar dentro de uma RECEITA tira o objecto da cena (só desenha com a receita
///   seleccionada) — mudo, leria-se *«o meu objecto desapareceu»* (auditoria 27/08 §1.1);
/// * ⛔ a peça dada por uma receita tem o lugar da receita (F5.12) — a recusa diz onde fazer.
pub(crate) fn drain_reparent(
    intent: ph2d_editor_core::screens::hero::HierReparentIntent,
    selected: &[u64],
    live: &crate::HeroLive,
    sim: &mut SimWorld,
    toasts: &mut ph2d_editor_core::ToastQueue,
) -> bool {
    use ph2d_app_components::hierarchy_reparent::{Desfecho, Queda, quem_viaja, reparentar};
    // O PEDIDO (report 07/09: três gestos com a mesma cara no ecrã e três resultados).
    if crate::App::undo_log_on() {
        eprintln!(
            "[hier] pedido: dragged={:?} new_parent={:?} before={:?} after={:?} seleccao={}",
            intent.dragged,
            intent.new_parent,
            intent.before,
            intent.after,
            selected.len()
        );
    }
    let entity = |n: Option<ph2d_editor_core::NodeId>| {
        n.and_then(|n| live.bridge.entity_for(n))
            .map(ph2d_ecs::Entity::from_bits)
    };
    let Some(agarrado) = entity(Some(intent.dragged)) else {
        return false;
    };
    let seleccionados: Vec<ph2d_ecs::Entity> = selected
        .iter()
        .map(|&b| ph2d_ecs::Entity::from_bits(b))
        .collect();
    let quem = quem_viaja(sim.world_mut(), agarrado, &seleccionados);
    let queda = Queda {
        new_parent: entity(intent.new_parent),
        before: entity(intent.before),
        after: entity(intent.after),
    };
    match reparentar(sim, &quem, queda) {
        Desfecho::RecusaPecaDeReceita => {
            toasts.push(ph2d_editor_core::Toast::warning(tr(
                "shell.hierarchy.that_piece_s_place",
            )));
        }
        Desfecho::RecusaCiclo => {}
        Desfecho::Movidos {
            quem,
            into_a_recipe,
        } => {
            // LARGAR UM SPRITE DENTRO DE UMA FOLHA PÕE-NO DENTRO DELA (Enio 19/08) — anula de
            // propósito a pose de mundo, e só para este pai; depois de a pose local estar resolvida.
            for m in quem {
                ph2d_sheet_bounds::confine(sim, m);
            }
            if into_a_recipe {
                toasts.push(ph2d_editor_core::Toast::warning(tr(
                    "shell.hierarchy.moved_into_a_prefab_it",
                )));
            }
        }
    }
    false
}

#[cfg(test)]
#[path = "hierarchy_order_tests.rs"]
mod order_tests;
