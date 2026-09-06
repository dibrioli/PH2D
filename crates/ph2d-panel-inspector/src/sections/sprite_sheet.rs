//! Sprite Sheet (+ origin + flip) — Inspector section painter (split from sections.rs,
//! architecture_panel_loc_cap). Logic verbatim; behavior unchanged.

use super::*;
use ph2d_editor_core::widget::SectionFold;

/// W2 Sprite Inspector v2 — Sprite Sheet section (anatomia §03 §3.4).
/// HFrames / VFrames / Frame integer NumberInputs. Renders today: the
/// extract slices the atlas rect into the grid and selects `frame`'s
/// cell (clamps `hframes`/`vframes >= 1`, `frame < cells` at commit).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sprite_sheet_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &ph2d_editor_core::screens::hero::InspectorSpriteInfo,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let field_h = ROW_H_PX;
    let row_gap = Spacing::Xs.px();
    let label_color = resolve(ColorToken::Text2, theme);
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let color_id = ids::INSP_LIVE_SHEET_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default for unconfigured section accent
    let header = section_header(store, ids::INSP_LIVE_SHEET_SECTION, "Sprite Sheet").color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    // ⚠️ **A DOBRA do corpo** — o escopo recorta a cena E o hit, e escala o `y` de saída, para
    //    que tudo o que está por baixo suba junto. Ver `SectionFold`.
    // ⚠️ **Pergunta o `t`, e NUNCA o `is_collapsed`:** ao clicar para fechar o flag semântico vira
    //    neste mesmo quadro enquanto o `t` ainda desce, então um corpo gateado no flag sumiria de
    //    repente por baixo de um chevron a rodar — as duas metades a discordar outra vez.
    let Some(fold) = SectionFold::begin(
        store,
        ids::INSP_LIVE_SHEET_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;

    let label_col_w = 78.0_f32; // LITERAL-PX-OK: row-label column width
    let col_gap = Spacing::Md.px();
    let field_x = x + label_col_w + col_gap;
    // A coluna de animação sai da largura ANTES de o campo a repartir — pela porta, e uma vez para
    // todas as linhas desta secção.
    let (control_w, _) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H_PX);
    let field_w = (control_w - label_col_w - col_gap).max(0.0);
    let number_row = |scene: &mut VectorScene,
                      text_system: &mut TextSystem,
                      hit_index: &mut HitIndex,
                      row_y: f32,
                      label: &str,
                      id: NodeId| {
        paint_text(
            text_system,
            scene,
            label,
            x,
            row_y + (field_h - label_font) * 0.5,
            label_font,
            label_col_w,
            label_color,
        );
        let rect = Rect::new(field_x, row_y, field_w, field_h);
        hit_index.register(id, rect);
        let (state, value, buffer, caret, anchor) = read_number_input(store, id);
        let input = NumberInput::new(id, "", value)
            .step(1.0)
            .visual((state, store.hover_live(id)));
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            rect,
            scene,
            text_system,
            theme,
        );
        let (_, dot) = ph2d_editor_core::widget::form_row_columns(x, w, row_y, field_h);
        ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    };
    // Origin controls (spec §3.4) — Centered toggle (quad center vs
    // texture top-left + offset) + Offset X/Y (intrinsic px). Render via
    // Sprite::resolve_anchor (no atlas-UV change — they move the quad).
    let cb_h = 18.0_f32; // LITERAL-PX-OK: Checkbox visual height
    let (_, ce_value) = store
        .checkbox(ids::INSP_SPRITE_CENTERED)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Checked));
    let ce_rect = Rect::new(x, cur_y, w, cb_h);
    hit_index.register(ids::INSP_SPRITE_CENTERED, ce_rect);
    paint_checkbox(
        &Checkbox::new(ids::INSP_SPRITE_CENTERED, "Centered")
            .visual(store.checkbox_visual(ids::INSP_SPRITE_CENTERED))
            .value(ce_value),
        ce_rect,
        scene,
        text_system,
        theme,
    );
    cur_y += cb_h + row_gap;
    number_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Offset X",
        ids::INSP_SPRITE_OFFSET_X,
    );
    cur_y += field_h + row_gap;
    number_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Offset Y",
        ids::INSP_SPRITE_OFFSET_Y,
    );
    cur_y += field_h + row_gap;
    // Logical Flip H / Flip V (Sprite.flip_x/flip_y) — spec §3.4 orders
    // flip with the origin controls (after Offset, before the frame
    // grid). Two checkboxes side by side; toggling dispatches an
    // InspectorSpriteEdit and the shader mirrors the sampled UV.
    let flip_row_h = 18.0_f32; // LITERAL-PX-OK: matches Checkbox visual height
    let flip_gap = Spacing::Md.px();
    let flip_half = ((w - flip_gap) * 0.5).max(0.0);
    let (_, fx_value) = store
        .checkbox(ids::INSP_SPRITE_FLIP_X)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let fx_rect = Rect::new(x, cur_y, flip_half, flip_row_h);
    hit_index.register(ids::INSP_SPRITE_FLIP_X, fx_rect);
    paint_checkbox(
        &Checkbox::new(ids::INSP_SPRITE_FLIP_X, "Flip H")
            .visual(store.checkbox_visual(ids::INSP_SPRITE_FLIP_X))
            .value(fx_value),
        fx_rect,
        scene,
        text_system,
        theme,
    );
    let (_, fy_value) = store
        .checkbox(ids::INSP_SPRITE_FLIP_Y)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let fy_rect = Rect::new(x + flip_half + flip_gap, cur_y, flip_half, flip_row_h);
    hit_index.register(ids::INSP_SPRITE_FLIP_Y, fy_rect);
    paint_checkbox(
        &Checkbox::new(ids::INSP_SPRITE_FLIP_Y, "Flip V")
            .visual(store.checkbox_visual(ids::INSP_SPRITE_FLIP_Y))
            .value(fy_value),
        fy_rect,
        scene,
        text_system,
        theme,
    );
    cur_y += flip_row_h + row_gap;

    number_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "H Frames",
        ids::INSP_SPRITE_HFRAMES,
    );
    cur_y += field_h + row_gap;
    number_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "V Frames",
        ids::INSP_SPRITE_VFRAMES,
    );
    cur_y += field_h + row_gap;
    number_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Frame",
        ids::INSP_SPRITE_FRAME,
    );
    cur_y += field_h + row_gap;

    cur_y = sheet_preview_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        cb_h,
        info,
    );
    cur_y += SECTION_BOTTOM_PAD_PX;

    fold.finish(store, scene, hit_index, cur_y)
}

