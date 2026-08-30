//! **O despacho da §12 Sockets / Named Anchors** ([ADR-0072]).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — mesmo padrão do [`crate::event_slice`].
//!
//! # A seleção de linha NÃO vai ao barramento
//!
//! Clicar numa linha da lista muda **estado do painel** (`InspectorState::anchor_selected`) e
//! consome o evento. Publicá-la como ação obrigaria a shell a saber qual ficha está aberta — um
//! facto que só a UI tem — e faria toda troca de linha custar um quadro. É a mesma fronteira
//! que separa «que painel está visível» de «o que a cena contém».
//!
//! [ADR-0072]: ../../../docs/architecture/decisions/0072-named-anchor-unification.md

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::AnchorFieldEdit;
use ph2d_editor_core::widget::{ButtonState, CheckboxValue};

use crate::state::{self, InspectorState};

/// Despacha um evento da §12. `true` = consumido.
pub(crate) fn apply_anchor_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = state::current_inspector_anchor() else {
        return false;
    };
    let sel = panel.anchor_selected.min(info.rows.len().saturating_sub(1));
    let sel_u8 = u8::try_from(sel).unwrap_or(0);

    if let WidgetEvent::Click(id) = ev {
        // Uma linha da lista: só muda a ficha aberta.
        if let Some(i) = ids::INSP_ANCHOR_ROW.iter().position(|&o| o == id)
            && i < info.rows.len()
        {
            panel.anchor_selected = i;
            demote_button(host, id);
            return true;
        }
        if id == ids::INSP_ANCHOR_ADD {
            push(host, info.entity_bits, AnchorFieldEdit::Add);
            demote_button(host, id);
            return true;
        }
        if id == ids::INSP_ANCHOR_REMOVE && !info.rows.is_empty() {
            push(host, info.entity_bits, AnchorFieldEdit::Remove(sel_u8));
            demote_button(host, id);
            return true;
        }
        // **Uma opção do seletor «Rides Parent Anchor».**
        //
        // ⚠️ O que viaja é o NOME, nunca o índice: o índice é do widget e vale só neste quadro,
        // enquanto o vínculo tem de sobreviver a reordenar e a reabrir o projeto.
        if let Some(name) = mount_choice(&info, id) {
            close_mount_popover(host);
            push(host, info.entity_bits, AnchorFieldEdit::Mount(name));
            demote_button(host, id);
            return true;
        }
        // ⚠️ **A guarda `is_off_anchor` está aqui também, e não só na pintura.** O botão é
        // registado no arranque e um clique sintético alcança-o mesmo quando não é pintado;
        // sem esta linha, o gesto escreveria uma pose zerada sobre um objeto que não monta.
        if id == ids::INSP_MOUNT_SNAP && info.is_off_anchor() {
            push(host, info.entity_bits, AnchorFieldEdit::SnapToAnchor);
            demote_button(host, id);
            return true;
        }
    }

    // Sem âncoras não há nada que os campos abaixo possam editar.
    //
    // ⚠️ O braço do **snap** fica ACIMA desta guarda de propósito: quem monta numa âncora do pai
    // pode não ter âncora nenhuma sua, e o botão dele não pode depender disso.
    if info.rows.is_empty() {
        return false;
    }

    // A caixa de VISIBILIDADE do dono das âncoras — **uma**, e já não duas.
    //
    // ⛔⛔ **A «Show anchors at runtime» saiu daqui em 2026-08-30, com o bloqueador NOMEADO:** não
    // existe modo de jogo (`shells/game` / Runtime R1, adiado pelo dono do produto), então
    // `AnchorVisibility::at_runtime` gravava e **não tinha um único leitor**. Ela continua pintada
    // (a cinzento, com a razão no rótulo) e registada `Disabled`, e o campo continua no modelo para
    // não partir ficheiros gravados — o que sai é a PROMESSA.
    //
    // ⚠️ **A recusa vive AQUI também, e não só no registo `Disabled`.** Um `Toggled` sintético
    // alcança o braço sem passar pelo `is_focusable`, e é exactamente a lacuna que o braço do
    // `INSP_MOUNT_SNAP` acima documenta. Sem esta linha o valor voltava a mudar por uma porta que
    // ninguém vê.
    //
    // ⚠️ **A IRMÃ fica** — «Always show anchors» tem consumidor vivo (`anchor_overlay`).
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_ANCHOR_VIS_EDITOR
    {
        let on = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        push(
            host,
            info.entity_bits,
            AnchorFieldEdit::VisibilityInEditor(on),
        );
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && id == ids::INSP_ANCHOR_NAME
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        push(
            host,
            info.entity_bits,
            AnchorFieldEdit::Rename(sel_u8, text),
        );
        return true;
    }

    if let WidgetEvent::Toggled(id) = ev
        && matches!(id, ids::INSP_ANCHOR_BOUNDS_ON | ids::INSP_ANCHOR_CENTER_ON)
    {
        let on = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        let edit = if id == ids::INSP_ANCHOR_BOUNDS_ON {
            AnchorFieldEdit::BoundsOn(sel_u8, on)
        } else {
            AnchorFieldEdit::CenterOn(sel_u8, on)
        };
        push(host, info.entity_bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        // ⚠️ Cada campo despacha SÓ o seu eixo — a lei do `PerCornerTintAt`.
        let edit = ids::INSP_ANCHOR_POS
            .iter()
            .position(|&o| o == id)
            .map(|i| AnchorFieldEdit::Pos(sel_u8, i as u8, v))
            .or_else(|| (id == ids::INSP_ANCHOR_ROT).then_some(AnchorFieldEdit::Rot(sel_u8, v)))
            .or_else(|| {
                ids::INSP_ANCHOR_BOUNDS
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| AnchorFieldEdit::Bounds(sel_u8, i as u8, v))
            })
            .or_else(|| {
                ids::INSP_ANCHOR_CENTER
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| AnchorFieldEdit::Center(sel_u8, i as u8, v))
            });
        if let Some(edit) = edit {
            push(host, info.entity_bits, edit);
            return true;
        }
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: AnchorFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorAnchorEdit { entity_bits, edit });
}

