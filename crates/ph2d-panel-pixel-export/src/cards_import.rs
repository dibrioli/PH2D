//! **Import Sprite Sheet** (File ▸ Import, `Ctrl+I`): a imagem com a grelha por cima e os números dela.

use ph2d_editor_core::panel::cards::ferramenta::{Numero, campo, cartao, opcoes};
use ph2d_editor_core::panel::cards::{CardDef, Widget};
use ph2d_i18n::{tr, tr_with};
use ph2d_pixel_io::abrir::{Grelha, celulas};

use crate::canal::Imagem;
use crate::comum::{ImagemDoCartao, imagem_no_cartao};
use crate::ids;
use crate::state::{GRELHA_INICIAL, grelha_inicial};

pub(crate) fn numeros(g: &Grelha) -> Vec<Numero> {
    let n = |chip, fabrica: u32, valor: u32| Numero {
        chip,
        step: 1.0,
        scrub: 1.0,
        fabrica: f64::from(fabrica),
        valor: f64::from(valor),
    };
    let i = grelha_inicial();
    vec![
        n(ids::GRID_X, i.x, g.x),
        n(ids::GRID_Y, i.y, g.y),
        n(ids::GRID_W, GRELHA_INICIAL, g.w),
        n(ids::GRID_H, GRELHA_INICIAL, g.h),
        n(ids::GAP_X, i.gap_x, g.gap_x),
        n(ids::GAP_Y, i.gap_y, g.gap_y),
    ]
}

pub(crate) fn cards(g: Grelha, img: Option<(ImagemDoCartao, Imagem)>) -> Vec<CardDef<'static>> {
    let lados = img.as_ref().map_or((0, 0), |(_, p)| (p.w, p.h));
    vec![
        cartao(
            ids::SEC_IMAGE,
            tr("panel.pixel.export.sec_image"),
            None,
            move |c, y| match &img {
                Some((i, _)) => imagem_no_cartao(c, i, Some((g, lados)), y),
                None => c.hint(tr("panel.pixel.export.preview_wait"), y),
            },
        ),
        cartao(
            ids::SEC_GRID,
            tr("panel.pixel.export.sec_grid"),
            None,
            move |c, y| {
                let y = c.pair(
                    campo(ids::GRID_X, tr("panel.pixel.export.grid_x")),
                    campo(ids::GRID_Y, tr("panel.pixel.export.grid_y")),
                    y,
                );
                let y = c.pair(
                    campo(ids::GRID_W, tr("panel.pixel.export.grid_w")),
                    campo(ids::GRID_H, tr("panel.pixel.export.grid_h")),
                    y,
                );
                let y = c.pair(
                    campo(ids::GAP_X, tr("panel.pixel.export.gap_x")),
                    campo(ids::GAP_Y, tr("panel.pixel.export.gap_y")),
                    y,
                );
                let segs = opcoes(
                    &[
                        (ids::BY_ROWS, tr("panel.pixel.export.by_rows")),
                        (ids::BY_COLUMNS, tr("panel.pixel.export.by_columns")),
                    ],
                    usize::from(g.por_colunas),
                );
                let y = c.full(
                    Some(tr("panel.pixel.export.order")),
                    Widget::Choice(&segs),
                    y,
                );
                let (cols, rows) = celulas(&g, lados.0, lados.1);
                if cols == 0 || rows == 0 {
                    return c.problem(tr("panel.pixel.export.import_none"), y);
                }
                let t = tr_with(
                    "panel.pixel.export.import_count",
                    &[
                        ("c", &cols.to_string()),
                        ("r", &rows.to_string()),
                        ("n", &(cols * rows).to_string()),
                    ],
                );
                c.hint(&t, y)
            },
        ),
    ]
}
