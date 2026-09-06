//! **§12 Sockets / Named Anchors** ([ADR-0072], spec
//! [`07_named_anchors.md`](../../../../docs/Sprite_projeto/07_named_anchors.md) §7.5).
//!
//! O ADR está `Accepted` desde 2026-05-28 e, até 2026-08-21, `NamedAnchor` tinha **cinco
//! ocorrências no repositório, todas doc-comment ou `#[ignore]`**.
//!
//! # LISTA + EDITOR, e a razão
//!
//! A spec desenha uma **ficha por âncora**, empilhadas. Com o cap de 64 âncoras isso são ~768
//! ids pré-alocados e um painel de 320 px a rolar por 64 fichas abertas. Aqui é uma lista de
//! nomes (um id por linha) mais um editor do que está selecionado.
//!
//! ⚠️ **O que a ficha empilhada mostrava de relance não se perde:** cada linha traz o **tipo
//! derivado** ao lado do nome — `Socket` · `Slice` · `Region` —, que é a informação que a forma
//! da âncora codifica.

use super::*;
use ph2d_editor_core::screens::hero::InspectorAnchorInfo;
use ph2d_editor_core::widget::SectionFold;

const FIELD_H: f32 = 24.0; // LITERAL-PX-OK: altura de campo do Inspector
const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector
const ROW_H: f32 = 22.0; // LITERAL-PX-OK: altura de uma linha da lista
/// Onde o rótulo do TIPO começa, como fração da largura da linha. ⚠️ Fração de layout, não uma
/// medida em pixels: o painel é redimensionável, e uma coluna fixa em px descolaria do nome.
const KIND_COL_FRAC: f32 = 0.62; // LITERAL-PX-OK: fração de largura, não um token de espaçamento
/// Passo de scrub dos campos em pixels da fonte.
const PX_STEP: f64 = 1.0; // LITERAL-PX-OK: passo em pixels da FONTE, não um token de desenho
/// Passo de scrub da rotação, em graus.
const DEG_STEP: f64 = 1.0; // LITERAL-PX-OK: passo em graus

/// Uma linha «rótulo em cima, N NumberInput lado a lado». Devolve o `y` seguinte.
///
/// ⚠️ **`pub(super)` para a §11 a reusar** — duas cópias de um layout de linha divergem no dia em
/// que uma delas ganha um espaçamento novo, e o painel passa a ter duas gramáticas.
#[allow(clippy::too_many_arguments)]
pub(super) fn field_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    field_ids: &[NodeId],
    step: f64,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let row_y = y + label_h;
    let gap = Spacing::Xs.px();
    let n = field_ids.len().max(1) as f32;
    let cw = ((w - gap * (n - 1.0)) / n).max(0.0);
    for (i, &id) in field_ids.iter().enumerate() {
        let rect = Rect::new(x + (cw + gap) * i as f32, row_y, cw, FIELD_H);
        hit_index.register(id, rect);
        let (state, value, buffer, caret, anchor) = read_number_input(store, id);
        let input = NumberInput::new(id, "", value)
            .step(step)
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
    }
    row_y + FIELD_H + Spacing::Sm.px()
}

/// Uma checkbox simples. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
) -> f32 {
    let cb_h = 18.0_f32; // LITERAL-PX-OK: altura visual do Checkbox
    let (_, value) = store
        .checkbox(id)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let rect = Rect::new(x, y, w, cb_h);
    hit_index.register(id, rect);
    paint_checkbox(
        &Checkbox::new(id, label)
            .visual(store.checkbox_visual(id))
            .value(value),
        rect,
        scene,
        text_system,
        theme,
    );
    y + cb_h + Spacing::Sm.px()
}

/// A lista de âncoras. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn anchor_list(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnchorInfo,
    selected: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    // ⚠️ `zip` com o array de ids: se a lista tiver mais âncoras do que ids (impossível
    // enquanto o gate `the_row_ids_cover_the_model_cap` viver), as excedentes ficam **fora** e
    // não são pintadas em cima umas das outras.
    for (i, (row, &id)) in info
        .rows
        .iter()
        .zip(ids::INSP_ANCHOR_ROW.iter())
        .enumerate()
    {
        let rect = Rect::new(x, cur_y, w, ROW_H);
        hit_index.register(id, rect);
        let is_sel = i == selected;
        if is_sel {
            ph2d_editor_core::paint::fill_rounded_rect(
                scene,
                rect,
                Radius::Sm.px(),
                resolve(ColorToken::Bg2, theme),
            );
        }
        let color = if is_sel {
            resolve(ColorToken::Text1, theme)
        } else {
            resolve(ColorToken::Text2, theme)
        };
        paint_text(
            text_system,
            scene,
            &row.name,
            x + Spacing::Sm.px(),
            cur_y + (ROW_H - font) * 0.5,
            font,
            w,
            color,
        );
        // O tipo derivado, à direita — é o que a ficha empilhada da spec mostrava de relance.
        //
        // ⚠️ **Mais quem monta nesta âncora**, desde 2026-08-22. Fecha o laço do outro lado: quem
        // monta vê em que âncora anda, e sem isto o dono da âncora não via ninguém — saber se
        // `hand_r` está em uso obrigava a selecionar cada filho, e apagá-la não avisava de nada.
        let kind = if row.riders == 0 {
            row.kind_label().to_string()
        } else {
            format!("{} \u{b7} {} riding", row.kind_label(), row.riders)
        };
        paint_text(
            text_system,
            scene,
            &kind,
            x + w * KIND_COL_FRAC,
            cur_y + (ROW_H - font) * 0.5,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += ROW_H;
    }
    cur_y + Spacing::Sm.px()
}

