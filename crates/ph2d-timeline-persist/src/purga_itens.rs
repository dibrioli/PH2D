//! ⭐ **O binding de um item que o ficheiro trouxe e que já não existe sai** (F32 §1).
//!
//! Na sessão, o item apagado fica `item_missing` — escondido e guardado, porque o undo global o pode
//! devolver (decisão do dono 25/10). Depois de gravar e reabrir, nenhum undo o devolve: sem esta
//! purga ele viajava escondido no ficheiro para sempre. Só o load liga
//! [`TimelineState::itens_por_decidir`]; cada binding decide-se quando a família publicou a entidade
//! dele, e a bandeira desliga quando nenhum fica por decidir — a lei da sessão fica intacta.

use ph2d_ecs::World;
use ph2d_ecs::canal_item_vivo::{decidido, nomeia_item};
use ph2d_timeline::{PropKind, TimelineState};

/// Corre no `upkeep`, depois da cura e da purga dos objectos. Sem a bandeira não faz nada.
pub(crate) fn decide(timeline: &mut TimelineState, world: &World) {
    if !timeline.itens_por_decidir {
        return;
    }
    let mut por_decidir = false;
    let mortos: Vec<ph2d_anim::AnimTarget> = (timeline.doc.bindings().iter())
        .filter_map(|b| {
            let PropKind::Field(k) = b.prop else {
                return None;
            };
            if !nomeia_item(k) {
                return None;
            }
            // Objecto por recolar (empate do nome): espera.
            if b.missing && !b.item_missing {
                por_decidir = true;
                return None;
            }
            match decidido(world, b.entity, k) {
                None => {
                    por_decidir = true;
                    None
                }
                Some(vivo) => (!vivo).then_some(b.target),
            }
        })
        .collect();
    if !mortos.is_empty() {
        for t in mortos {
            timeline.doc.purge_binding(t);
        }
        timeline.selection.clear();
    }
    timeline.itens_por_decidir = por_decidir;
}

#[cfg(test)]
#[path = "purga_itens_tests.rs"]
mod tests;
