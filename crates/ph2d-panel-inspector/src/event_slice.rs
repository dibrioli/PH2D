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

/// Despacha um evento da §5. `true` = consumido.
pub(crate) fn apply_slice_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // Anexar / retirar existem SEM snapshot de componente — são o que o cria.
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_slice()
        && (id == ids::INSP_SLICE_ADD || id == ids::INSP_SLICE_REMOVE)
    {
        let edit = if id == ids::INSP_SLICE_ADD {
            SliceFieldEdit::Attach
        } else {
            SliceFieldEdit::Detach
        };
        host.bus_mut().push(EditorAction::InspectorSliceEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        demote_button(host, id);
        return true;
    }

    // Os dois segmentados: a POSIÇÃO no array É a tag.
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_slice()
    {
        let edit = ids::INSP_SLICE_MODE
            .iter()
            .position(|&o| o == id)
            .map(|i| SliceFieldEdit::DrawMode(i as u8))
            .or_else(|| {
                ids::INSP_SLICE_TILE_MODE
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| SliceFieldEdit::TileMode(i as u8))
            })
            .or_else(|| {
                // ⚠️ **O MIOLO cicla só três** — Stretch → Repeat → Mirror. `Blank` é o que o
                // `Fill Center` já exprime, e oferecê-lo aqui daria duas portas para o mesmo
                // estado.
                (id == ids::INSP_SLICE_CENTRE).then(|| {
                    let next = (info.centre_tile_mode + 1) % CENTRE_MODE_COUNT;
                    SliceFieldEdit::CentreMode(next)
                })
            })
            .or_else(|| {
                // A grelha 3×3: cicla o modo desta região a partir do que a ENTIDADE tem.
                ids::INSP_SLICE_REGION
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| {
                        let cur = info.tile_modes[i];
                        let next = (cur + 1) % REGION_MODE_COUNT;
                        SliceFieldEdit::RegionMode(i as u8, next)
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
        if id == ids::INSP_SLICE_STRETCH {
            let v = host
                .store()
                .slider(id)
                .map(|(_, v)| v)
                .unwrap_or(info.stretch_value);
            host.bus_mut().push(EditorAction::InspectorSliceEdit {
                entity_bits: info.entity_bits,
                edit: SliceFieldEdit::StretchValue(v),
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
