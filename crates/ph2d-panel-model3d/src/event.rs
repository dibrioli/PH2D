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

/// De que **linha e botão** de uma fileira de escolha é este id, se for de alguma.
///
/// ⚠️ **Uma varredura sobre as duas dimensões da família**, pela razão do [`slot_of`]: um `NodeId` é
/// um hash de nome e hash não se inverte. `64 × 4` corre uma vez por evento, não por quadro.
fn choice_of(id: ph2d_a11y::NodeId) -> Option<(usize, u32)> {
    (0..MAX_ROWS as u32).find_map(|linha| {
        (0..crate::populate::MAX_CHOICES)
            .find(|&cell| id == ids::model3d_choice_button(linha, cell))
            .map(|cell| (linha as usize, cell))
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
        // ⭐⭐⭐ **A FILEIRA DE ESCOLHA DE UMA LINHA** — o eixo de um modificador (Enio,
        // 2026-08-31). ⚠️ Ela despacha o **mesmo** `SetParam` que o slider da linha: para a porta do
        // documento uma escolha é um número, e o que muda é só o controlo que o produz.
        WidgetEvent::Click(id) if choice_of(id).is_some() => {
            let (slot, cell) = choice_of(id).unwrap_or((0, 0));
            let snap = state::current();
            match snap.rows.get(slot) {
                // ⚠️ **Uma linha inerte não despacha**, e um id cuja linha deste quadro não é uma
                // escolha também não: a família está registada às cegas para `MAX_ROWS × MAX_CHOICES`
                // (ver `populate`), então um clique num id que o retrato não pintou é alcançável.
                Some(row) if row.live && (cell as usize) < row.choices.len() => {
                    state::push_intent(ModelIntent::SetParam {
                        entity: row.entity,
                        param: row.param,
                        // ⭐ **O ÍNDICE do botão é o valor** — ver `ph2d_field::Span::Choice`. Sem
                        // trilha, sem faixa: a posição no ecrã não entra na conta, e é isso que
                        // separa uma escolha de um slider de três posições.
                        value: cell as f32,
                    });
                    true
                }
                _ => false,
            }
        }
        // ⚠️ **A `line/UIUX` TROCOU estes dois braços** (o verbo e o referencial passaram para os
        // chips do TRILHO, que já existiam e estavam MORTOS) e a `line/3DModeling` acrescentou dois
        // vizinhos — a fileira de ESCOLHA de uma linha e o modo do LAÇO. ⇒ na integração de
        // 2026-09-04 ficam os dois que ela acrescentou (ninguém os moveu para lado nenhum) e os da
        // outra pelo braço NOVO. ⛔ Manter também os antigos daria DUAS portas para o mesmo verbo.
        // ⭐⭐⭐ **OS CHIPS DO TRILHO QUE JÁ EXISTIAM** — `MOVE`/`ROT`/`SCALE` conduzem o verbo do
        // gizmo, e o `SPACE` o referencial.
        //
        // > Enio, 2026-09-01 (com foto): *«esses botões de mover, rot e scale já existiam. só não
        // > estavam ligados a cada modo.»*
        //
        // ⛔⛔ **Eles eram controlos MORTOS** (a 2.ª espécie do `CLAUDE.md` §5.0): o clique chegava,
        // a luz acendia, e o valor não alcançava consumidor nenhum. ⇒ ligá-los é a cura; construir
        // um selector NOVO ao lado seria a segunda porta.
        //
        // ⚠️ **A guarda `armed` é obrigatória:** o registry entrega todo evento a todo painel,
        // fechado incluído, e sem ela um módulo 3D fora de cena roubaria estes cliques ao editor
        // 2D. Ver [`state::set_armed`].
        WidgetEvent::Click(id)
            if state::armed()
                && crate::area_bar::rail_verb_slot(id, &state::current()).is_some() =>
        {
            let snap = state::current();
            let slot = crate::area_bar::rail_verb_slot(id, &snap).unwrap_or(0);
            state::push_intent(ModelIntent::SetGizmoMode { slot });
            true
        }
        // ⚠️ **O `SPACE` é um INTERRUPTOR e o referencial é uma FILEIRA** — a ponte é *avançar para
        // o próximo*, e não um índice fixo: com dois referenciais isso é exactamente alternar, e
        // com um terceiro continua a ser o que o chip promete (cada toque muda para o seguinte).
        WidgetEvent::Click(id) if state::armed() && id == ids::TOOL_SPACE => {
            let snap = state::current();
            let n = snap.frames.len();
            n > 0 && {
                let active = snap.frames.iter().position(|c| c.active).unwrap_or(0);
                state::push_intent(ModelIntent::SetGizmoFrame {
                    slot: (active + 1) % n,
                });
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_select_button).is_some() => {
            let slot = slot_in(id, ids::model3d_select_button).unwrap_or(0);
            slot < state::current().selects.len() && {
                state::push_intent(ModelIntent::SetLassoMode { slot });
                true
            }
        }
        // ⭐⭐⭐ **UM botão, e ele ABRE a paleta** (W100) — ver [`crate::ModelIntent::OpenShapes`].
        //
        // ⚠️ O `slot` continua a ser conferido contra a fileira publicada, e não é cerimónia: a
        // família de ids tem `MAX_MODES` slots registados sempre, então um clique num id que o
        // retrato deste quadro não pintou é alcançável — e sem a guarda ele viraria um pedido.
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_add_button).is_some() => {
            let slot = slot_in(id, ids::model3d_add_button).unwrap_or(0);
            slot < state::current().adds.len() && {
                state::push_intent(ModelIntent::OpenShapes);
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
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_verb_button).is_some() => {
            let slot = slot_in(id, ids::model3d_verb_button).unwrap_or(0);
            slot < state::current().verbs.len() && {
                state::push_intent(ModelIntent::SetVerb { slot });
                true
            }
        }
        WidgetEvent::Click(id) if slot_in(id, ids::model3d_character_button).is_some() => {
            let slot = slot_in(id, ids::model3d_character_button).unwrap_or(0);
            slot < state::current().characters.len() && {
                state::push_intent(ModelIntent::SetCharacter { slot });
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
