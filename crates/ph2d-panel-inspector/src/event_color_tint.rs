//! **Os três cliques de COR da sprite** — irmão do [`super::event`] por CAP de LOC.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE:** o `event.rs` bateu `605` de `600` ao ganhar os dois braços
//! do ABANÃO (suplente #25), e este bloco é **um assunto fechado** — o Tint / Self Tint, um canto do
//! Per-Corner e o *Equalize Corners*, todos a semear o MESMO selector de cor. ⛔ **Curado por CORTE,
//! nunca por uma entrada no `FILE_OVERAGE_OK`** — aquela lista está VAZIA.
//!
//! ⚠️ Ele saiu **verbatim e na MESMA posição da escada** do despacho: os três corriam logo a seguir
//! à tabela `SINGLE_ID_CLICKS`, e continuam a correr — *um corte que muda a ordem do despacho não é
//! um corte, é uma mudança de comportamento com cara de arrumação.*

use crate::{ids, state};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::SpriteFieldEdit;
use ph2d_editor_core::widget::ButtonState;

/// **Os três cliques de COR da sprite** — o Tint / Self Tint, um canto do Per-Corner e o *Equalize
/// Corners* —, cada um a responder *«era eu?»* como os da tabela [`SINGLE_ID_CLICKS`].
///
/// ⚠️ Saiu do [`apply_event_impl`] pelo tecto de 200 LOC por função, verbatim e na MESMA posição da
/// escada: os três corriam logo a seguir à tabela, e continuam a correr.
pub(crate) fn color_tint_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // W2 Color & Tint — Tint / Self Tint swatch click opens the shared
    // BlenderColorPicker (OKLCH) seeded from the sprite's CURRENT channel
    // (not the generic per-widget accent the section dot uses). The
    // chosen color round-trips via `widget_color(<swatch>)` — mirrored
    // each frame from the picker in `hero.rs` — and `sync.rs` dispatches
    // it as `SpriteFieldEdit::Tint` / `SelfTint` while the picker targets
    // this swatch.
    if let WidgetEvent::Click(id) = ev
        && matches!(
            id,
            ids::INSP_SPRITE_TINT_SWATCH | ids::INSP_SPRITE_SELF_TINT_SWATCH
        )
        && let Some(info) = state::current_inspector_sprite()
    {
        let chan = if id == ids::INSP_SPRITE_TINT_SWATCH {
            info.tint
        } else {
            info.self_tint
        };
        let seed = crate::state_tint::tint_f32_to_u8(chan);
        host.store_mut().set_widget_color(id, seed);
        host.store_mut().set_picker_target(Some(id));
        host.store_mut().set_blender_value(
            core_ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return true;
    }

    // (Color & Tint sub-tabs retired 2026-05-31 — the section stacks all
    // controls visible at once, so there's no tab selection to pin.)

    // W2 Color & Tint — per-corner swatch click opens the picker seeded
    // from the sprite's CURRENT corner color (TL=0, TR=1, BL=2, BR=3).
    // `sync.rs` replaces that one corner of the [[f32;4];4] array and
    // dispatches the whole `SpriteFieldEdit::PerCornerTint`.
    if let WidgetEvent::Click(id) = ev
        && let Some(corner) = match id {
            ids::INSP_SPRITE_CORNER_TL => Some(0usize),
            ids::INSP_SPRITE_CORNER_TR => Some(1),
            ids::INSP_SPRITE_CORNER_BL => Some(2),
            ids::INSP_SPRITE_CORNER_BR => Some(3),
            _ => None,
        }
        && let Some(info) = state::current_inspector_sprite()
    {
        let seed = crate::state_tint::tint_f32_to_u8(info.per_corner_tint[corner]);
        host.store_mut().set_widget_color(id, seed);
        host.store_mut().set_picker_target(Some(id));
        host.store_mut().set_blender_value(
            core_ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return true;
    }

    // W2 Color & Tint — "Equalize Corners" copies the top-left corner to
    // the other three (spec §3.6), dispatched as one PerCornerTint edit.
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_SPRITE_CORNER_EQUALIZE
        && let Some(info) = state::current_inspector_sprite()
    {
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit: SpriteFieldEdit::EqualizeCorners,
        });
        // Momentary button — demote the visual back to Normal so it
        // doesn't stick Pressed after the click.
        if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
            *state = ButtonState::Normal;
        }
        return true;
    }
    false
}
