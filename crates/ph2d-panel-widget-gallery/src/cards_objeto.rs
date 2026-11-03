//! Os cartões do OBJETO (doc `CartoesUI/08` §1.1): a identidade sem título, a fila de ferramentas
//! Align e o Boolean — o topo do painel direito do Figma.

use crate::controlos::{card, segs};
use crate::ids as g;
use crate::state::Choices;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_i18n::tr;

/// `[tipo] [nome] [olho] [cadeado]`, sem título (maquete C).
pub(crate) fn identity(ch: Choices) -> CardDef<'static> {
    card(
        g::GAL_CARD_IDENTITY,
        "",
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let olho = segs([g::VISIBLE, g::LOCK], &ch, c.store);
            c.row(
                &[
                    Cell::new(Widget::Badge(IconId::CardPath)),
                    Cell::new(Widget::Text {
                        id: g::NAME,
                        placeholder: tr("panel.widget_gallery.name"),
                    }),
                    Cell::new(Widget::Bare(&olho)),
                ],
                y,
            )
        }),
    )
}

/// A fila do Figma: alinhar × distribuir. Na coluna que não leva as duas, cada grupo na sua linha.
pub(crate) fn align(ch: Choices) -> CardDef<'static> {
    card(
        g::GAL_CARD_ALIGN,
        tr("panel.widget_gallery.card.align"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let alinhar = segs(
                [
                    g::ALIGN_LEFT,
                    g::ALIGN_HCENTER,
                    g::ALIGN_RIGHT,
                    g::ALIGN_TOP,
                    g::ALIGN_VCENTER,
                    g::ALIGN_BOTTOM,
                ],
                &ch,
                c.store,
            );
            let distribuir = segs([g::DISTRIBUTE_H, g::DISTRIBUTE_V], &ch, c.store);
            let toda = (alinhar.len() + distribuir.len()) as f32 * m::FIELD_H + m::FIELD_GAP;
            if c.w >= toda {
                return c.row(
                    &[
                        Cell::new(Widget::Bare(&alinhar)),
                        Cell::new(Widget::Bare(&distribuir)),
                    ],
                    y,
                );
            }
            let y = c.row(&[Cell::new(Widget::Bare(&alinhar))], y);
            c.row(&[Cell::new(Widget::Bare(&distribuir))], y)
        }),
    )
}

/// Union · Subtract · Intersect · Exclude + Flatten e Convert (dono 02/11: a fila do topo).
pub(crate) fn boolean(ch: Choices) -> CardDef<'static> {
    card(
        g::GAL_CARD_BOOLEAN,
        tr("panel.widget_gallery.card.boolean"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let modos = segs(
                [
                    g::BOOL_UNION,
                    g::BOOL_SUBTRACT,
                    g::BOOL_INTERSECT,
                    g::BOOL_EXCLUDE,
                ],
                &ch,
                c.store,
            );
            let actos = segs([g::FLATTEN, g::CONVERT], &ch, c.store);
            c.row(
                &[
                    Cell::new(Widget::Choice(&modos)),
                    Cell::new(Widget::Bare(&actos)),
                ],
                y,
            )
        }),
    )
}
