//! ⭐ **A linha *Image*** ([plano GEF 12](../../../../docs/GameEngineFuncional/12_imagem_como_asset_plano.md)) — a
//! ranhura da imagem da sprite, logo abaixo de *Visible* e fora de toda secção que fecha (o campo *Sprite* do
//! *Sprite Renderer* da Unity é o primeiro). Clicar abre o seletor; largar uma imagem do navegador escolhe-a.

use super::*;
use ph2d_i18n::tr;

/// Pinta a linha se a selecção é uma sprite; devolve o `y` de baixo (o mesmo se não é).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_image_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let Some(info) = crate::state::current_inspector_sprite() else {
        return y;
    };
    let sec = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("panel.inspector.image")],
    );
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        tr("panel.inspector.image"),
        sec,
    );
    let slot = linha.control;
    let slot_radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, slot, slot_radius, resolve(ColorToken::Bg2, theme));
    ph2d_editor_core::paint::stroke_frame(
        scene,
        slot,
        slot_radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        SLOT_BORDER_PX,
        resolve(ColorToken::Border, theme),
    );
    if !crate::slot_image::paint(scene, text_system, theme, slot, info.entity_bits) {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.image.none"),
            slot.x + Spacing::Xs.px(),
            slot.y + (slot.h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            (slot.w - Spacing::Xs.px() * 2.0).max(0.0),
            resolve(ColorToken::Text2, theme),
        );
    }
    // ⭐ O ALVO: o clique abre o seletor e a queda de uma imagem escolhe-a (`populate.rs`).
    hit_index.register(ids::INSP_RENDER_TEXTURE_SLOT, slot);
    y + ROW_H_PX + ph2d_tokens::control_gap_px()
}
