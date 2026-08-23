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
                    // ⚠️ **Uma linha inerte não despacha**, mesmo que um evento chegue: ela não
                    // regista nada no índice de acerto, mas o widget continua vivo no *store* (o
                    // `populate` cunha a família inteira às cegas), e um arrasto que atravessasse a
                    // trava a meio ainda podia disparar. Emitir aqui daria uma edição que a escrita
                    // recusa — o número a saltar e a voltar, que é o defeito na sua forma mais
                    // confusa. Ver `ParamRow::live`.
                    Some(row) if !row.live => false,
                    Some(row) => {
                        state::push_intent(ModelIntent::SetParam {
                            // ⭐ **A ENTIDADE e o ÍNDICE, nunca a posição do controle.** A posição
                            // escolheu o widget; o que ela não pode escolher é o nó nem a dimensão
                            // — a lista muda quando a seleção muda, e um intent guardado por
                            // posição escreveria noutro número.
                            entity: row.entity,
                            param: row.param,
                            // ⚠️ **A trilha é 0..1 e a faixa tem DUAS pontas** — esta conta era
                            // `track * bound`, com o piso implícito em zero. Ela concordava com a
                            // pintura só enquanto toda linha começava em zero: numa **posição**
                            // (piso negativo) a ponta esquerda do slider emitiria `0` em vez do
                            // mínimo, e o objeto saltaria para a origem a meio do arrasto.
                            //
                            // ⭐ É a **mesma** aritmética que o `paint` instala em
                            // `link_slider_number_mapped(slider, chip, hi - lo, lo)`, e o gate
                            // `the_dispatched_value_is_the_one_the_painted_mapping_promises` prende
                            // as duas portas uma à outra — porque um par destes só falha quando
                            // discordam, e cada lado sozinho parece certo.
                            value: row.lo + track * (row.bound.value() - row.lo),
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
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_add_button).is_some() => {
            let slot = slot_in(id, ids::model3d_add_button).unwrap_or(0);
            slot < state::current().adds.len() && {
                state::push_intent(ModelIntent::AddShape { slot });
                true
            }
        }
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_op_button).is_some() => {
            let slot = slot_in(id, ids::model3d_op_button).unwrap_or(0);
            slot < state::current().ops.len() && {
                state::push_intent(ModelIntent::ApplyOp { slot });
                true
            }
        }
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_mod_button).is_some() => {
            let slot = slot_in(id, ids::model3d_mod_button).unwrap_or(0);
            slot < state::current().mods.len() && {
                state::push_intent(ModelIntent::ToggleMod { slot });
                true
            }
        }
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_export_button).is_some() => {
            let slot = slot_in(id, ids::model3d_export_button).unwrap_or(0);
            slot < state::current().exports.len() && {
                state::push_intent(ModelIntent::Export { slot });
                true
            }
        }
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_act_button).is_some() => {
            let slot = slot_in(id, ids::model3d_act_button).unwrap_or(0);
            slot < state::current().acts.len() && {
                state::push_intent(ModelIntent::Act { slot });
                true
            }
        }
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_view_button).is_some() => {
            let slot = slot_in(id, ids::model3d_view_button).unwrap_or(0);
            slot < state::current().views.len() && {
                state::push_intent(ModelIntent::SetView { slot });
                true
            }
        }
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_camera_button).is_some() => {
            let slot = slot_in(id, ids::model3d_camera_button).unwrap_or(0);
            slot < state::current().camera.len() && {
                state::push_intent(ModelIntent::Camera { slot });
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
