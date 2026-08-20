//! Painel → shell. Cada arm sai do **retrato**, então uma linha que existe é uma linha que
//! despacha.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

use crate::state::{self, Model3dPanelState, ModelIntent};
use crate::{Model3dPanel, populate::MAX_ROWS};

/// De que **nó** é este id, se for de algum.
///
/// ⚠️ Uma varredura sobre a família, e não uma inversão do hash — um `NodeId` é um hash de nome e
/// hash não se inverte. O laço é sobre [`MAX_ROWS`] e corre uma vez por evento, não por quadro.
fn node_of(id: ph2d_a11y::NodeId) -> Option<(u32, bool)> {
    (0..MAX_ROWS as u32).find_map(|n| {
        if id == ids::model3d_radius_slider(n) {
            Some((n, false))
        } else if id == ids::model3d_radius_chip(n) {
            Some((n, true))
        } else {
            None
        }
    })
}

pub(crate) fn apply_event(
    _state: &mut Model3dPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        WidgetEvent::ValueChanged(id) => match node_of(id) {
            // ⚠️ O campo numérico já foi espelhado no slider ligado a ele, que disparou o seu
            // próprio `ValueChanged` e foi tratado no braço de baixo. Engolir aqui, ou uma edição
            // notifica duas vezes — e a segunda chega com o valor da primeira.
            Some((_, true)) => true,
            Some((node, false)) => {
                let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                let snap = state::current();
                match snap.rows.iter().find(|r| r.node == node) {
                    Some(row) => {
                        state::push_intent(ModelIntent::SetRadius {
                            node,
                            // A trilha é 0..1; o valor é ela vezes o teto **daquela linha**, que é
                            // o mesmo teto que o documento aplica.
                            radius: track * row.bound.value(),
                        });
                        true
                    }
                    // Um id da família sem linha no retrato: o documento encolheu entre o quadro
                    // pintado e o evento. Ignorar é a resposta certa — inventar um nó não é.
                    None => false,
                }
            }
            None => false,
        },
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
