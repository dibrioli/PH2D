//! Painel → shell. Cada arm sai do **retrato**, então uma linha que existe é uma linha que
//! despacha.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

use crate::state::{self, Model3dPanelState, ModelIntent};
use crate::{Model3dPanel, populate::MAX_ROWS};

/// De que **linha** é este id, se for de alguma. `(posição, é o campo numérico?)`.
///
/// ⚠️ Uma varredura sobre a família, e não uma inversão do hash — um `NodeId` é um hash de nome e
/// hash não se inverte. O laço é sobre [`MAX_ROWS`] e corre uma vez por evento, não por quadro.
fn slot_of(id: ph2d_a11y::NodeId) -> Option<(usize, bool)> {
    (0..MAX_ROWS as u32).find_map(|n| {
        if id == ids::model3d_radius_slider(n) {
            Some((n as usize, false))
        } else if id == ids::model3d_radius_chip(n) {
            Some((n as usize, true))
        } else {
            None
        }
    })
}

/// De que **posição** de um seletor é este id, se for de algum.
///
/// ⚠️ O seletor viaja como parâmetro: os dois (verbo e referencial) têm famílias de id próprias, e
/// partilhá-las faria um clique em «Local» disparar o verbo da mesma posição.
fn slot_in(id: ph2d_a11y::NodeId, of: fn(u32) -> ph2d_a11y::NodeId) -> Option<usize> {
    (0..crate::populate::MAX_MODES).find_map(|n| (id == of(n)).then_some(n as usize))
}

pub(crate) fn apply_event(
    _state: &mut Model3dPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        WidgetEvent::ValueChanged(id) => match slot_of(id) {
            // ⚠️ O campo numérico já foi espelhado no slider ligado a ele, que disparou o seu
            // próprio `ValueChanged` e foi tratado no braço de baixo. Engolir aqui, ou uma edição
            // notifica duas vezes — e a segunda chega com o valor da primeira.
            Some((_, true)) => true,
            Some((slot, false)) => {
                let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                let snap = state::current();
                match snap.rows.get(slot) {
                    Some(row) => {
                        state::push_intent(ModelIntent::SetRadius {
                            // ⭐ **A ENTIDADE, e não a posição.** A posição escolheu o controle; o
                            // que ela não pode escolher é o nó — a lista muda de tamanho quando a
                            // peça muda, e um intent guardado por posição escreveria no vizinho.
                            entity: row.entity,
                            // A trilha é 0..1; o valor é ela vezes o teto **daquela linha**, que é
                            // o mesmo teto que a peça aplica.
                            radius: track * row.bound.value(),
                        });
                        true
                    }
                    // Um id da família sem linha no retrato: a peça encolheu entre o quadro
                    // pintado e o evento. Ignorar é a resposta certa — inventar um nó não é.
                    None => false,
                }
            }
            None => false,
        },
        // ⭐ Os dois seletores. ⚠️ A POSIÇÃO é o que viaja: o painel não conhece os enums do
        // gizmo, e uma cópia deles aqui seria uma segunda contagem a envelhecer.
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_mode_button).is_some() => {
            let slot = slot_in(id, ids::model3d_mode_button).unwrap_or(0);
            // Um slot da família sem verbo no retrato: ignorar é a resposta certa.
            slot < state::current().modes.len() && {
                state::push_intent(ModelIntent::SetGizmoMode { slot });
                true
            }
        }
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_frame_button).is_some() => {
            let slot = slot_in(id, ids::model3d_frame_button).unwrap_or(0);
            slot < state::current().frames.len() && {
                state::push_intent(ModelIntent::SetGizmoFrame { slot });
                true
            }
        }
        WidgetEvent::Click(id) if id == ids::MODEL3D_CLOSE => {
            host.set_panel_visible(Model3dPanel::ID, false);
            true
        }
        _ => false,
    };
    if consumed {
        EventOutcome::Consumed
    } else {
        EventOutcome::Ignored
    }
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
