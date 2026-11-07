//! **Export As** (File ▸ Export As, `Ctrl+Alt+Shift+S`): formato, camadas, quadros, direção, escala.

use ph2d_editor_core::panel::cards::ferramenta::{Numero, campo, cartao, opcoes};
use ph2d_editor_core::panel::cards::{CardDef, Widget, dropdown};
use ph2d_i18n::tr;
use ph2d_pixel_io::render::{Direcao, Quadros};
use ph2d_pixel_io::saida::Formato;

use crate::canal::Contexto;
use crate::comum::{
    FORMATOS, Linhas, linhas_de_area, linhas_de_camadas, linhas_de_quadros, listas,
};
use crate::ids;
use crate::state::OpcoesAs;

pub(crate) const DIRECOES: [Direcao; 4] = [
    Direcao::Forward,
    Direcao::Reverse,
    Direcao::PingPong,
    Direcao::PingPongReverse,
];

fn linhas_de_direcao(d: Direcao) -> Linhas {
    const NOMES: [&str; 4] = [
        "panel.pixel.export.dir_forward",
        "panel.pixel.export.dir_reverse",
        "panel.pixel.export.dir_pingpong",
        "panel.pixel.export.dir_pingpong_reverse",
    ];
    DIRECOES
        .iter()
        .enumerate()
        .map(|(i, x)| (ids::DIRECTIONS[i], tr(NOMES[i]).to_owned(), *x == d))
        .collect()
}

/// O formato anima? (O laço só existe nestes.)
const fn anima(f: Formato) -> bool {
    matches!(f, Formato::Gif | Formato::Apng | Formato::Webp)
}

/// Os números que esta janela mostra agora, com o valor das opções.
pub(crate) fn numeros(o: &OpcoesAs, x: &Contexto) -> Vec<Numero> {
    let n = |chip, fabrica: f64, valor: f64| Numero {
        chip,
        step: 1.0,
        scrub: 1.0,
        fabrica,
        valor,
    };
    let ultimo = f64::from(x.quadros.max(1));
    let (de, ate) = match o.quadros {
        Quadros::Intervalo { de, ate } => (de as f64 + 1.0, ate as f64 + 1.0),
        _ => (1.0, ultimo),
    };
    vec![
        n(
            ids::SCALE,
            f64::from(OpcoesAs::default().escala),
            f64::from(o.escala),
        ),
        n(ids::LOOPS, 0.0, f64::from(o.lacos)),
        n(ids::FROM, 1.0, de),
        n(ids::TO, ultimo, ate),
    ]
}

pub(crate) fn prepare() {
    for caixa in [ids::LAYERS, ids::FRAMES, ids::DIRECTION, ids::AREA] {
        dropdown::declare(caixa);
    }
}

pub(crate) fn cards(o: OpcoesAs, x: &Contexto) -> Vec<CardDef<'static>> {
    let fmt = o.formato;
    let i_fmt = FORMATOS.iter().position(|f| *f == fmt).unwrap_or(0);
    let mut v = vec![cartao(
        ids::SEC_FILE,
        tr("panel.pixel.export.sec_file"),
        None,
        move |c, y| {
            let segs = opcoes(
                &[
                    (ids::FORMATS[0], tr("panel.pixel.export.fmt_png")),
                    (ids::FORMATS[1], tr("panel.pixel.export.fmt_gif")),
                    (ids::FORMATS[2], tr("panel.pixel.export.fmt_apng")),
                    (ids::FORMATS[3], tr("panel.pixel.export.fmt_webp")),
                    (ids::FORMATS[4], tr("panel.pixel.export.fmt_ase")),
                ],
                i_fmt,
            );
            let y = c.full(
                Some(tr("panel.pixel.export.format")),
                Widget::Choice(&segs),
                y,
            );
            let dica = match fmt {
                Formato::Png => "panel.pixel.export.hint_png",
                Formato::Gif => "panel.pixel.export.hint_gif",
                Formato::Apng => "panel.pixel.export.hint_apng",
                Formato::Webp => "panel.pixel.export.hint_webp",
                Formato::Ase => "panel.pixel.export.hint_ase",
            };
            c.hint(tr(dica), y)
        },
    )];
    // O `.ase` leva o sprite inteiro: nada aqui se aplica.
    if fmt != Formato::Ase {
        let camadas = linhas_de_camadas(x, o.camadas);
        let quadros = linhas_de_quadros(x, o.quadros);
        let direcao = linhas_de_direcao(o.direcao);
        let area = (!x.slices.is_empty()).then(|| linhas_de_area(x, o.area));
        let intervalo = matches!(o.quadros, Quadros::Intervalo { .. });
        v.push(cartao(
            ids::SEC_CONTENT,
            tr("panel.pixel.export.sec_content"),
            None,
            move |c, y| {
                let y = listas(
                    c,
                    (ids::LAYERS, tr("panel.pixel.export.layers"), &camadas),
                    Some((ids::FRAMES, tr("panel.pixel.export.frames"), &quadros)),
                    y,
                );
                let y = if intervalo {
                    c.pair(
                        campo(ids::FROM, tr("panel.pixel.export.from")),
                        campo(ids::TO, tr("panel.pixel.export.to")),
                        y,
                    )
                } else {
                    y
                };
                let y = listas(
                    c,
                    (ids::DIRECTION, tr("panel.pixel.export.direction"), &direcao),
                    area.as_ref()
                        .map(|a| (ids::AREA, tr("panel.pixel.export.area"), a)),
                    y,
                );
                let escala = campo(ids::SCALE, tr("panel.pixel.export.scale"));
                if anima(fmt) {
                    let lacos = campo(ids::LOOPS, tr("panel.pixel.export.loops"));
                    c.pair(escala, lacos, y)
                } else {
                    c.row(&[escala], y)
                }
            },
        ));
    }
    v
}
