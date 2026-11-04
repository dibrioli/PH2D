//! Os corpos do painel Painter já em linhas de cartão (`docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, e o
//! valor continua a vir do instantâneo do pincel. O id de cada caixa continua `Button` e emite `Click`.
//! Nenhum destes ids tem linha esmaecida (`Dependente`): o gate `no_native_id_has_a_dimmed_law` avisa quando tiver.

use crate::cards_linhas::marca;
use crate::{number_field, paint_brush, paint_pigment, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, corpo as linhas, ferramenta};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;

type Corpo = fn(&mut CardCtx<'_>, f32) -> f32;

/// O corpo da secção `id`, se ela já foi convertida.
pub(crate) fn corpo(id: NodeId) -> Option<Corpo> {
    let f: Corpo = if id == pids::PAINTER_CARD_MIXING {
        mixing
    } else if id == pids::PAINTER_BRUSH_SYMMETRY_SECTION {
        simetria
    } else if id == pids::PAINTER_BRUSH_TILING_SECTION {
        tiling
    } else {
        return None;
    };
    Some(f)
}

/// A secção convertida está à vista? O título e o Reset dela: as condições de `paint_brush_sections`.
pub(crate) fn presente(
    id: NodeId,
    b: &ph2d_tool_painter::BrushSettings,
) -> Option<(String, Option<NodeId>)> {
    use ph2d_tool_painter::LinhaDaFerramenta as L;
    let fora = !b.is_inpaint;
    if id == pids::PAINTER_CARD_MIXING {
        let gesto = !b.wetpaint || b.le(L::MisturaDoPigmento);
        (fora && b.pigment_offered && gesto)
            .then(|| (tr("panel.painter_layers.brush.mixing").to_owned(), None))
    } else if id == pids::PAINTER_BRUSH_SYMMETRY_SECTION {
        (fora && b.le(L::Simetria)).then(|| {
            (
                tr("panel.painter_layers.symmetry.symmetry").to_owned(),
                Some(pids::PAINTER_BRUSH_SYMMETRY_RESET),
            )
        })
    } else if id == pids::PAINTER_BRUSH_TILING_SECTION {
        fora.then(|| {
            (
                tr("panel.painter_layers.stroke.tiling").to_owned(),
                Some(pids::PAINTER_BRUSH_TILING_RESET),
            )
        })
    } else {
        None
    }
}

fn pincel() -> ph2d_tool_painter::BrushSettings {
    state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH)
}

/// Com o store MUTÁVEL, antes de pintar: o que o corpo do cartão lê e não pode escrever.
pub(crate) fn prepare(store: &mut WidgetStore) {
    let brush = pincel();
    store.mark_card_field(
        pids::PAINTER_WATERCOLOR_MIX,
        CardFieldSpec {
            step: number_field::FINE_STEP,
            scrub: number_field::FINE_STEP,
            default: Some(f64::from(paint_pigment::pigment_amount(
                &paint_brush::FALLBACK_BRUSH,
            ))),
        },
    );
    number_field::arm_field(
        store,
        pids::PAINTER_WATERCOLOR_MIX,
        paint_pigment::pigment_amount(&brush),
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    let segmentos = pids::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP;
    store.mark_card_field(
        segmentos,
        CardFieldSpec {
            step: 1.0,
            scrub: 1.0,
            default: Some(f64::from(paint_brush::FALLBACK_BRUSH.symmetry_segments)),
        },
    );
    if store.number_input_drag().map(|d| d.id) != Some(segmentos) {
        store.set_number_value(segmentos, f64::from(brush.symmetry_segments));
    }
}

fn campo(id: NodeId, rotulo: &str) -> Cell<'_> {
    Cell::new(Widget::Field {
        id,
        glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
    })
    .label(rotulo)
}

fn mixing(c: &mut CardCtx<'_>, y: f32) -> f32 {
    c.row(
        &[campo(
            pids::PAINTER_WATERCOLOR_MIX,
            tr("panel.painter_layers.watercolor.pigment"),
        )],
        y,
    )
}

fn caixa(c: &mut CardCtx<'_>, id: NodeId, chave: &'static str, on: bool, y: f32) -> f32 {
    ferramenta::caixa(c, id, chave, on, y)
}

fn tiling(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let b = pincel();
    let y = c.pair(
        marca(
            pids::PAINTER_BRUSH_TILING_X,
            "panel.painter_layers.stroke.tiling_x",
            b.tiling[0],
        ),
        marca(
            pids::PAINTER_BRUSH_TILING_Y,
            "panel.painter_layers.stroke.tiling_y",
            b.tiling[1],
        ),
        y,
    );
    caixa(
        c,
        pids::PAINTER_BRUSH_REPEAT_IMAGE,
        "panel.painter_layers.stroke.repeat_image",
        b.repeat_image,
        y,
    )
}

fn botao(c: &mut CardCtx<'_>, id: NodeId, chave: &str, armado: bool, y: f32) -> f32 {
    c.full(
        None,
        Widget::Button {
            id,
            text: tr(chave),
            primary: armado,
        },
        y,
    )
}

fn simetria(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let b = pincel();
    let y = caixa(
        c,
        pids::PAINTER_BRUSH_SYMMETRY_USE,
        "panel.painter_layers.symmetry.use_symmetry",
        b.symmetry_enabled,
        y,
    );
    if !b.symmetry_enabled {
        return y;
    }
    let y = caixa(
        c,
        pids::PAINTER_BRUSH_SYMMETRY_CIRCULAR,
        "panel.painter_layers.symmetry.circular",
        b.symmetry_circular,
        y,
    );
    if b.symmetry_circular {
        let chip = pids::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP;
        let y = c.row(
            &[campo(chip, tr("panel.painter_layers.symmetry.segments"))],
            y,
        );
        return botao(
            c,
            pids::PAINTER_BRUSH_SYMMETRY_PICK_CENTER,
            "panel.painter_layers.symmetry.pick_center",
            b.symmetry_pick_center,
            y,
        );
    }
    let eixo = |id, text, n| Seg {
        id,
        icon: None,
        text,
        on: b.symmetry_axis == n,
    };
    let eixos = [
        eixo(pids::PAINTER_BRUSH_SYMMETRY_AXIS_X, "X", 0),
        eixo(pids::PAINTER_BRUSH_SYMMETRY_AXIS_Y, "Y", 1),
        eixo(
            pids::PAINTER_BRUSH_SYMMETRY_AXIS_CUSTOM,
            tr("panel.painter_layers.symmetry.custom"),
            2,
        ),
    ];
    let y = c.full(
        Some(tr("panel.painter_layers.symmetry.axis")),
        Widget::Choice(&eixos),
        y,
    );
    if b.symmetry_axis == 2 {
        return botao(
            c,
            pids::PAINTER_BRUSH_SYMMETRY_DRAW_LINE,
            "panel.painter_layers.symmetry.draw_custom_line",
            b.symmetry_pick_line,
            y,
        );
    }
    y
}
