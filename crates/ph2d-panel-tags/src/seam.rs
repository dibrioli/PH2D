//! A costura do painel TAGS — o registo dos widgets e o que um clique faz.
//!
//! ⚠️ **Registar é o que os torna clicáveis**: pintar + hit-rect não basta, e é a classe de defeito
//! que já matou botões deste repo várias vezes (o mais recente são os quatro chips da booleana, que
//! só o gesto REAL apanhou — um `Click` sintético passa com o chip morto).
//!
//! ⚠️ **Os verbos são registados INCONDICIONALMENTE**, mesmo os que só são pintados com uma linha em
//! mãos: o store é agnóstico de estado, e quem decide se o clique é possível é a PINTURA (sem
//! hit-rect não há `Click`). Registar só o pintado faria o `+ Child` nascer **morto sob o dedo** no
//! primeiro quadro em que alguém escolhe uma linha.
//!
//! ⚠️ **As LINHAS registam-se por quadro**, no `populate` vivo que a shell chama: a população delas
//! é o documento, e um `populate` de arranque não a conhece.

use crate::state::TagsPanelState;
use ph2d_editor_core::TagTreeEdit;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{
    InteractiveState, PanelRowDrop, PanelRowFamily, WidgetEvent, WidgetStore,
};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// **A tabela dos verbos** — a MESMA que o `rows::verbs` percorre para pintar.
///
/// ⚠️ Ela é escrita aqui por extenso (e não derivada do `verbs()`) porque aquele depende da linha em
/// mãos, que no arranque não existe; o gate `every_verb_of_the_bar_is_registered` ata as duas.
pub(crate) const VERBS: [ph2d_a11y::NodeId; 7] = [
    crate::ids::TAGS_NEW,
    crate::ids::TAGS_CHILD,
    crate::ids::TAGS_RENAME,
    crate::ids::TAGS_UNPARENT,
    crate::ids::TAGS_SELECT,
    crate::ids::TAGS_DELETE,
    crate::ids::TAGS_CLOSE,
];

pub(crate) fn populate(store: &mut WidgetStore) {
    for id in VERBS {
        button(store, id);
    }
}

/// ⭐⭐⭐ **O registo POR QUADRO das linhas — feito por QUEM AS PINTA.**
///
/// ⛔⛔ **A Hierarquia faz isto a partir da shell e aqui isso seria um defeito à espera.** Lá os
/// ids das linhas são ALOCADOS pela shell (`EntityNodeMap`, monótono por sessão), então só ela os
/// conhece; aqui eles são **derivados da identidade da tag** ([`crate::ids::row_id`]), logo o
/// painel sabe-os sozinho — e um registo que vive do outro lado da fronteira é um registo que
/// alguém esquece: *o gate de costura apanhou exactamente isso*, com as linhas **mortas sob o
/// dedo** enquanto a shell era a única a chamar.
///
/// ⚠️ `register_if_absent`: uma linha que sobrevive de um quadro para o outro mantém o hover, e uma
/// tag apagada deixa uma ranhura morta no store — fuga limitada e aceite, o mesmo contrato da lista
/// de entidades.
pub(crate) fn register_rows(store: &mut WidgetStore, tags: &[u64]) {
    for &t in tags {
        store.register_if_absent(crate::ids::row_id(t), InteractiveState::Plain);
    }
    // ⭐⭐⭐ **E declara-as ARRASTÁVEIS** (W4b) — é isto que faz o despacho armar um arrasto no Down
    // sobre uma linha e emitir um `PanelRowReparent` no Up.
    //
    // ⚠️ **Republicado a cada quadro, e o store apaga primeiro as da MESMA família:** sem isso uma
    // tag apagada ficaria para sempre arrastável sobre um sítio onde já não há linha nenhuma.
    store.set_panel_row_ids(
        PanelRowFamily::TagTree,
        tags.iter().map(|&t| crate::ids::row_id(t)).collect(),
    );
}

/// Abre o campo de renomear com `seed` dentro e o cursor no fim.
pub(crate) fn open_rename(store: &mut WidgetStore, seed: &str) {
    store.register(
        crate::ids::TAGS_RENAME_INPUT,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: seed.to_string(),
            caret: seed.len(),
            selection_anchor: None,
        },
    );
    store.set_focus(Some(crate::ids::TAGS_RENAME_INPUT));
    // O `Esc` aborta — a mesma marca que a renomeação da Hierarquia usa.
    store.mark_cancel_on_escape(crate::ids::TAGS_RENAME_INPUT);
}

/// A tag de uma linha, se `id` for uma delas.
fn tag_of(id: ph2d_a11y::NodeId) -> Option<u64> {
    crate::state::with_current(|info| {
        info.rows
            .iter()
            .find(|r| crate::ids::row_id(r.id) == id)
            .map(|r| r.id)
    })
}

/// O rótulo de uma tag, para semear o campo de renomear.
fn label_of(tag: u64) -> String {
    crate::state::with_current(|info| {
        info.rows
            .iter()
            .find(|r| r.id == tag)
            .map(|r| r.label.clone())
            .unwrap_or_default()
    })
}

