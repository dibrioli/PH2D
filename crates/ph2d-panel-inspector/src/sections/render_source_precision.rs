//! **O par `Format` da §3 Render Source** — os dois botões de precisão e a linha de consequência.
//!
//! ⚠️ **Irmão de [`super::render_source`] por CAP de LOC** (600): a linha de consequência ganhou
//! o caso hand-packed e a Strategy ganhou o ramo read-only da textura cozida (auditoria
//! `docs/Sprite_projeto/20` §4.2/§4.6), levando o ficheiro a 602. *Cortar para o irmão é a cura.*
//!
//! O corte é por responsabilidade: este par é a única coisa da seção que fala de **bytes por
//! pixel**; tudo o resto ali fala de **onde os pixels vivem** (Strategy/Storage) ou de **que
//! pedaço deles se amostra** (Region).

use super::*;
use ph2d_editor_core::screens::hero::{InspectorSpriteInfo, InspectorSpriteSource};

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_precision_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorSpriteInfo,
    x: f32,
    w: f32,
    mut cur_y: f32,
    label_font: f32,
) -> f32 {
    paint_text(
        text_system,
        scene,
        "Format",
        x,
        cur_y,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    cur_y += label_font + SECTION_LABEL_TO_CONTROL_PX;
    if matches!(info.source_kind, InspectorSpriteSource::CookedTexture) {
        paint_text(
            text_system,
            scene,
            info.source_kind.pixel_format(info.source_precision),
            x,
            cur_y,
            label_font,
            w,
            resolve(ColorToken::Text1, theme),
        );
        return cur_y
            + label_font
            + ph2d_editor_core::widget::panel_chrome::SECTION_INNER_ROW_GAP_PX;
    }
    let h = paint_segmented_group_adaptive(
        Rect::new(x, cur_y, w, ROW_H_PX),
        &[
            (
                "RGBA8",
                info.source_precision == Some(ph2d_editor_core::Precision::Rgba8),
                ids::INSP_RENDER_FORMAT_RGBA8,
            ),
            (
                "RGBA16",
                info.source_precision == Some(ph2d_editor_core::Precision::Rgba16),
                ids::INSP_RENDER_FORMAT_RGBA16,
            ),
        ],
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    cur_y += h + SECTION_LABEL_TO_CONTROL_PX;
    // A consequência, escrita ANTES do clique. Só aparece quando há algo a avisar — um sprite que
    // já é de 16 bits não precisa de ler o preço outra vez.
    if info.source_precision == Some(ph2d_editor_core::Precision::Rgba8) {
        // ⚠️ **A consequência de SAIR DA FOLHA estava em falta** (auditoria
        // `docs/Sprite_projeto/20` §4.2). Numa peça hand-packed a conversão faz
        // `drop_sheet_authorship` — a sprite deixa de ser uma peça — e nem esta linha nem o toast
        // («Format · RGBA16») o diziam, enquanto a linha Storage uma acima continuava a ler
        // *«Hand-packed · folha · região»*. O doc desta função argumenta exatamente que *uma
        // consequência que só aparece depois do clique lê-se como bug*; faltava aplicá-lo aqui.
        let consequence = if matches!(info.source_kind, InspectorSpriteSource::HandPacked { .. }) {
            "RGBA16 doubles memory, forces Individual, and leaves the sheet"
        } else {
            "RGBA16 doubles memory and forces Individual"
        };
        paint_text(
            text_system,
            scene,
            consequence,
            x,
            cur_y,
            label_font,
            w,
            resolve(ColorToken::Text2, theme),
        );
        cur_y += label_font;
    }
    cur_y + ph2d_editor_core::widget::panel_chrome::SECTION_INNER_ROW_GAP_PX
}
