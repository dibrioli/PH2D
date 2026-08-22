//! **O despacho da §5 9-Slice.**
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o `apply_event_impl` tem catraca em 384
//! (`architecture_panel_loc_cap`) e *a catraca só desce*. Mesmo padrão do `event_ordering`,
//! `event_sprite_value` e `event_precision`.
//!
//! # A célula da grelha CICLA contra o snapshot
//!
//! ⚠️ Os oito modos por-região são botões momentâneos: quem sabe em que modo a região está é a
//! **entidade**, não o widget. Ciclar contra um estado guardado no botão daria modos diferentes
//! em sprites diferentes depois do primeiro clique — é a mesma lei que o bit de camada da §8 já
//! paga (`a_layer_bit_toggles_against_the_snapshot`).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::SliceFieldEdit;
use ph2d_editor_core::widget::{ButtonState, CheckboxValue};

use crate::state;

/// Quantos modos por-região existem. ⚠️ Espelha `TileRegionMode::ALL.len()`; o gate da shell
/// (`the_slice_section_offers_every_mode_the_engine_has`) prende os dois, porque o painel não
/// depende do `ph2d-ecs`.
const REGION_MODE_COUNT: u8 = 4;

/// Quantos modos o MIOLO cicla — três, sem o `Blank`: apagar o miolo é o `Fill Center`.
const CENTRE_MODE_COUNT: u8 = 3;

/// Tag de `TileRegionMode::Stretch`.
const MODE_STRETCH: u8 = 0;
/// Tag de `TileRegionMode::Repeat`.
const MODE_REPEAT: u8 = 1;
/// Tag de `TileRegionMode::Blank`.
const MODE_BLANK: u8 = 3;

/// O próximo modo da célula `i` da grelha.
///
/// ⚠️ **Um CANTO só tem duas posições, e descobri-lo foi o achado nº 1 da auditoria de
/// 2026-08-22.** Um canto nunca ladrilha — por construção: `tile_count` só repete no eixo que
/// *cresce*, e num canto nenhum dos dois cresce (ele fica no tamanho intrínseco, que é a razão de
/// existir do 9-slice). Logo `Stretch`, `Repeat` e `Mirror` produzem geometria **idêntica** ali:
/// medido, três dos quatro estados eram inertes, e a célula ciclava por eles a fingir que fazia
/// alguma coisa. *Um controlo com quatro posições sobre um modelo de duas é a afordância a
/// mentir* — as duas que um canto tem são **desenhar** e **não desenhar**.
fn next_region_mode(cell: usize, cur: u8) -> u8 {
    if is_corner(cell) {
        return if cur == MODE_BLANK {
            MODE_STRETCH
        } else {
            MODE_BLANK
        };
    }
    (cur + 1) % REGION_MODE_COUNT
}

/// A célula `i` é um dos quatro cantos? Deriva de [`crate::REGION_CELLS`] — a mesma tabela que o
/// pintor itera e que o gate da shell confronta com o motor, **nunca** uma segunda cópia.
fn is_corner(cell: usize) -> bool {
    crate::REGION_CELLS
        .get(cell)
        .is_some_and(|&(col, row)| col != 1 && row != 1)
}

