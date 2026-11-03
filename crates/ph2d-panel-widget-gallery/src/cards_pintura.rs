//! Appearance, Fill, Stroke e Effects — a pilha de tinta do Inspector aprovado (maquete C) e a
//! lista de efeitos à F43. As linhas são fixas por tabela: «+» liga a primeira livre, «−» a sua.

use crate::controlos::{
    CAMPO_MIN, SWATCHES, caixa_para, card, dropdown, field, head, seg_of, segs,
};
use crate::ids::{self as g, PaintRow};
use crate::state::{Choices, FILLS, FXS, Flag, Pick, STROKES};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, HeadAction, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;

pub(crate) fn appearance(ch: Choices, store: &WidgetStore) -> CardDef<'static> {
    card(
        g::CARD_APPEARANCE,
        tr("panel.widget_gallery.card.appearance"),
        vec![head(g::VISIBLE, &ch, store)],
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let modos = segs(
                [
                    g::BLEND_NORMAL,
                    g::BLEND_MULTIPLY,
                    g::BLEND_SCREEN,
                    g::BLEND_OVERLAY,
                ],
                &ch,
                c.store,
            );
            c.row(
                &[
                    Cell::new(field(g::OPACITY, Glyph::Icon(IconId::CardOpacity)))
                        .label(tr("panel.widget_gallery.opacity"))
                        .key(&[g::OPACITY]),
                    Cell::new(dropdown(g::BLEND, &modos))
                        .label(tr("panel.widget_gallery.blend"))
                        .key(&[g::BLEND]),
                ],
                y,
            )
        }),
    )
}

/// A amostra `i` de [`SWATCHES`] (cor e código), pela tinta do cartão.
fn swatch(c: &CardCtx<'_>, ch: &Choices, i: usize, code: &'_ mut String) -> ([u8; 4], NodeId) {
    let rgba = crate::valores::rgba(ch, i, c.ink);
    *code = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
    (rgba, SWATCHES[i].1)
}

/// `[amostra│código] [opacidade] 👁 −`; a primeira linha da lista leva os rótulos.
fn paint_row(
    c: &mut CardCtx<'_>,
    ch: &Choices,
    row: &PaintRow,
    i: usize,
    rot: bool,
    y: f32,
) -> f32 {
    let mut code = String::new();
    let (rgba, id) = swatch(c, ch, i, &mut code);
    let olho = segs([row.eye, row.remove], ch, c.store);
    let k_cor = [row.swatch];
    let k_op = [row.opacity];
    let mut cor = Cell::new(Widget::Swatch {
        id,
        rgba: Some(rgba),
        text: &code,
        edit: None,
        detach: None,
    });
    let mut op = Cell::new(field(row.opacity, Glyph::Icon(IconId::CardOpacity)));
    if rot {
        cor = cor.label(tr("panel.widget_gallery.color")).key(&k_cor);
        op = op.label(tr("panel.widget_gallery.opacity")).key(&k_op);
    }
    let olhos = Cell::new(Widget::Bare(&olho));
    let toda = caixa_para(c, &code) + CAMPO_MIN + m::FIELD_H * 2.0 + m::FIELD_GAP * 2.0;
    if c.w >= toda {
        return c.row(&[cor, op, olhos], y);
    }
    let y = c.row(&[cor], y);
    c.row(&[op, olhos], y)
}

/// As linhas presentes de uma lista de tinta, cada uma com a sua amostra (`base + i`).
fn paint_rows(
    c: &mut CardCtx<'_>,
    ch: &Choices,
    rows: &[PaintRow; 3],
    flags: [Flag; 3],
    base: usize,
    y: f32,
) -> f32 {
    let mut y = y;
    let mut primeira = true;
    for (i, row) in rows.iter().enumerate() {
        if ch.on(flags[i]) {
            y = paint_row(c, ch, row, base + i, primeira, y);
            primeira = false;
        }
    }
    y
}

/// As ações de cabeçalho de uma lista: os estilos e o «+» (que some com a lista cheia).
fn head_of(
    ch: &Choices,
    store: &WidgetStore,
    styles: Option<NodeId>,
    add: NodeId,
    flags: [Flag; 3],
) -> Vec<HeadAction> {
    let mut v = Vec::new();
    if let Some(s) = styles {
        v.push(head(s, ch, store));
    }
    if flags.iter().any(|f| ch.on(*f)) && !flags.iter().all(|f| ch.on(*f)) {
        v.push(head(add, ch, store));
    }
    v
}

fn ausente(flags: [Flag; 3], ch: &Choices, add: NodeId) -> Option<NodeId> {
    (!flags.iter().any(|f| ch.on(*f))).then_some(add)
}

pub(crate) fn fill(ch: Choices, store: &WidgetStore) -> CardDef<'static> {
    let mut d = card(
        g::CARD_FILL,
        tr("panel.widget_gallery.card.fill"),
        head_of(&ch, store, Some(g::FILL_STYLES), g::FILL_ADD, FILLS),
        Box::new(move |c: &mut CardCtx<'_>, y| paint_rows(c, &ch, &g::FILLS, FILLS, 0, y)),
    );
    d.add = ausente(FILLS, &ch, g::FILL_ADD);
    d
}

