//! **A janela do File ▸ Import** (dono 05/11): os ficheiros escolhidos, a prévia e o que eles viram,
//! Pixel Sprite ou imagem do Painter.

use ph2d_editor_core::panel::cards::ferramenta::{cartao, opcoes};
use ph2d_editor_core::panel::cards::{CardDef, Widget};
use ph2d_i18n::{tr, tr_with};

use crate::canal::{Imagem, Pergunta};
use crate::comum::{ImagemDoCartao, imagem_no_cartao};
use crate::ids;

fn facto(p: &Pergunta) -> String {
    tr_with(
        "panel.pixel.export.as_facts",
        &[
            ("w", &p.lados.0.to_string()),
            ("h", &p.lados.1.to_string()),
            ("f", &p.quadros.to_string()),
            ("l", &p.camadas.to_string()),
            ("t", &p.tags.to_string()),
        ],
    )
}

pub(crate) fn cards(
    p: Pergunta,
    pixel: bool,
    img: Option<(ImagemDoCartao, Imagem)>,
) -> Vec<CardDef<'static>> {
    let varios = p.ficheiros > 1;
    let n = p.ficheiros.to_string();
    let titulo = if varios {
        tr_with(
            "panel.pixel.export.as_many",
            &[("name", &p.nome), ("n", &n)],
        )
    } else {
        p.nome.clone()
    };
    let so_png = varios && p.nome.to_ascii_lowercase().ends_with(".png");
    vec![
        cartao(
            ids::SEC_IMAGE,
            tr("panel.pixel.export.sec_file"),
            None,
            move |c, y| {
                let y = c.row(
                    &[ph2d_editor_core::panel::cards::Cell::new(Widget::Readout {
                        text: &titulo,
                    })],
                    y,
                );
                let y = match &img {
                    Some((i, _)) => imagem_no_cartao(c, i, None, y),
                    None => y,
                };
                match &p.erro {
                    Some(e) => c.problem(e, y),
                    None => c.hint(&facto(&p), y),
                }
            },
        ),
        cartao(
            ids::SEC_GRID,
            tr("panel.pixel.export.as_title"),
            None,
            move |c, y| {
                let segs = opcoes(
                    &[
                        (ids::AS_PIXEL, tr("panel.pixel.export.as_pixel")),
                        (ids::AS_IMAGE, tr("panel.pixel.export.as_image")),
                    ],
                    usize::from(!pixel),
                );
                let y = c.full(None, Widget::Choice(&segs), y);
                let dica = match (pixel, so_png) {
                    (true, true) => "panel.pixel.export.as_pixel_seq",
                    (true, false) => "panel.pixel.export.as_pixel_hint",
                    (false, _) => "panel.pixel.export.as_image_hint",
                };
                c.hint(tr(dica), y)
            },
        ),
    ]
}