/// **«Show sheet on canvas»** — a grelha desdobra-se em células fantasma à volta da viva
/// (Enio, 2026-08-23: *«você digita 8 quadros e não vê onde eles começam ou terminam»*).
///
/// ⚠️ **Função irmã por CAP** (200): com este bloco inline a `paint_sprite_sheet_section` media
/// **209**. *A cura de um teto estourado é o corte, nunca uma isenção.*
///
/// ⚠️ **Só quando HÁ grelha.** Numa sprite `1×1` a caixa não teria o que desdobrar, e um
/// interruptor que não faz nada ensina a desconfiar dos outros — a mesma lei do `+ Add Animator`
/// da §11: a face sem estado mostra o que se **pode** fazer.
///
/// ⚠️ **A contagem vem do SNAPSHOT, e não dos campos do store** — a lei que este módulo pagou no
/// mesmo dia na caixa «Playing»: quem *decide* lê a mesma fonte que quem *consome*. O canvas abre
/// a folha a partir do `Sprite` do MUNDO; decidir pelo número que está no campo faria a caixa
/// existir a meio de uma edição — antes de o mundo ter grelha — e ligá-la não mostraria nada.
#[allow(clippy::too_many_arguments)]
fn sheet_preview_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    cb_h: f32,
    info: &ph2d_editor_core::screens::hero::InspectorSpriteInfo,
) -> f32 {
    let cells = u64::from(info.hframes.max(1)) * u64::from(info.vframes.max(1));
    if cells <= 1 {
        return y;
    }
    let (_, value) = store
        .checkbox(ids::INSP_SHEET_PREVIEW)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let rect = Rect::new(x, y, w, cb_h);
    hit_index.register(ids::INSP_SHEET_PREVIEW, rect);
    paint_checkbox(
        &Checkbox::new(ids::INSP_SHEET_PREVIEW, "Show sheet on canvas")
            .visual(store.checkbox_visual(ids::INSP_SHEET_PREVIEW))
            .value(value),
        rect,
        scene,
        text_system,
        theme,
    );
    y + cb_h
}