fn stroke_props(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let posicoes = segs(
        [
            g::STROKE_POS_CENTER,
            g::STROKE_POS_INSIDE,
            g::STROKE_POS_OUTSIDE,
        ],
        ch,
        c.store,
    );
    let y = c.row(
        &[
            Cell::new(dropdown(g::STROKE_POS, &posicoes))
                .label(tr("panel.widget_gallery.stroke_position"))
                .key(&[g::STROKE_POS]),
            Cell::new(field(g::WEIGHT, Glyph::Icon(IconId::CardWeight)))
                .label(tr("panel.widget_gallery.weight"))
                .key(&[g::WEIGHT]),
        ],
        y,
    );
    let caps = segs([g::CAP_BUTT, g::CAP_ROUND, g::CAP_SQUARE], ch, c.store);
    let joins = segs([g::JOIN_MITER, g::JOIN_ROUND, g::JOIN_BEVEL], ch, c.store);
    c.row(
        &[
            Cell::new(Widget::Choice(&caps))
                .label(tr("panel.widget_gallery.cap"))
                .key(&[g::CAP_BUTT, g::CAP_ROUND, g::CAP_SQUARE]),
            Cell::new(Widget::Choice(&joins))
                .label(tr("panel.widget_gallery.join"))
                .key(&[g::JOIN_MITER, g::JOIN_ROUND, g::JOIN_BEVEL]),
        ],
        y,
    )
}

pub(crate) fn stroke(ch: Choices, store: &WidgetStore) -> CardDef<'static> {
    let mut d = card(
        g::CARD_STROKE,
        tr("panel.widget_gallery.card.stroke"),
        head_of(&ch, store, Some(g::STROKE_STYLES), g::STROKE_ADD, STROKES),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let y = paint_rows(c, &ch, &g::STROKES, STROKES, 3, y);
            stroke_props(c, &ch, y)
        }),
    );
    d.add = ausente(STROKES, &ch, g::STROKE_ADD);
    d
}

/// Uma linha de efeito: `[tipo ▾] 👁 −`; as sombras levam deslocamento, cor, desfoque e expansão,
/// os desfoques só o desfoque.
fn effect_row(c: &mut CardCtx<'_>, ch: &Choices, i: usize, y: f32) -> f32 {
    let row = g::EFFECTS[i];
    let picks = [Pick::Fx0, Pick::Fx1, Pick::Fx2];
    let tipos = row.options.map(|id| seg_of(id, ch, c.store));
    let olho = segs([row.eye, row.remove], ch, c.store);
    let lista = Cell::new(dropdown(row.kind, &tipos));
    let olhos = Cell::new(Widget::Bare(&olho));
    let larga = tipos
        .iter()
        .fold(0.0_f32, |w, s| w.max(caixa_para(c, s.text)));
    let y = if c.w >= larga + m::FIELD_H * 2.0 + m::FIELD_GAP {
        c.row(&[lista, olhos], y)
    } else {
        let y = c.row(&[lista], y);
        c.row(&[olhos], y)
    };
    let k_blur = [row.blur];
    if ch.pick(picks[i]) >= 2 {
        return c.row(
            &[Cell::new(field(row.blur, Glyph::Icon(IconId::CardSoft)))
                .label(tr("panel.widget_gallery.blur"))
                .key(&k_blur)],
            y,
        );
    }
    let mut code = String::new();
    let (rgba, id) = swatch(c, ch, 6 + i, &mut code);
    let (k_pos, k_cor, k_sp) = ([row.x, row.y], [row.swatch], [row.spread]);
    let ox = Cell::new(field(row.x, Glyph::Text(tr("card.glyph.x"))))
        .label(tr("panel.widget_gallery.offset"));
    let oy = Cell::new(field(row.y, Glyph::Text(tr("card.glyph.y")))).key(&k_pos);
    let cor = Cell::new(Widget::Swatch {
        id,
        rgba: Some(rgba),
        text: &code,
        edit: None,
        detach: None,
    })
    .label(tr("panel.widget_gallery.color"))
    .key(&k_cor);
    let y = if c.w >= caixa_para(c, &code) + CAMPO_MIN * 2.0 + m::FIELD_GAP * 2.0 {
        c.row(&[ox, oy, cor], y)
    } else {
        let y = c.row(&[ox, oy], y);
        c.row(&[cor], y)
    };
    c.row(
        &[
            Cell::new(field(row.blur, Glyph::Icon(IconId::CardSoft)))
                .label(tr("panel.widget_gallery.blur"))
                .key(&k_blur),
            Cell::new(field(row.spread, Glyph::Icon(IconId::CardLen)))
                .label(tr("panel.widget_gallery.spread"))
                .key(&k_sp),
        ],
        y,
    )
}

pub(crate) fn effects(ch: Choices, store: &WidgetStore) -> CardDef<'static> {
    let mut d = card(
        g::CARD_EFFECTS,
        tr("panel.widget_gallery.card.effects"),
        head_of(&ch, store, None, g::FX_ADD, FXS),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let mut y = y;
            for (i, f) in FXS.iter().enumerate() {
                if ch.on(*f) {
                    y = effect_row(c, &ch, i, y);
                }
            }
            y
        }),
    );
    d.add = ausente(FXS, &ch, g::FX_ADD);
    d
}
