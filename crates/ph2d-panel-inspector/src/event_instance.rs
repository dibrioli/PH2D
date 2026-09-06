//! ⭐⭐⭐ **OS CLIQUES DO CARTÃO DE INSTÂNCIA** (ADR-0164 / F5) — irmão por ASSUNTO do
//! [`super::event`], como o `event_anchor`, o `event_anim` e o `event_value`.
//!
//! ⚠️ **O corte foi imposto pelo `panel_files_under_loc_cap`** (612 de 600) quando o *Put back*
//! entrou, e o assunto estava à mão: os quatro handlers respondem à mesma superfície — o cartão do
//! topo — e nenhum deles toca no `InspectorState`. *Um tecto paga-se com um corte.*
//!
//! # ⚠️ A lei que os quatro partilham: o ÍNDICE do botão morre AQUI
//!
//! Todos traduzem *«a linha `i` do cartão que este quadro pintou»* para a **identidade** que o
//! mundo entende — o `StableId` da receita, a chave da excepção, o id da peça. O cartão é
//! reconstruído a cada quadro e as listas reordenam-se; mandar o índice pelo barramento seria pedir
//! ao mundo que resolvesse *«a terceira»* contra uma lista que já pode ter outra terceira.
//!
//! ⛔ E **um botão que a lista deste quadro não pinta não despacha**: o `get(i)` sobre a lista é a
//! mesma porta que o pintor percorre.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;

/// ⭐⭐ **Limpar as excepções SEM ALVO** (ADR-0164 / F5.3).
///
/// ⚠️ **O painel diz QUEM pediu, não o que fazer** — o `root_bits` é a RAIZ da instância, que é
/// onde o `ObjectInstance` mora. A shell é quem tem o mundo; este ficheiro só honra o clique.
///
/// ⚠️ **Função irmã, e não um braço do `apply_event_impl`** — o precedente é o
/// [`add_component_click`] logo abaixo, e a razão é a mesma (o teto de LOC daquela função).
pub(crate) fn clear_orphans_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if ev != WidgetEvent::Click(ids::INSP_INSTANCE_CLEAR_ORPHANS) {
        return false;
    }
    let Some(root_bits) = crate::state::current_inspector_instance().map(|i| i.root_bits) else {
        return false;
    };
    host.bus_mut()
        .push(EditorAction::InspectorClearUnusedOverrides { root_bits });
    true
}

/// ⭐⭐⭐ **UM DEGRAU DA ESCADA do *Aplicar*** (ADR-0164 / F5, critério 4).
///
/// ⚠️ **A leitura inversa vem da PORTA** (`ids::instance_apply_level`), e o que viaja no barramento
/// é a **identidade da receita**, nunca o índice do botão: a escada é derivada do mundo e
/// reordena-se quando uma receita é aninhada — um índice diria *«o segundo»* a quem já não tem o
/// mesmo segundo.
///
/// ⚠️ **O sujeito é a PEÇA selecionada**, e não a raiz da cópia: o escopo do *Aplicar* é o que se
/// clicou (a lei do *Revert*), e a escada que o cartão mostrou é a daquela peça.
pub(crate) fn apply_level_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(level) = ids::instance_apply_level(id) else {
        return false;
    };
    let Some(info) = crate::state::current_inspector_instance() else {
        return false;
    };
    // ⛔ **Um degrau que o cartão não pinta não despacha** — o `apply_rows` é a mesma porta que o
    // pintor usa, e perguntar-lhe aqui é o que impede um botão de outro quadro de chegar a um
    // índice que já não existe.
    let Some(choice) = info.apply_rows().get(level) else {
        return false;
    };
    host.bus_mut().push(EditorAction::InspectorApplyToLevel {
        entity_bits: info.entity_bits,
        master: choice.master,
    });
    true
}

/// ⭐⭐⭐ **Largar UMA excepção sem alvo** (F5.3-ter) — o `✕` da linha dela.
///
/// ⚠️ **Ele traduz o índice do botão para a CHAVE aqui, e não no shell**: a linha `i` só significa
/// alguma coisa dentro do cartão que este quadro pintou, e o cartão é reconstruído a cada quadro.
/// Mandar o índice pelo barramento seria pedir ao mundo que resolvesse *«a terceira»* contra uma
/// lista que já pode ter outra terceira — a mesma lei do `apply_level_click` logo acima.
///
/// ⛔ **E um botão que a lista deste quadro não pinta não despacha:** o `get(i)` sobre o
/// `orphan_rows` é a mesma porta que o pintor percorre.
pub(crate) fn drop_orphan_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(i) = ids::instance_drop_orphan(id) else {
        return false;
    };
    let Some(info) = crate::state::current_inspector_instance() else {
        return false;
    };
    let Some(row) = info.orphan_rows.get(i) else {
        return false;
    };
    host.bus_mut()
        .push(EditorAction::InspectorDropUnusedOverride {
            root_bits: info.root_bits,
            piece: row.piece_id,
            type_id: row.type_id,
        });
    true
}

/// ⭐⭐⭐ **Devolver uma peça que a cópia recusou** (F5.10) — o *Put back* da linha.
///
/// ⚠️ Ele traduz o índice do botão para a CHAVE aqui, e não no shell — a mesma lei do
/// `drop_orphan_click` e do `apply_level_click`: a lista é reconstruída a cada quadro.
pub(crate) fn restore_piece_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(i) = ids::instance_restore_piece(id) else {
        return false;
    };
    let Some(info) = crate::state::current_inspector_instance() else {
        return false;
    };
    let Some(row) = info.removed_rows.get(i) else {
        return false;
    };
    host.bus_mut()
        .push(EditorAction::InspectorRestoreRemovedPiece {
            root_bits: info.root_bits,
            piece: row.piece_id,
        });
    true
}
