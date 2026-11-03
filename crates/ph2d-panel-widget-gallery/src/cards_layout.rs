//! Layout, Corners e Constraints à Figma (doc `CartoesUI/08` §1.1).

use crate::controlos::{CAMPO_MIN, card, dropdown, field, segs};
use crate::ids as g;
use crate::state::{Choices, Flag};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::corpo::{self, Marca};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::tr;

/// A grelha 3×3 do alinhamento à esquerda, o `Gap` e o `Padding` à direita. Compõe-se só com a
/// `row` pública: duas colunas estreitando o `x`/`w` do corpo e o fundo é o da mais alta.
fn grelha_e_folgas(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let (x0, w0) = (c.x, c.w);
    let linhas = [
        [g::GRID[0], g::GRID[1], g::GRID[2]],
        [g::GRID[3], g::GRID[4], g::GRID[5]],
        [g::GRID[6], g::GRID[7], g::GRID[8]],
    ];
    let col = m::FIELD_H * linhas[0].len() as f32;
    c.w = col;
    let mut fundo = y;
    for (i, ids) in linhas.into_iter().enumerate() {
        let s = segs(ids, ch, c.store);
        let cel = Cell::new(Widget::Icons(&s));
        let cel = if i == 0 {
            cel.label(tr("panel.widget_gallery.align")).key(&g::GRID)
        } else {
            cel
        };
        fundo = c.row(&[cel], fundo);
    }
    // Na coluna que não leva as duas, as folgas descem para baixo da grelha, a toda a largura.
    let lado = w0 >= col + m::FIELD_GAP * 2.0 + CAMPO_MIN * 2.0;
    let topo = if lado {
        c.x = x0 + col + m::FIELD_GAP;
        c.w = w0 - col - m::FIELD_GAP;
        y
    } else {
        c.w = w0;
        fundo
    };
    let direita = c.row(
        &[Cell::new(field(g::GAP, Glyph::Icon(IconId::CardLen)))
            .label(tr("panel.widget_gallery.gap"))
            .key(&[g::GAP])],
        topo,
    );
    let direita = c.row(
        &[
            Cell::new(field(
                g::PAD_H,
                Glyph::Text(tr("panel.widget_gallery.glyph.h")),
            ))
            .label(tr("panel.widget_gallery.padding")),
            Cell::new(field(
                g::PAD_V,
                Glyph::Text(tr("panel.widget_gallery.glyph.v")),
            ))
            .key(&[g::PAD_H, g::PAD_V]),
        ],
        direita,
    );
    c.x = x0;
    c.w = w0;
    fundo.max(direita)
}

pub(crate) fn layout(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_LAYOUT,
        tr("panel.widget_gallery.card.layout"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let dirs = segs([g::DIR_ROW, g::DIR_COL, g::DIR_WRAP], &ch, c.store);
            let y = c.row(
                &[Cell::new(Widget::Choice(&dirs))
                    .label(tr("panel.widget_gallery.direction"))
                    .key(&[g::DIR_ROW, g::DIR_COL, g::DIR_WRAP])],
                y,
            );
            let (gw, gh) = (
                tr("panel.widget_gallery.glyph.w"),
                tr("panel.widget_gallery.glyph.h"),
            );
            let y = c.row(
                &[
                    Cell::new(field(g::SIZE_W, Glyph::Text(gw)))
                        .label(tr("panel.widget_gallery.size")),
                    Cell::new(field(g::SIZE_H, Glyph::Text(gh))).key(&[g::SIZE_W, g::SIZE_H]),
                ],
                y,
            );
            let modo_w = segs(
                [g::RESIZE_W_FIXED, g::RESIZE_W_HUG, g::RESIZE_W_FILL],
                &ch,
                c.store,
            );
            let modo_h = segs(
                [g::RESIZE_H_FIXED, g::RESIZE_H_HUG, g::RESIZE_H_FILL],
                &ch,
                c.store,
            );
            let y = c.row(
                &[
                    Cell::new(dropdown(g::RESIZE_W, &modo_w))
                        .label(tr("panel.widget_gallery.resizing")),
                    Cell::new(dropdown(g::RESIZE_H, &modo_h)).key(&[g::RESIZE_W, g::RESIZE_H]),
                ],
                y,
            );
            grelha_e_folgas(c, &ch, y)
        }),
    )
}

pub(crate) fn corners(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_CORNERS,
        tr("panel.widget_gallery.card.corners"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let cantos = segs([g::CORNER_EACH], &ch, c.store);
            let mut y = c.row(
                &[
                    Cell::new(field(g::RADIUS, Glyph::Icon(IconId::CardRadius)))
                        .label(tr("panel.widget_gallery.radius"))
                        .key(&[g::RADIUS]),
                    Cell::new(Widget::Bare(&cantos)),
                ],
                y,
            );
            if ch.on(Flag::CornerEach) {
                y = c.row(
                    &[
                        Cell::new(field(
                            g::RADIUS_TL,
                            Glyph::Text(tr("panel.widget_gallery.glyph.tl")),
                        ))
                        .label(tr("panel.widget_gallery.each_corner")),
                        Cell::new(field(
                            g::RADIUS_TR,
                            Glyph::Text(tr("panel.widget_gallery.glyph.tr")),
                        ))
                        .key(&[
                            g::RADIUS_TL,
                            g::RADIUS_TR,
                            g::RADIUS_BL,
                            g::RADIUS_BR,
                        ]),
                    ],
                    y,
                );
                y = c.row(
                    &[
                        Cell::new(field(
                            g::RADIUS_BL,
                            Glyph::Text(tr("panel.widget_gallery.glyph.bl")),
                        )),
                        Cell::new(field(
                            g::RADIUS_BR,
                            Glyph::Text(tr("panel.widget_gallery.glyph.br")),
                        )),
                    ],
                    y,
                );
            }
            corpo::check(
                c.scene,
                c.text_system,
                c.store,
                c.hit_index,
                c.ink,
                c.x,
                c.w,
                y,
                Marca {
                    id: g::CLIP,
                    label: Rotulo::vista(TextKey::new("panel.widget_gallery.clip")),
                    on: ch.on(Flag::Clip),
                },
            )
        }),
    )
}

pub(crate) fn constraints(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_CONSTRAINTS,
        tr("panel.widget_gallery.card.constraints"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let h = segs(
                [
                    g::CONS_H_LEFT,
                    g::CONS_H_RIGHT,
                    g::CONS_H_CENTER,
                    g::CONS_H_SCALE,
                ],
                &ch,
                c.store,
            );
            let v = segs(
                [
                    g::CONS_V_TOP,
                    g::CONS_V_BOTTOM,
                    g::CONS_V_CENTER,
                    g::CONS_V_SCALE,
                ],
                &ch,
                c.store,
            );
            c.row(
                &[
                    Cell::new(dropdown(g::CONS_H, &h))
                        .label(tr("panel.widget_gallery.horizontal"))
                        .key(&[g::CONS_H]),
                    Cell::new(dropdown(g::CONS_V, &v))
                        .label(tr("panel.widget_gallery.vertical"))
                        .key(&[g::CONS_V]),
                ],
                y,
            )
        }),
    )
}
