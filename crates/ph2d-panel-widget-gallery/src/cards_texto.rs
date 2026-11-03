//! Text e Export à Figma (doc `CartoesUI/08` §1.1).

use crate::controlos::{caixa_para, card, dropdown, field, segs};
use crate::ids as g;
use crate::state::Choices;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;

pub(crate) fn text(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_TEXT,
        tr("panel.widget_gallery.card.text"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let fontes = segs([g::FONT_INTER, g::FONT_ROBOTO, g::FONT_NOTO], &ch, c.store);
            let y = c.row(
                &[Cell::new(dropdown(g::FONT, &fontes))
                    .label(tr("panel.widget_gallery.font"))
                    .key(&[g::FONT])],
                y,
            );
            let pesos = segs(
                [g::FONT_REGULAR, g::FONT_MEDIUM, g::FONT_BOLD],
                &ch,
                c.store,
            );
            let y = c.row(
                &[
                    Cell::new(dropdown(g::FONT_WEIGHT, &pesos))
                        .label(tr("panel.widget_gallery.font_weight"))
                        .key(&[g::FONT_WEIGHT]),
                    Cell::new(field(g::FONT_SIZE, Glyph::Icon(IconId::Text)))
                        .label(tr("panel.widget_gallery.font_size"))
                        .key(&[g::FONT_SIZE]),
                ],
                y,
            );
            let y = c.row(
                &[
                    Cell::new(field(g::LINE_HEIGHT, Glyph::Icon(IconId::CardWeight)))
                        .label(tr("panel.widget_gallery.line_height"))
                        .key(&[g::LINE_HEIGHT]),
                    Cell::new(field(g::LETTER_SPACING, Glyph::Icon(IconId::CardLen)))
                        .label(tr("panel.widget_gallery.letter_spacing"))
                        .key(&[g::LETTER_SPACING]),
                ],
                y,
            );
            let alinhar = segs([g::TEXT_LEFT, g::TEXT_CENTER, g::TEXT_RIGHT], &ch, c.store);
            c.row(
                &[Cell::new(Widget::Choice(&alinhar))
                    .label(tr("panel.widget_gallery.text_align"))
                    .key(&[g::TEXT_LEFT, g::TEXT_CENTER, g::TEXT_RIGHT])],
                y,
            )
        }),
    )
}

pub(crate) fn export(ch: Choices, exported: bool) -> CardDef<'static> {
    card(
        g::CARD_EXPORT,
        tr("panel.widget_gallery.card.export"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let escalas = segs([g::EXPORT_X1, g::EXPORT_X2, g::EXPORT_X3], &ch, c.store);
            let formatos = segs(
                [g::FORMAT_PNG, g::FORMAT_JPG, g::FORMAT_SVG, g::FORMAT_PDF],
                &ch,
                c.store,
            );
            let escala = Cell::new(dropdown(g::EXPORT_SCALE, &escalas))
                .label(tr("panel.widget_gallery.export_scale"));
            let sufixo = Cell::new(Widget::Text {
                id: g::SUFFIX,
                placeholder: tr("panel.widget_gallery.suffix_placeholder"),
            })
            .label(tr("panel.widget_gallery.suffix"));
            let formato =
                Cell::new(dropdown(g::FORMAT, &formatos)).label(tr("panel.widget_gallery.format"));
            let larga = formatos
                .iter()
                .fold(0.0_f32, |w, s| w.max(caixa_para(c, s.text)));
            let tres = [escala, sufixo, formato];
            let n = tres.len() as f32;
            let y = if (c.w - m::FIELD_GAP * (n - 1.0)) / n >= larga {
                c.row(&tres, y)
            } else {
                let y = c.row(&[escala, sufixo], y);
                c.row(&[formato], y)
            };
            let estado = if exported {
                tr("panel.widget_gallery.export_status.done")
            } else {
                tr("panel.widget_gallery.export_status.ready")
            };
            c.row(
                &[Cell::new(Widget::Button {
                    id: g::EXPORT,
                    text: tr("panel.widget_gallery.export"),
                    primary: true,
                })
                .label(estado)],
                y,
            )
        }),
    )
}
