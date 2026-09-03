//! **UMA LINHA da secção Transform** — rótulo à esquerda, e um ou dois campos numéricos.
//!
//! ⚠️ Este ficheiro nasceu de um **tecto de LOC por função**: a `paint_transform_section` passou a
//! folga dela quando a coluna de animação entrou (2026-09-03). ⛔ A casa não aumenta a folga —
//! *«allowances shrink, they never grow»* —, corta. E o corte é o que a própria mensagem do gate
//! sugere: *o desenho de uma linha* sai da *orquestração das linhas*.
//!
//! ⭐ Os catorze valores que a closure capturava viraram o [`RowStyle`]: eles são **a geometria da
//! secção**, calculada uma vez e igual para as quatro linhas — e enquanto viviam como capturas,
//! nada dizia isso.

use super::*;
use ph2d_editor_core::widget::DECORATOR_W;

/// A geometria partilhada por TODA linha desta secção — calculada uma vez, lida por cada linha.
pub(super) struct RowStyle<'a> {
    pub x: f32,
    /// A largura INTEIRA da secção. A coluna de animação sai daqui.
    pub w: f32,
    /// A largura que sobra para as colunas da linha (`w` menos a coluna de animação).
    pub row_w: f32,
    pub field_h: f32,
    pub label_font: f32,
    pub label_color: ph2d_vector::Color,
    pub theme: Theme,
    pub store: &'a WidgetStore,
    pub section_narrow: bool,
    pub two_chip_w: f32,
    pub col_gap: f32,
    pub axis_col_w: f32,
    pub tag_box_gap: f32,
    pub label_above_gap: f32,
    /// A coluna do rótulo, quando a linha é inline.
    pub label_col_w: f32,
    /// O corpo de letra da etiqueta de eixo (`X` / `Y`).
    pub axis_label_font: f32,
}

/// Pinta uma linha e devolve a altura que ela usou.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_row(
    st: &RowStyle<'_>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    row_y: f32,
    row_label: &str,
    left_id: NodeId,
    left_tag: &str,
    left_color: ColorToken,
    left_step: f64,
    right: Option<(NodeId, &str, ColorToken, f64)>,
) -> f32 {
    let chips_origin_x = if st.section_narrow {
        st.x
    } else {
        st.x + st.label_col_w + st.col_gap
    };
    let label_h_used = if st.section_narrow {
        st.field_h
    } else {
        0.0_f32
    };
    let total_h = if st.section_narrow {
        st.field_h + st.label_above_gap + st.field_h
    } else {
        st.field_h
    };
    let chips_y = row_y
        + label_h_used
        + if st.section_narrow {
            st.label_above_gap
        } else {
            0.0
        };

    // Label — full-width on its own row when narrow; left column when inline.
    paint_text(
        text_system,
        scene,
        row_label,
        st.x,
        row_y + (st.field_h - st.label_font) * 0.5,
        st.label_font,
        if st.section_narrow {
            st.row_w
        } else {
            st.label_col_w
        },
        st.label_color,
    );

    // Single-chip rows (Rotation) span from X-chip start to Y-chip end —
    // alignment with the 2-chip rows above + below. The span must
    // include: chip_X + st.col_gap + Y-tag slot + chip_Y =
    // 2*st.two_chip_w + st.col_gap + st.axis_col_w + st.tag_box_gap.
    // The previous formula missed `st.axis_col_w + st.tag_box_gap` and
    // left Rotation ending short of Y-chip's right edge.
    let single_chip = right.is_none();
    let left_box_w = if single_chip {
        (st.two_chip_w * 2.0 + st.col_gap + st.axis_col_w + st.tag_box_gap).max(0.0)
    } else {
        st.two_chip_w
    };

    let left_tag_x = chips_origin_x;
    paint_text(
        text_system,
        scene,
        left_tag,
        left_tag_x,
        chips_y + (st.field_h - st.axis_label_font) * 0.5,
        st.axis_label_font,
        st.axis_col_w,
        resolve(left_color, st.theme),
    );
    let left_box_x = left_tag_x + st.axis_col_w + st.tag_box_gap;
    let left_rect = Rect::new(left_box_x, chips_y, left_box_w, st.field_h);
    hit_index.register(left_id, left_rect);
    let (state, value, buffer, caret, anchor) = read_number_input(st.store, left_id);
    let input = NumberInput::new(left_id, "", value)
        .step(left_step)
        .visual((state, st.store.hover_live(left_id)));
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        caret,
        anchor,
        left_rect,
        scene,
        text_system,
        st.theme,
    );
    if let Some((right_id, right_tag, right_color, right_step)) = right {
        let right_tag_x = left_box_x + st.two_chip_w + st.col_gap;
        paint_text(
            text_system,
            scene,
            right_tag,
            right_tag_x,
            chips_y + (st.field_h - st.axis_label_font) * 0.5,
            st.axis_label_font,
            st.axis_col_w,
            resolve(right_color, st.theme),
        );
        let right_box_x = right_tag_x + st.axis_col_w + st.tag_box_gap;
        let right_rect = Rect::new(right_box_x, chips_y, st.two_chip_w, st.field_h);
        hit_index.register(right_id, right_rect);
        let (r_state, r_value, r_buffer, r_caret, r_anchor) = read_number_input(st.store, right_id);
        let r_input = NumberInput::new(right_id, "", r_value)
            .step(right_step)
            .visual((r_state, st.store.hover_live(right_id)));
        paint_number_input_with_buffer(
            &r_input,
            Some(r_buffer),
            r_caret,
            r_anchor,
            right_rect,
            scene,
            text_system,
            st.theme,
        );
    }
    // O ponto da coluna de animação — **um por LINHA**, não por campo: a propriedade é a
    // linha, e um par X/Y é uma propriedade com duas componentes.
    ph2d_editor_core::widget::paint_decorator_dot(
        scene,
        st.theme,
        Rect::new(st.x + st.w - DECORATOR_W, chips_y, DECORATOR_W, st.field_h),
    );
    total_h
}
