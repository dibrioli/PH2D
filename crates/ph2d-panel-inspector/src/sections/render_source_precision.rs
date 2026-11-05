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
use ph2d_i18n::tr;

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
    if matches!(info.source_kind, InspectorSpriteSource::CookedTexture) {
        // ⚠️ Uma textura cozida não tem escolha: o nome fica por cima de um FACTO, não de um
        //    controlo (a irmã `Strategy` faz o mesmo).
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.render_source.format"),
            x,
            cur_y,
            label_font,
            w,
            resolve(ColorToken::Text2, theme),
        );
        cur_y += label_font + SECTION_LABEL_TO_CONTROL_PX;
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
    // ⭐⭐ **Pela porta da ESCOLHA, com o nome ao LADO** (2026-09-23) — ver a irmã `Strategy`.
    let seccao = super::render_source::seccao_do_render(text_system);
    cur_y = ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.render_source.format"),
        &[
            (
                tr("panel.inspector.render_source.rgba8"),
                info.source_precision == Some(ph2d_editor_core::Precision::Rgba8),
                ids::INSP_RENDER_FORMAT_RGBA8,
            ),
            (
                tr("panel.inspector.render_source.rgba16"),
                info.source_precision == Some(ph2d_editor_core::Precision::Rgba16),
                ids::INSP_RENDER_FORMAT_RGBA16,
            ),
        ],
        seccao,
    );
    // Num cartão a frase vem depois da fila do Format e do Emissive, que dividem a linha (05 §4, lei 1).
    if ph2d_editor_core::widget::card_ink::card_body().is_none() {
        cur_y = paint_precision_price(scene, text_system, theme, info, x, w, cur_y);
    }
    cur_y + ph2d_editor_core::widget::panel_chrome::SECTION_INNER_ROW_GAP_PX
}

/// A consequência, escrita ANTES do clique. Só aparece quando há algo a avisar: um sprite que já é de
/// 16 bits não precisa de ler o preço outra vez. A frase quebra: cortada a meio não diz o preço (foto a2).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_precision_price(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    info: &InspectorSpriteInfo,
    x: f32,
    w: f32,
    cur_y: f32,
) -> f32 {
    if info.source_precision != Some(ph2d_editor_core::Precision::Rgba8)
        || matches!(info.source_kind, InspectorSpriteSource::CookedTexture)
    {
        return cur_y;
    }
    // ⚠️ Sair da folha também custa: numa peça hand-packed a conversão faz `drop_sheet_authorship`,
    // e a linha Storage uma acima ainda lê «Hand-packed · folha · região» (`docs/Sprite_projeto/20` §4.2).
    let consequence = if matches!(info.source_kind, InspectorSpriteSource::HandPacked { .. }) {
        tr("panel.inspector.render_source.rgba16_doubles_memory_forces_individual")
    } else {
        tr("panel.inspector.render_source.rgba16_doubles_memory_and_forces")
    };
    super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        consequence,
        ColorToken::Text2,
    )
}