/// O pai de uma tag na ordem da árvore — a linha ANTERIOR de profundidade `depth − 1`.
///
/// ⚠️ **Derivado da ordem, e não guardado:** a árvore chega ordenada com o pai imediatamente antes
/// dos descendentes (é a lei da chave), então o pai é o último antecessor mais raso. Um campo
/// `parent` no instantâneo seria um segundo sítio a dizer a mesma coisa.
fn parent_of(tag: u64) -> Option<u64> {
    crate::state::with_current(|info| {
        let i = info.rows.iter().position(|r| r.id == tag)?;
        let d = info.rows[i].depth;
        if d == 0 {
            return None;
        }
        info.rows[..i]
            .iter()
            .rev()
            .find(|r| r.depth == d - 1)
            .map(|r| r.id)
    })
}

fn push(host: &mut dyn PanelHostInternal, edit: TagTreeEdit) {
    host.bus_mut().push(EditorAction::TagTreeEdit { edit });
}

pub(crate) fn apply_event(
    state: &mut TagsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        // ⭐ **O duplo clique renomeia** — e ele vem ANTES do clique simples no `match`, porque o
        // despacho emite `DoubleClick` **em vez** do segundo `Click`.
        WidgetEvent::DoubleClick(id) if tag_of(id).is_some() => {
            let tag = tag_of(id).expect("acabou de ser lida");
            state.focus = Some(tag);
            state.renaming = Some(tag);
            let seed = label_of(tag);
            open_rename(host.store_mut(), &seed);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if tag_of(id).is_some() => {
            state.focus = tag_of(id);
            // ⚠️ Escolher outra linha FECHA um campo de renomear aberto noutra — senão o `Submit`
            // seguinte escreveria o nome na tag errada.
            state.renaming = None;
            EventOutcome::Consumed
        }
        WidgetEvent::Submit(id) if id == crate::ids::TAGS_RENAME_INPUT => {
            let alvo = state.renaming.take();
            let buf = match host.store().get(id) {
                Some(InteractiveState::TextInput { text, .. }) => text.clone(),
                _ => String::new(),
            };
            if let Some(tag) = alvo {
                push(
                    host,
                    TagTreeEdit::Rename {
                        id: tag,
                        label: buf,
                    },
                );
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == crate::ids::TAGS_RENAME_INPUT => {
            state.renaming = None;
            EventOutcome::Consumed
        }
        // ⭐⭐⭐ **ARRASTAR uma tag para dentro de outra** (W4b, gate 26) — o gesto do Blender.
        //
        // ⚠️ **`Before`/`After` e `Inside` NÃO são três respostas aqui, são DUAS**: a `TagTree`
        // ordena-se sozinha pela chave dobrada, então *«antes da Flying»* e *«depois da Flying»*
        // significam os dois **irmã da Flying** — o lugar na lista não é autorado, é derivado do
        // nome. ⛔ Fingir três destinos daria ao artista um gesto cujo efeito ele não consegue ver.
        WidgetEvent::PanelRowReparent {
            family: PanelRowFamily::TagTree,
            dragged,
            drop,
        } => {
            let Some(tag) = tag_of(dragged) else {
                return EventOutcome::Ignored;
            };
            let destino = match drop {
                PanelRowDrop::Inside(alvo) => tag_of(alvo).map(Some),
                PanelRowDrop::Before(alvo) | PanelRowDrop::After(alvo) => {
                    tag_of(alvo).map(parent_of)
                }
                // Largada abaixo de todas as linhas: raiz.
                PanelRowDrop::End => Some(None),
            };
            if let Some(parent) = destino
                // ⚠️ **Largar sobre o PAI que já se tem é no-op** — o `move_under` aceitá-lo-ia e a
                // revisão subiria, o que daria um passo de undo sobre uma árvore que não mudou.
                && parent != parent_of(tag)
            {
                state.focus = Some(tag);
                push(host, TagTreeEdit::Move { id: tag, parent });
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == crate::ids::TAGS_CLOSE => {
            host.set_panel_visible(crate::TagsPanel::ID, false);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == crate::ids::TAGS_NEW => {
            push(host, TagTreeEdit::Create { parent: None });
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == crate::ids::TAGS_CHILD => {
            if let Some(tag) = state.focus {
                push(host, TagTreeEdit::Create { parent: Some(tag) });
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == crate::ids::TAGS_RENAME => {
            if let Some(tag) = state.focus {
                state.renaming = Some(tag);
                let seed = label_of(tag);
                open_rename(host.store_mut(), &seed);
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == crate::ids::TAGS_UNPARENT => {
            if let Some(tag) = state.focus {
                push(
                    host,
                    TagTreeEdit::Move {
                        id: tag,
                        parent: None,
                    },
                );
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == crate::ids::TAGS_SELECT => {
            if let Some(tag) = state.focus {
                push(host, TagTreeEdit::SelectTagged { id: tag });
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == crate::ids::TAGS_DELETE => {
            if let Some(tag) = state.focus {
                // ⚠️ A linha em mãos some com o gesto: deixá-la posta faria a barra seguinte
                // oferecer verbos sobre uma tag que já não existe.
                state.focus = parent_of(tag);
                state.renaming = None;
                push(host, TagTreeEdit::Delete { id: tag });
            }
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}
