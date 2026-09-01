//! ⭐⭐⭐ **O VALOR de uma propriedade: escolher um, e RENOMEAR o vigente.**
//!
//! # ⛔⛔⛔ Ele tentou quatro vezes, e o app nunca o deixou
//!
//! Report do Enio (2026-08-31): *«Que inferno!!!»* — depois de escrever `Canvas{Size=Big}` no nome
//! de uma **cópia** pela quarta vez, à espera de que o botão passasse a dizer `Big`.
//!
//! ⚠️ **O modelo estava certo as quatro vezes:** uma propriedade é do COMPONENTE, e a cópia
//! herda-a. O defeito é outro, e é de fluxo: **autorar o valor obrigava a seleccionar OUTRO
//! objecto** (a receita, escondida entre linhas quase iguais) do que aquele que se está a olhar.
//! *O artista olha para o cartão, o valor está no cartão, e o sítio onde ele se muda estava noutra
//! linha da Hierarquia.*
//!
//! ⭐ **E o gesto não é novo: era um clique MORTO.** Carregar no chip já aceso era um no-op
//! silencioso, e é exactamente onde o dedo dele estava. Hoje abre o campo.
//!
//! # ⚠️ Irmão do `event_anchor` e do `event_anim`, e pela mesma razão
//!
//! Ele precisa do `InspectorState` — *qual eixo está aberto* é estado de painel, não uma edição da
//! cena —, e o `apply_event_impl` só vê o `host`.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::TextInputState;

use crate::state::InspectorState;

/// `true` quando o evento era desta família.
pub(crate) fn apply_value_event(
    state: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    match ev {
        WidgetEvent::Click(id) => chip_click(state, host, id),
        // ⚠️ **Enter e o clique-fora GRAVAM; o Esc abandona** — o idioma dos outros campos em
        // linha desta casa. O `take` faz do segundo de um par `Submit`+`Blur` um no-op.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::INSP_INSTANCE_VALUE_EDIT => {
            commit(state, host);
            true
        }
        WidgetEvent::Cancel(id) if id == ids::INSP_INSTANCE_VALUE_EDIT => {
            state.value_edit = None;
            true
        }
        // ⭐⭐⭐ **O campo do NOME fechou** — e as chaves que ele declara passam a valer (decisão
        // do Enio, 2026-08-31). ⛔ **Aqui e não no `TextChanged`**: aquele chega por TECLA, e a
        // consequência disto é fora do objecto que se edita. Ver `EditorAction::InspectorNameCommitted`.
        //
        // ⚠️ **`Ignored` de propósito** — a edição do nome em si continua a ser tratada pelo
        // caminho de sempre; isto só ACRESCENTA o commit.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::INSP_ENTITY_NAME => {
            if let Some(info) = crate::state::current_inspector_name() {
                host.bus_mut().push(EditorAction::InspectorNameCommitted {
                    entity_bits: info.entity_bits,
                });
            }
            false
        }
        _ => false,
    }
}

/// ⭐⭐⭐ **Largar o campo sem gravar** — a porta que TODA saída anormal chama.
///
/// # ⛔⛔ O molde tem TRÊS portas destas, e a 1.ª versão não tinha NENHUMA
///
/// A auditoria de 2026-08-31 (A1/A3) mediu o preço: trocar a seleção deixava um campo com cara de
/// focado e teclado morto; fechar o painel deixava um `TextInput` focado e **invisível a comer as
/// teclas do app inteiro** — o defeito que o `paint_catalog_rename` do navegador de assets nomeia
/// letra a letra e abandona por três portas (coluna colapsada · painel fechado · alvo sumido).
///
/// ⚠️ **O foco só se larga se for NOSSO** — largar o foco alheio roubaria a caixa que o artista
/// acabou de clicar noutra secção.
pub(crate) fn abandon(
    state: &mut InspectorState,
    store: &mut ph2d_editor_core::interaction::WidgetStore,
) {
    if state.value_edit.take().is_some() && store.focus_id() == Some(ids::INSP_INSTANCE_VALUE_EDIT)
    {
        store.set_focus(None);
    }
}