/// **Que montagem este id escolhe.**
///
/// `None` = o id não é uma opção do seletor. `Some(None)` = a opção «—» (não montar).
/// `Some(Some(nome))` = montar nessa âncora do pai.
///
/// ⚠️ **A opção «—» é reconhecida SEMPRE**, mesmo com a lista do pai vazia — é ela que desfaz um
/// vínculo pendurado, e um vínculo que não se pode desfazer é um estado preso.
fn mount_choice(
    info: &ph2d_editor_core::screens::hero::InspectorAnchorInfo,
    id: ph2d_a11y::NodeId,
) -> Option<Option<String>> {
    if id == ids::INSP_MOUNT_NONE_OPT {
        return Some(None);
    }
    let i = ids::INSP_MOUNT_OPT.iter().position(|&o| o == id)?;
    // ⚠️ O array tem 64 ids sempre pintados-ou-não; só os que a lista do pai alcança valem.
    // Sem esta guarda, um id registado e nunca pintado escolheria uma âncora inexistente.
    info.parent_anchors.get(i).cloned().map(Some)
}

/// Fecha o popover do seletor depois de uma escolha.
///
/// ⚠️ **E NÃO escreve o `selected_index`.** Quem é dono do que está montado é o snapshot: o
/// próximo quadro relê-o da cena. Escrever aqui criaria a segunda porta para o mesmo estado — e
/// ela mentiria exatamente quando a shell recusasse a edição.
fn close_mount_popover(host: &mut dyn PanelHostInternal) {
    if let Some(InteractiveState::Dropdown { open, .. }) =
        host.store_mut().get_mut(ids::INSP_MOUNT_PICK)
    {
        *open = false;
    }
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote_button(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