/// Despacha um evento da §5. `true` = consumido.
pub(crate) fn apply_slice_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // ⚠️ **A CAIXA que liga o 9-slice é UMA edição, não duas.**
    //
    // Ligá-la numa sprite sem componente escreve `DrawMode(1)`, e o commit da shell já anexa o
    // componente em qualquer edição de campo — por isso não é preciso um `Attach` antes. Emitir
    // os dois seria **dois passos de undo para um clique**, e a variante `Attach` deixou de ter
    // produtor: foi retirada em vez de ficar como um braço que ninguém alcança.
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_SLICE_ENABLE
        && let Some(info) = state::current_inspector_slice()
    {
        let on = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        host.bus_mut().push(EditorAction::InspectorSliceEdit {
            entity_bits: info.entity_bits,
            edit: SliceFieldEdit::DrawMode(u8::from(on)),
        });
        return true;
    }

    // Retirar existe SEM snapshot de componente — ele é o que o apaga.
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_slice()
        && id == ids::INSP_SLICE_REMOVE
    {
        host.bus_mut().push(EditorAction::InspectorSliceEdit {
            entity_bits: info.entity_bits,
            edit: SliceFieldEdit::Detach,
        });
        demote_button(host, id);
        return true;
    }

    // O segmentado do Tile Mode: a POSIÇÃO no array É a tag.
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_slice()
    {
        let edit = ids::INSP_SLICE_TILE_MODE
            .iter()
            .position(|&o| o == id)
            .map(|i| SliceFieldEdit::TileMode(i as u8))
            .or_else(|| {
                // ⚠️ **O MIOLO cicla só três** — Stretch → Repeat → Mirror. `Blank` é o que o
                // `Fill Center` já exprime, e oferecê-lo aqui daria duas portas para o mesmo
                // estado.
                (id == ids::INSP_SLICE_CENTRE).then(|| {
                    let next = (info.centre_tile_mode + 1) % CENTRE_MODE_COUNT;
                    SliceFieldEdit::CentreMode(next)
                })
            })
            // ⚠️ **Os dois atalhos escrevem UMA edição, não nove.** Nove ações no barramento
            // seriam nove passos de undo para um gesto só, e o `Ctrl+Z` desfaria a grelha célula
            // a célula — a lei «um gesto, um passo».
            .or(match id {
                ids::INSP_SLICE_ALL_TILE => Some(SliceFieldEdit::AllRegions(MODE_REPEAT)),
                ids::INSP_SLICE_ALL_STRETCH => Some(SliceFieldEdit::AllRegions(MODE_STRETCH)),
                _ => None,
            })
            .or_else(|| {
                // A grelha 3×3: cicla o modo desta região a partir do que a ENTIDADE tem.
                ids::INSP_SLICE_REGION
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| {
                        SliceFieldEdit::RegionMode(i as u8, next_region_mode(i, info.tile_modes[i]))
                    })
            });
        if let Some(edit) = edit {
            host.bus_mut().push(EditorAction::InspectorSliceEdit {
                entity_bits: info.entity_bits,
                edit,
            });
            demote_button(host, id);
            return true;
        }
    }

    // Fill Center.
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_SLICE_FILL_CENTER
        && let Some(info) = state::current_inspector_slice()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        host.bus_mut().push(EditorAction::InspectorSliceEdit {
            entity_bits: info.entity_bits,
            edit: SliceFieldEdit::FillCenter(checked),
        });
        return true;
    }

    // Bordas / tamanho / stretch.
    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(info) = state::current_inspector_slice()
    {
        // ⚠️ Cada borda despacha SÓ o seu índice. Mandar o array inteiro atropelaria, num
        // fan-out de seleção múltipla, as bordas divergentes de todas as outras sprites — a lei
        // que o `PerCornerTintAt` e o `RegionX/Y/W/H` já pagaram.
        if let Some(i) = ids::INSP_SLICE_BORDER.iter().position(|&o| o == id) {
            let v = host
                .store()
                .number_value(id)
                .unwrap_or(f64::from(info.borders[i])) as f32;
            host.bus_mut().push(EditorAction::InspectorSliceEdit {
                entity_bits: info.entity_bits,
                edit: SliceFieldEdit::Border(i as u8, v),
            });
            return true;
        }
        if let Some(i) = ids::INSP_SLICE_SIZE.iter().position(|&o| o == id) {
            let v = host
                .store()
                .number_value(id)
                .unwrap_or(f64::from(info.size[i])) as f32;
            let edit = if i == 0 {
                SliceFieldEdit::SizeX(v)
            } else {
                SliceFieldEdit::SizeY(v)
            };
            host.bus_mut().push(EditorAction::InspectorSliceEdit {
                entity_bits: info.entity_bits,
                edit,
            });
            return true;
        }
    }
    false
}

/// Repõe o visual de um botão momentâneo em `Normal` — senão ele fica `Pressed` depois do clique.
fn demote_button(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