/// ⭐⭐ **O chip: trocar de versão, ou abrir o vigente para escrita.**
///
/// ⚠️ **O painel manda o `StableId` do mestre, e não o índice do chip.** O índice é uma posição
/// numa lista que o construtor refaz por quadro; se ela reordenar entre o pintar e o clicar, o
/// artista escolhe `Large` e recebe `Medium` — **sem erro nenhum**. *A identidade viaja; a posição
/// fica no painel.*
fn chip_click(
    state: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    id: ids::NodeId,
) -> bool {
    // ⚠️ **A leitura inversa vem da PORTA** (`ids::instance_axis_option`), e ela vem ANTES do
    // estado: este braço corre em TODO clique do Inspector, e o `current_inspector_properties()`
    // **clona** o cartão inteiro. Perguntar primeiro o que custa 3 ns é a ordem certa das duas.
    let Some((a, v)) = ids::instance_axis_option(id) else {
        return false;
    };
    let Some(info) = crate::state::current_inspector_properties() else {
        return false;
    };
    let Some(choice) = info.rows.get(a).and_then(|ax| ax.options.get(v)) else {
        return false;
    };
    if choice.current {
        // ⛔⛔ **No modo PLANO o chip aceso NÃO abre campo nenhum** (auditoria de 2026-08-31): o
        // eixo plano não tem chave, então o `commit` recusaria — e um campo que abre, aceita
        // texto e nunca grava é um controlo que come trabalho em silêncio. A decisão de ABRIR e a
        // de GRAVAR têm de ler a MESMA pergunta.
        if info.rows.get(a).is_none_or(|ax| ax.name.is_empty()) || choice.master == 0 {
            return true;
        }
        // ⭐ **Nasce cheio e SELECCIONADO** — a lição do `CatalogHeroes`: um campo aberto por um
        // gesto que diz *«muda isto»* já sabe que o artista quer OUTRO texto.
        let seed = choice.label.clone();
        host.store_mut().register(
            ids::INSP_INSTANCE_VALUE_EDIT,
            InteractiveState::TextInput {
                state: TextInputState::Focused,
                caret: seed.len(),
                selection_anchor: Some(0),
                text: seed,
            },
        );
        host.store_mut()
            .set_focus(Some(ids::INSP_INSTANCE_VALUE_EDIT));
        // O Esc aborta — quem o diz é o CAMPO, não uma lista de ids dentro do `dispatch_key`.
        host.store_mut()
            .mark_cancel_on_escape(ids::INSP_INSTANCE_VALUE_EDIT);
        state.value_edit = Some(crate::state::ValueEdit {
            entity_bits: info.entity_bits,
            axis: info
                .rows
                .get(a)
                .map(|ax| ax.name.clone())
                .unwrap_or_default(),
        });
        return true;
    }
    // ⚠️ **Trocar de versão FECHA o campo**: aberto sobre um valor que já não é o vigente, o
    // `Blur` seguinte gravaria o texto na propriedade errada.
    state.value_edit = None;
    // ⛔ **Sem raiz não há a quem pedir a troca** — é o estado de um objecto que DECLARA
    // propriedades sem ser cópia de nada. O cartão pinta essas fileiras como texto, então este
    // braço não é alcançável pelo ponteiro; ele existe para que a ausência não vire um `root_bits`
    // de `0` a viajar no barramento.
    if info.root_bits == 0 || choice.master == 0 {
        return true;
    }
    host.bus_mut().push(EditorAction::InspectorSwapVariant {
        root_bits: info.root_bits,
        master: choice.master,
    });
    true
}

/// ⭐⭐⭐ **A gravação: o sujeito é a RECEITA, e a chave é o NOME DA FILEIRA.**
///
/// ⚠️ **Nada acontece sem os dois** — o eixo aberto tem de continuar a existir (o cartão refaz-se
/// por quadro, e uma troca de selecção pode tê-lo levado) e ele tem de ter um vigente com mestre.
/// *Um campo cujo alvo desapareceu abandona; ele não grava no que sobrou.*
fn commit(state: &mut InspectorState, host: &mut dyn PanelHostInternal) {
    let Some(edit) = state.value_edit.take() else {
        return;
    };
    let Some(InteractiveState::TextInput { text, .. }) =
        host.store().get(ids::INSP_INSTANCE_VALUE_EDIT)
    else {
        return;
    };
    let value = text.clone();
    let Some(info) = crate::state::current_inspector_properties() else {
        return;
    };
    // ⛔⛔⛔ **A IDENTIDADE decide, nunca a posição** (auditoria de 2026-08-31, achados A1/A5): o
    // `Blur` chega com o snapshot VIGENTE, que pode já ser de OUTRO objecto (a seleção mudou) ou
    // ter as fileiras reordenadas. Um índice gravava o texto de A na receita de B, ou na chave
    // errada do mesmo A. ⇒ o campo só grava se o cartão ainda for da MESMA entidade e o eixo com
    // AQUELE nome ainda existir; senão, ABANDONA — *um campo cujo alvo desapareceu não grava no
    // que sobrou*.
    if info.entity_bits != edit.entity_bits {
        return;
    }
    let Some(axis) = info.rows.iter().find(|ax| ax.name == edit.axis) else {
        return;
    };
    let Some(current) = axis.options.iter().find(|o| o.current) else {
        return;
    };
    if axis.name.is_empty() || current.master == 0 {
        return;
    }
    host.bus_mut()
        .push(EditorAction::InspectorRenameVariantValue {
            master: current.master,
            key: axis.name.clone(),
            value,
            root_bits: info.root_bits,
        });
}