/// O editor da âncora selecionada. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn anchor_editor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &ph2d_editor_core::screens::hero::InspectorAnchorRow,
) -> f32 {
    let mut cur_y = y;
    // Nome.
    let (control_w, name_dot) = ph2d_editor_core::widget::form_row_columns(x, w, cur_y, ROW_H_PX);
    let host = Rect::new(x, cur_y, control_w, ROW_H_PX);
    hit_index.register(ids::INSP_ANCHOR_NAME, host);
    let (state, text, caret, sel_anchor) = match store.get(ids::INSP_ANCHOR_NAME) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let input = TextInput::new(ids::INSP_ANCHOR_NAME, "")
        .placeholder("anchor_name\u{2026}")
        .visual((state, store.hover_live(ids::INSP_ANCHOR_NAME)));
    paint_text_input_with_buffer(
        &input,
        text,
        Some(caret),
        sel_anchor,
        host,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, name_dot);
    cur_y += ROW_H_PX + Spacing::Sm.px();

    cur_y = field_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Position X / Y (px)",
        &ids::INSP_ANCHOR_POS,
        PX_STEP,
    );
    cur_y = field_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Rotation (deg)",
        &[ids::INSP_ANCHOR_ROT],
        DEG_STEP,
    );
    cur_y = check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Bounds (makes it a Slice)",
        ids::INSP_ANCHOR_BOUNDS_ON,
    );
    // ⚠️ Os campos da área só existem quando ela existe. Pintá-los sobre um Socket seria
    // oferecer quatro números que não vão a lado nenhum.
    if row.bounds.is_some() {
        cur_y = field_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            "Bounds X / Y / W / H (px)",
            &ids::INSP_ANCHOR_BOUNDS,
            PX_STEP,
        );
        cur_y = check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            "Center (makes it a 9-slice Region)",
            ids::INSP_ANCHOR_CENTER_ON,
        );
        if row.center.is_some() {
            cur_y = field_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                "Center X / Y / W / H (px)",
                &ids::INSP_ANCHOR_CENTER,
                PX_STEP,
            );
        }
    }
    let rm = Rect::new(x, cur_y, w, BTN_H);
    hit_index.register(ids::INSP_ANCHOR_REMOVE, rm);
    paint_button(
        &Button::new(ids::INSP_ANCHOR_REMOVE, "x Remove Anchor")
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_ANCHOR_REMOVE)),
        rm,
        scene,
        text_system,
        theme,
    );
    cur_y + BTN_H
}

/// Pinta a §12 e devolve o `y` a seguir a ela.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_anchors_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnchorInfo,
    selected: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = ids::INSP_LIVE_ANCHOR_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let title = if info.rows.is_empty() {
        String::from("Sockets / Anchors")
    } else {
        format!("Sockets / Anchors  ({})", info.rows.len())
    };
    let header = section_header(store, ids::INSP_LIVE_ANCHOR_SECTION, &title).color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        ids::INSP_LIVE_ANCHOR_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    let font = TypeToken::Sm.px();

    // ⚠️ **A montagem vem PRIMEIRO, e é de outro dono.** Ela diz de que âncora do PAI este objeto
    // parte; a lista abaixo são as âncoras DESTE objeto. Os rótulos dizem qual é qual — a seção
    // sem eles teria duas listas de âncoras e nenhuma pista de quem as possui.
    cur_y = super::anchor_mount_row::paint_mount_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    if info.mount_pick_is_useful() {
        // O rótulo só existe quando há a OUTRA metade de que se distinguir. Sozinho, ele seria
        // ruído: uma seção com uma lista só não precisa de dizer de quem ela é.
        paint_text(
            text_system,
            scene,
            "This object's anchors",
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + Spacing::Xs.px();
    }

    if info.rows.is_empty() {
        paint_text(
            text_system,
            scene,
            "No anchors on this sprite.",
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + Spacing::Sm.px();
    } else {
        cur_y = anchor_list(
            scene,
            text_system,
            theme,
            hit_index,
            x,
            w,
            cur_y,
            info,
            selected,
        );
    }

    // As duas caixas de visibilidade — do DONO das âncoras, logo abaixo da lista dele.
    cur_y = super::anchor_mount_row::paint_visibility_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );

    let add = Rect::new(x, cur_y, w, BTN_H);
    hit_index.register(ids::INSP_ANCHOR_ADD, add);
    paint_button(
        &Button::new(ids::INSP_ANCHOR_ADD, "+ Add Anchor")
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_ANCHOR_ADD)),
        add,
        scene,
        text_system,
        theme,
    );
    cur_y += BTN_H + Spacing::Sm.px();

    if let Some(row) = info.rows.get(selected) {
        cur_y = paint_section_separator(scene, theme, x, w, cur_y);
        cur_y = anchor_editor(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            row,
        );
    }

    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
