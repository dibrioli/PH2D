//! **A REGIÃO e a ORIGEM da sprite** — sub-rect (`region_*`), `Centered` e `Offset X/Y`.
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função.** O `apply_event_impl` tem catraca em 384
//! (`architecture_panel_loc_cap`) e a §5 9-Slice empurrou-o para 389 em 2026-08-21. *A catraca só
//! desce, e um cluster de cada vez* — levar só as cinco linhas novas deixaria o número onde
//! estava, e ficar no mesmo sítio não é encolher.
//!
//! # A lei que junta estes sete braços
//!
//! **Despacho POR EIXO.** `RegionX/Y/W/H` e `OffsetX/Y` existem em vez de um `RegionRect` e um
//! `Offset` inteiros pela mesma razão que o `PerCornerTintAt`: numa seleção múltipla, mandar o
//! vetor todo atropela o eixo divergente de cada uma das outras sprites — enquanto o painel pinta
//! «Mixed» para esse mesmo estado. *A promessa e o verbo discordavam* (auditoria
//! `docs/Sprite_projeto/20` §3.2).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::SpriteFieldEdit;
use ph2d_editor_core::widget::CheckboxValue;

use crate::state;

/// Despacha um evento de região/origem. `true` = consumido.
pub(crate) fn apply_sprite_geometry_event(
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    // W2 Region (spec §3.3) — enable / filter-clip toggles.
    if let WidgetEvent::Toggled(id) = ev
        && matches!(id, ids::INSP_REGION_ENABLED | ids::INSP_REGION_FILTER_CLIP)
        && let Some(info) = state::current_inspector_sprite()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        if id == ids::INSP_REGION_FILTER_CLIP {
            host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                entity_bits: info.entity_bits,
                edit: SpriteFieldEdit::RegionFilterClip(checked),
            });
        } else {
            host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                entity_bits: info.entity_bits,
                edit: SpriteFieldEdit::RegionEnabled(checked),
            });
            // Enabling region on a still-zero rect would make the sprite
            // vanish (zero-area UV = no-op). Seed the rect to the full
            // source (spec §3.3 default `[0, 0, w, h]`) so toggling on is
            // visible and editable. SINGLE-SELECT ONLY: on a multi-select
            // the source size is per-sprite, so seeding the primary's dims
            // onto all would give every other sprite a wrong rect (audit
            // D-3). For a multi-select the user sets the rect explicitly.
            if checked
                && info.selected_count == 1
                && (info.region_rect[2] <= 0.0 || info.region_rect[3] <= 0.0)
                && let Some((sw, sh)) = info.source_pixels
            {
                host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                    entity_bits: info.entity_bits,
                    edit: SpriteFieldEdit::RegionRect([0.0, 0.0, sw as f32, sh as f32]),
                });
            }
        }
        return true;
    }
    // W2 Region — X/Y/W/H px NumberInputs. Each dispatches ONLY its own
    // axis (per-axis SpriteFieldEdit) so a bulk edit of one axis can't
    // re-read + stomp a diverging sibling (audit D-1). W/H floor at 0 at
    // the commit boundary.
    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(axis) = match id {
            ids::INSP_REGION_X => Some(0usize),
            ids::INSP_REGION_Y => Some(1),
            ids::INSP_REGION_W => Some(2),
            ids::INSP_REGION_H => Some(3),
            _ => None,
        }
        && let Some(info) = state::current_inspector_sprite()
    {
        let v = host
            .store()
            .number_value(id)
            .unwrap_or(info.region_rect[axis] as f64) as f32;
        let edit = match axis {
            0 => SpriteFieldEdit::RegionX(v),
            1 => SpriteFieldEdit::RegionY(v),
            2 => SpriteFieldEdit::RegionW(v),
            _ => SpriteFieldEdit::RegionH(v),
        };
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    // W2 origin (spec §3.4) — Centered toggle.
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_SPRITE_CENTERED
        && let Some(info) = state::current_inspector_sprite()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit: SpriteFieldEdit::Centered(checked),
        });
        return true;
    }
    // W2 origin — Offset X/Y px NumberInputs. Per-axis dispatch (not the
    // whole Offset vector) so editing one axis can't stomp a diverging
    // sibling on a multi-selection (audit D-1).
    if let WidgetEvent::ValueChanged(id) = ev
        && matches!(id, ids::INSP_SPRITE_OFFSET_X | ids::INSP_SPRITE_OFFSET_Y)
        && let Some(info) = state::current_inspector_sprite()
    {
        let is_x = id == ids::INSP_SPRITE_OFFSET_X;
        let fallback = if is_x { info.offset[0] } else { info.offset[1] };
        let v = host.store().number_value(id).unwrap_or(fallback as f64) as f32;
        let edit = if is_x {
            SpriteFieldEdit::OffsetX(v)
        } else {
            SpriteFieldEdit::OffsetY(v)
        };
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    false
}
