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
        if id == crate::ids::model3d_radius_slider(n) {
            Some((n as usize, false))
        } else if id == crate::ids::model3d_radius_chip(n) {
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
            .find(|&cell| id == crate::ids::model3d_choice_button(linha, cell))
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

/// ⭐⭐⭐ **AS TREZE FILEIRAS DE CHIP, NUMA TABELA** — a família de ids · quantos chips o retrato
/// deste quadro publicou · que intenção um clique empurra.
///
/// # ⛔⛔ Porque ela substituiu treze braços copiados
///
/// Os treze eram **a mesma frase** treze vezes (`slot_in` → conferir contra a fileira → empurrar),
/// e uma fileira nova custava um bloco novo. ⚠️ O modo de falha desse molde já está medido nesta
/// casa: um bloco copiado com **a família certa e o comprimento do vizinho** confere o slot contra
/// a lista errada, e o botão fica morto só nas posições que a outra fileira não tem — *um defeito
/// que aparece na 6.ª posição de uma fileira de 8 e em lado nenhum antes*.
///
/// ⭐ **É a forma que este repo já mediu como a única sem knob morto** (`CLAUDE.md` §5.0: *«o único
/// painel 42/42 limpo é o gerado por TABELA»*), e ela leva o `apply_event` de `221` para dentro do
/// tecto de `200` **por corte de responsabilidade**, nunca por uma isenção.
///
/// ⚠️ **A conferência contra a fileira publicada NÃO é cerimónia:** o `populate` cunha `MAX_MODES`
/// ids por família **às cegas**, todo quadro, então um clique num id que o retrato não pintou é
/// alcançável — e sem ela viraria um pedido ao shell.
///
/// ⚠️ E o `+ Add shape…` entra aqui com a assinatura dos outros e **ignora o slot** de propósito:
/// ele abre a paleta (W100), que é a porta única de nascer uma forma.
type ChipRow = (
    fn(u32) -> ph2d_a11y::NodeId,
    fn(&state::ModelSnapshot) -> usize,
    fn(usize) -> ModelIntent,
);

const CHIP_ROWS: &[ChipRow] = &[
    (
        crate::ids::model3d_select_button,
        |s| s.selects.len(),
        |slot| ModelIntent::SetLassoMode { slot },
    ),
    (
        crate::ids::model3d_add_button,
        |s| s.adds.len(),
        // ⚠️ **Por SLOT desde 14/09**: a fileira tinha um chip e todo slot abria a paleta; o segundo
        // acende uma LUZ. ⛔ Um `_ =>` a abrir a paleta faria o chip novo parecer vivo e abrir a
        // coisa errada — *o pior dos dois, porque parece funcionar*.
        |slot| match slot {
            1 => ModelIntent::AddLight,
            _ => ModelIntent::OpenShapes,
        },
    ),
    (
        crate::ids::model3d_op_button,
        |s| s.ops.len(),
        |slot| ModelIntent::ApplyOp { slot },
    ),
    (
        crate::ids::model3d_verb_button,
        |s| s.verbs.len(),
        |slot| ModelIntent::SetVerb { slot },
    ),
    (
        crate::ids::model3d_character_button,
        |s| s.characters.len(),
        |slot| ModelIntent::SetCharacter { slot },
    ),
    (
        crate::ids::model3d_mod_button,
        |s| s.mods.len(),
        |slot| ModelIntent::ToggleMod { slot },
    ),
    (
        crate::ids::model3d_export_button,
        |s| s.exports.len(),
        |slot| ModelIntent::Export { slot },
    ),
    (
        crate::ids::model3d_act_button,
        |s| s.acts.len(),
        |slot| ModelIntent::Act { slot },
    ),
    (
        crate::ids::model3d_view_button,
        |s| s.views.len(),
        |slot| ModelIntent::SetView { slot },
    ),
    (
        crate::ids::model3d_camera_button,
        |s| s.camera.len(),
        |slot| ModelIntent::Camera { slot },
    ),
    // ⭐⭐⭐ **O SOMBREAMENTO** (`docs/Render3d/05`) — o modo, a vista da cena e a exposição.
    (
        crate::ids::model3d_shading_button,
        |s| s.shadings.len(),
        |slot| ModelIntent::SetShading { slot },
    ),
    (
        crate::ids::model3d_look_button,
        |s| s.looks.len(),
        |slot| ModelIntent::SetLook { slot },
    ),
    (
        crate::ids::model3d_exposure_button,
        |s| s.exposures.len(),
        |slot| ModelIntent::SetExposure { slot },
    ),
];

/// Este clique é de uma fileira de chips? Se for, empurra a intenção dela. Ver [`CHIP_ROWS`].
///
/// ⚠️ O retrato é lido **uma vez, e só quando uma família casa** — `state::current()` clona, e
/// perguntá-lo antes da varredura pagaria esse clone em todo clique do app.
fn chip_click(id: ph2d_a11y::NodeId) -> bool {
    CHIP_ROWS.iter().any(|(family, publicados, intent)| {
        slot_in(id, *family).is_some_and(|slot| {
            slot < publicados(&state::current()) && {
                state::push_intent(intent(slot));
                true
            }
        })
    })
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
                    // confusa. Ver `ParamRow::inert`.
                    Some(row) if row.inert.is_some() => false,
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
                Some(row) if row.inert.is_none() && (cell as usize) < row.choices.len() => {
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
                true
            }
        }
        // ⚠️ **O FECHAR vem ANTES da tabela** — o braço dela casa todo `Click` que sobra, e um
        // `_ => false` por baixo dele nunca corre. *Uma tabela que apanha tudo empurra para cima
        // tudo o que não é dela.*
        WidgetEvent::Click(id) if id == crate::ids::MODEL3D_CLOSE => {
            host.set_panel_visible(Model3dPanel::ID, false);
            true
        }
        // ⭐⭐⭐ **AS TREZE FILEIRAS, por TABELA** — ver [`CHIP_ROWS`]. Este braço substituiu treze
        // blocos que eram a mesma frase treze vezes, e é ele que mantém o `apply_event` dentro do
        // tecto de LOC **por corte de responsabilidade**, nunca por uma isenção.
        WidgetEvent::Click(id) => chip_click(id),
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
