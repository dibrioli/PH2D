//! Showcase `paint_actions_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_actions_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_ACTIONS,
        "Actions",
        4,
    );
    if !open {
        return y;
    }
    paint_actions_body(scene, text_system, theme, hit_index, store, x, w, y)
}

/// O CORPO da secção «Actions», sem o cabeçalho, a partir de `y`; devolve o fundo. O cartão
/// da galeria chama-o (`SHOWCASE_CARDS`, doc 08 §1.2); a janela flutuante, pelo `*_section`.
#[allow(clippy::too_many_arguments)]
pub fn paint_actions_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let btn_h = 30.0_f32; // LITERAL-PX-OK: action button height (distinct from ROW_H_PX)
    let gap = Spacing::Xs.px();
    // Three labelled buttons — as many per row as give each the box its word asks for
    // (`super::encaixe`): three in the floating window, fewer in the narrow side column.
    let trio = [
        (ids::INSP_SAMPLE_BTN_PRIMARY, "Save", ButtonKind::Accent),
        (
            ids::INSP_SAMPLE_BTN_SECONDARY,
            "Cancel",
            ButtonKind::Default,
        ),
        (ids::INSP_SAMPLE_BTN_DANGER, "Delete", ButtonKind::Danger),
    ];
    let precisa = trio.iter().fold(0.0_f32, |m, (id, label, kind)| {
        let font = Button::new(*id, *label).kind(*kind).font_size();
        m.max(crate::paint::rect_for_label(super::encaixe::largura(
            text_system,
            label,
            font,
        )))
    });
    let per_row = super::encaixe::pecas_por_fileira(w, gap, precisa, trio.len());
    let cell_w = (w - gap * (per_row - 1) as f32) / per_row as f32;
    for (i, (id, label, kind)) in trio.iter().enumerate() {
        let (row, col) = (i / per_row, i % per_row);
        let r = Rect::new(
            x + (cell_w + gap) * col as f32,
            y + (btn_h + row_gap()) * row as f32,
            cell_w,
            btn_h,
        );
        hit_index.register(*id, r);
        let state = store.button_visual(*id);
        let btn = Button::new(*id, *label).kind(*kind).visual(state);
        paint_button(&btn, r, scene, text_system, theme);
    }
    let rows = trio.len().div_ceil(per_row);
    y += (btn_h + row_gap()) * rows as f32;

    // Icon button + Tag (removable) on one row.
    let icon_size = Spacing::Xl3.px();
    let ir = Rect::new(x, y, icon_size, icon_size);
    hit_index.register(ids::INSP_SAMPLE_BTN_ICON, ir);
    let icon_state = store.button_visual(ids::INSP_SAMPLE_BTN_ICON);
    let icon_btn = Button::new(ids::INSP_SAMPLE_BTN_ICON, "")
        .kind(ButtonKind::IconOnly {
            icon: IconId::Settings,
        })
        .visual(icon_state);
    paint_button(&icon_btn, ir, scene, text_system, theme);

    let tag_h = Density::Compact.row_h_px();
    let tag_state = match store.get(ids::INSP_SAMPLE_TAG_REMOVE) {
        Some(InteractiveState::Tag { state }) => *state,
        _ => TagState::Normal,
    };
    let tag = Tag::new(ids::INSP_SAMPLE_TAG_REMOVE, "filter")
        .tone(TagTone::Accent)
        .removable(true)
        .state(tag_state);
    // ⛔ Era o literal `80,0`. Uma vitrina que reserva um número redondo demonstra a pílula do
    //    dia em que a palavra couber nele — e `filter` já não cabia.
    let tag_w = tag.natural_width(text_system, tag_h);
    let tr = Rect::new(
        x + icon_size + Spacing::Md.px(),
        y + (icon_size - tag_h) * 0.5,
        tag_w,
        tag_h,
    );
    paint_tag(&tag, tr, scene, text_system, theme);
    if let Some(close_r) = tag.close_rect(tr) {
        hit_index.register(ids::INSP_SAMPLE_TAG_REMOVE, close_r);
    }
    y + icon_size
}
