//! O cartão Transform (o Inspector aprovado da linha de teste, maquete C): posição, rotação com
//! os três ícones, escala com a corrente, inclinação.

use crate::controlos::{card, field, head, segs};
use crate::ids as g;
use crate::state::Choices;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;

pub(crate) fn transform(ch: Choices, store: &WidgetStore) -> CardDef<'static> {
    card(
        g::GAL_CARD_TRANSFORM,
        tr("panel.widget_gallery.card.transform"),
        vec![head(g::TRANSFORM_RESET, &ch, store)],
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let (gx, gy) = (tr("card.glyph.x"), tr("card.glyph.y"));
            let y = c.row(
                &[
                    Cell::new(field(g::POS_X, Glyph::Text(gx)))
                        .label(tr("panel.widget_gallery.position")),
                    Cell::new(field(g::POS_Y, Glyph::Text(gy))).key(&[g::POS_X, g::POS_Y]),
                ],
                y,
            );
            let giros = segs([g::ROTATE_90, g::FLIP_H, g::FLIP_V], &ch, c.store);
            let y = c.row(
                &[
                    Cell::new(field(g::ROTATION, Glyph::Icon(IconId::CardAngle)))
                        .label(tr("panel.widget_gallery.rotation"))
                        .key(&[g::ROTATION]),
                    Cell::new(Widget::Bare(&giros)),
                ],
                y,
            );
            let corrente = segs([g::SCALE_LINK], &ch, c.store);
            let y = c.row(
                &[
                    Cell::new(field(g::SCALE_X, Glyph::Text(gx)))
                        .label(tr("panel.widget_gallery.scale")),
                    Cell::new(field(g::SCALE_Y, Glyph::Text(gy))).key(&[g::SCALE_X, g::SCALE_Y]),
                    Cell::new(Widget::Bare(&corrente)),
                ],
                y,
            );
            c.row(
                &[
                    Cell::new(field(g::SKEW_X, Glyph::Text(gx)))
                        .label(tr("panel.widget_gallery.skew")),
                    Cell::new(field(g::SKEW_Y, Glyph::Text(gy))).key(&[g::SKEW_X, g::SKEW_Y]),
                ],
                y,
            )
        }),
    )
}
