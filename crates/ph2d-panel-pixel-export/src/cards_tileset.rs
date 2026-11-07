//! **Export Tileset** (File ▸ Export ▸ Export Tileset): os tiles em PNG e o mapa em JSON, com a imagem
//! ao vivo no primeiro cartão.

use ph2d_editor_core::panel::cards::ferramenta::{Numero, campo, cartao, opcoes};
use ph2d_editor_core::panel::cards::{CardDef, Widget};
use ph2d_i18n::{tr, tr_with};
use ph2d_pixel_io::tiles_saida::COLUNAS_INICIAIS;

use crate::canal::{Contexto, Imagem};
use crate::comum::{ImagemDoCartao, imagem_no_cartao};
use crate::ids;
use crate::state::OpcoesTi;

pub(crate) fn numeros(o: OpcoesTi) -> Vec<Numero> {
    vec![Numero {
        chip: ids::COLUMNS,
        step: 1.0,
        scrub: 1.0,
        fabrica: f64::from(COLUNAS_INICIAIS),
        valor: f64::from(o.colunas),
    }]
}

pub(crate) fn cards(
    o: OpcoesTi,
    x: &Contexto,
    img: Option<(ImagemDoCartao, Imagem)>,
) -> Vec<CardDef<'static>> {
    let escolhido = o.escolhido(x);
    let linhas: Vec<(ph2d_a11y::NodeId, String, bool)> = x
        .tilesets
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let rotulo = tr_with(
                "panel.pixel.tiles.set_label",
                &[
                    ("i", &(i + 1).to_string()),
                    ("n", &t.1.to_string()),
                    ("w", &t.2.to_string()),
                    ("h", &t.3.to_string()),
                ],
            );
            (ids::tileset_option(i), rotulo, Some(t.0) == escolhido)
        })
        .collect();
    let previa = cartao(
        ids::SEC_PREVIEW,
        tr("panel.pixel.export.sec_preview"),
        None,
        move |c, y| {
            let Some((i, p)) = &img else {
                return c.hint(tr("panel.pixel.export.tileset_none"), y);
            };
            let y = imagem_no_cartao(c, i, None, y);
            let t = tr_with(
                "panel.pixel.export.tileset_size",
                &[
                    ("w", &p.lados_reais.0.to_string()),
                    ("h", &p.lados_reais.1.to_string()),
                    ("n", &p.entradas.to_string()),
                ],
            );
            c.hint(&t, y)
        },
    );
    let conteudo = cartao(
        ids::SEC_TILESET,
        tr("panel.pixel.tiles.tileset"),
        None,
        move |c, mut y| {
            for (id, rotulo, aceso) in &linhas {
                let l = opcoes(&[(*id, rotulo.as_str())], usize::from(!*aceso));
                y = c.full(None, Widget::Choice(&l), y);
            }
            let y = c.row(&[campo(ids::COLUMNS, tr("panel.pixel.export.columns"))], y);
            c.hint(tr("panel.pixel.export.tileset_hint"), y)
        },
    );
    vec![previa, conteudo]
}
