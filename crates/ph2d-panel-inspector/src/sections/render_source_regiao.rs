//! ⭐⭐⭐ **A AMOSTRAGEM DE REGIÃO** — o sub-rectângulo do asset que a sprite mostra.
//!
//! ⚠️ **O corte foi NOMEADO pelo próprio ficheiro-pai antes de ser feito** (o doc do
//! `paint_region_rows` dizia-o por escrito): *«é o maior bloco da secção e o único que fala um
//! vocabulário próprio — um sub-rectângulo do asset —, enquanto o resto descreve a PROVENIÊNCIA»*.
//!
//! ⛔ O tecto de LOC cura-se por CORTE POR RESPONSABILIDADE, nunca por uma entrada no
//! `FILE_OVERAGE_OK` (`CLAUDE.md` §5.0).

use super::*;
use ph2d_i18n::tr;

/// **Uma célula X/Y/W/H da região** — rótulo do eixo + o campo numérico.
///
/// ⚠️ Era uma closure DENTRO do `paint_render_source_section`, e sair paga o tecto de 200 LOC que
/// a wave do hover fez a função cruzar. É também o *"per-row split"* que a tolerância dela nomeia
/// como diferido desde 2026-07-10 — feito para UMA row.
#[allow(clippy::too_many_arguments)]
fn paint_region_num_cell(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    cell: Rect,
    axis: ph2d_i18n::TextKey,
    id: NodeId,
    label_font: f32,
    theme: Theme,
) {
    // ⚠️ Lido AQUI e não recebido: é um TOKEN, e o chamador não tem opinião sobre ele.
    let axis_w = Spacing::Lg.px(); // mini X/Y/W/H label column
    paint_text(
        text_system,
        scene,
        axis.tr(),
        cell.x,
        cell.y + (cell.h - label_font) * 0.5,
        label_font,
        axis_w,
        resolve(ColorToken::Text2, theme),
    );
    let input_rect = Rect::new(cell.x + axis_w, cell.y, (cell.w - axis_w).max(0.0), cell.h);
    hit_index.register(id, input_rect);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value)
        .step(1.0)
        .visual((state, store.hover_live(id)));
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        caret,
        anchor,
        input_rect,
        scene,
        text_system,
        theme,
    );
}

/// **A amostragem de REGIÃO** (spec §3.3) — o toggle + os quatro campos px + o Filter Clip.
///
/// Saiu do corpo de [`paint_render_source_section`] pelo cap de fn do painel, e é o *per-row
/// split* que a nota do allowlist prescreve: é o maior bloco da seção e o único que fala um
/// vocabulário próprio (um sub-retângulo do asset), enquanto o resto descreve a PROVENIÊNCIA.
///
/// ⚠️ Escondido para `HandPacked` — aquele traz o próprio rect do asset, então o controle aqui
/// seria um knob que o extract ignora.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_region_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorSpriteInfo,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    row_gap: f32,
) -> f32 {
    let mut cur_y = y;
    // Region sampling (spec §3.3) — hidden for Hand-packed (it brings its
    // own rect from the asset). Toggle + (when on) X/Y/W/H px inputs +
    // Filter Clip. Renders via the extract `region_subrect` (W2.T2.4).
    if !matches!(info.source_kind, InspectorSpriteSource::HandPacked { .. }) {
        // ⭐⭐ **As duas caixas desta sub-secção partilham UMA coluna** (2026-09-15), medida sobre
        //    os dois nomes — e a segunda só é pintada com a primeira ligada, logo a coluna tem de
        //    contar as duas ou ela salta no clique. Ver [`Seccao::medida`].
        let sec = ph2d_editor_core::property_row::Seccao::medida(
            text_system,
            1,
            &[
                tr("panel.inspector.render_source.region"),
                tr("panel.inspector.render_source.filter_clip"),
            ],
        );
        let re_value = store
            .checkbox(ids::INSP_REGION_ENABLED)
            .map_or(CheckboxValue::Unchecked, |(_, v)| v);
        cur_y = ph2d_editor_core::property_row::paint_check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            (
                ids::INSP_REGION_ENABLED,
                tr("panel.inspector.render_source.region"),
                matches!(re_value, CheckboxValue::Checked),
            ),
            sec,
        );

        if matches!(re_value, CheckboxValue::Checked) {
            let field_h = ROW_H_PX;
            let cell_gap = Spacing::Md.px();
            let (region_w, region_dot) =
                ph2d_editor_core::widget::form_row_columns(x, w, cur_y, field_h);
            let cell_w = ((region_w - cell_gap) * 0.5).max(0.0);
            paint_region_num_cell(
                scene,
                text_system,
                hit_index,
                store,
                Rect::new(x, cur_y, cell_w, field_h),
                ph2d_i18n::TextKey::new("panel.inspector.region.x"),
                ids::INSP_REGION_X,
                label_font,
                theme,
            );
            paint_region_num_cell(
                scene,
                text_system,
                hit_index,
                store,
                Rect::new(x + cell_w + cell_gap, cur_y, cell_w, field_h),
                ph2d_i18n::TextKey::new("panel.inspector.region.y"),
                ids::INSP_REGION_Y,
                label_font,
                theme,
            );
            ph2d_editor_core::widget::paint_decorator_dot(scene, theme, region_dot);
            cur_y += field_h + row_gap;
            paint_region_num_cell(
                scene,
                text_system,
                hit_index,
                store,
                Rect::new(x, cur_y, cell_w, field_h),
                ph2d_i18n::TextKey::new("panel.inspector.region.w"),
                ids::INSP_REGION_W,
                label_font,
                theme,
            );
            paint_region_num_cell(
                scene,
                text_system,
                hit_index,
                store,
                Rect::new(x + cell_w + cell_gap, cur_y, cell_w, field_h),
                ph2d_i18n::TextKey::new("panel.inspector.region.h"),
                ids::INSP_REGION_H,
                label_font,
                theme,
            );
            cur_y += field_h + row_gap;

            let fc_value = store
                .checkbox(ids::INSP_REGION_FILTER_CLIP)
                .map_or(CheckboxValue::Checked, |(_, v)| v);
            cur_y = ph2d_editor_core::property_row::paint_check_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                (
                    ids::INSP_REGION_FILTER_CLIP,
                    tr("panel.inspector.render_source.filter_clip"),
                    matches!(fc_value, CheckboxValue::Checked),
                ),
                sec,
            );
        }
    }
    cur_y
}
