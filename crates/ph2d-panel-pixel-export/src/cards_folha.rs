//! **Export Sprite Sheet** (File ▸ Export ▸ Export Sprite Sheet, `Ctrl+E`): os quatro grupos do
//! Aseprite (Layout, Sprite, Borders, Output) em cartões, com a folha ao vivo no primeiro.

use ph2d_editor_core::panel::cards::ferramenta::{
    Numero, caixa, caixas_aos_pares, campo, cartao, opcoes,
};
use ph2d_editor_core::panel::cards::{CardDef, Cell, Widget, dropdown};
use ph2d_i18n::{tr, tr_with};
use ph2d_pixel_io::render::Quadros;
use ph2d_pixel_io::sheet::{Limite, Tipo};

use crate::canal::{Contexto, Imagem};
use crate::comum::{
    ImagemDoCartao, Linhas, imagem_no_cartao, linhas_de_camadas, linhas_de_quadros, listas,
};
use crate::ids;
use crate::state::OpcoesFo;

pub(crate) const TIPOS: [Tipo; 5] = [
    Tipo::Horizontal,
    Tipo::Vertical,
    Tipo::Rows,
    Tipo::Columns,
    Tipo::Packed,
];

fn linhas_de_tipo(t: Tipo) -> Linhas {
    const NOMES: [&str; 5] = [
        "panel.pixel.export.type_horizontal",
        "panel.pixel.export.type_vertical",
        "panel.pixel.export.type_rows",
        "panel.pixel.export.type_columns",
        "panel.pixel.export.type_packed",
    ];
    TIPOS
        .iter()
        .enumerate()
        .map(|(i, x)| (ids::SHEET_TYPES[i], tr(NOMES[i]).to_owned(), *x == t))
        .collect()
}

/// Os limites que cada tipo aceita (o diálogo do Aseprite, foto 31); as tiras não têm nenhum.
pub(crate) fn limites_do_tipo(t: Tipo) -> &'static [Limite] {
    const N: u32 = 0;
    match t {
        Tipo::Horizontal | Tipo::Vertical => &[Limite::Nenhum],
        Tipo::Rows => &[Limite::Nenhum, Limite::Colunas(N), Limite::Largura(N)],
        Tipo::Columns => &[Limite::Nenhum, Limite::Linhas(N), Limite::Altura(N)],
        Tipo::Packed => &[Limite::Nenhum, Limite::Largura(N), Limite::Altura(N)],
    }
}

pub(crate) const fn id_do_limite(l: Limite) -> ph2d_a11y::NodeId {
    match l {
        Limite::Nenhum => ids::LIMIT_NONE,
        Limite::Colunas(_) => ids::LIMIT_COLUMNS,
        Limite::Linhas(_) => ids::LIMIT_ROWS,
        Limite::Largura(_) => ids::LIMIT_WIDTH,
        Limite::Altura(_) => ids::LIMIT_HEIGHT,
    }
}

pub(crate) fn limite_de(id: ph2d_a11y::NodeId, n: u32) -> Option<Limite> {
    [
        (ids::LIMIT_NONE, Limite::Nenhum),
        (ids::LIMIT_COLUMNS, Limite::Colunas(n)),
        (ids::LIMIT_ROWS, Limite::Linhas(n)),
        (ids::LIMIT_WIDTH, Limite::Largura(n)),
        (ids::LIMIT_HEIGHT, Limite::Altura(n)),
    ]
    .into_iter()
    .find_map(|(i, l)| (i == id).then_some(l))
}

fn linhas_de_limite(t: Tipo, l: Limite) -> Linhas {
    limites_do_tipo(t)
        .iter()
        .map(|x| {
            let nome = match x {
                Limite::Nenhum => "panel.pixel.export.limit_none",
                Limite::Colunas(_) => "panel.pixel.export.limit_columns",
                Limite::Linhas(_) => "panel.pixel.export.limit_rows",
                Limite::Largura(_) => "panel.pixel.export.limit_width",
                Limite::Altura(_) => "panel.pixel.export.limit_height",
            };
            let id = id_do_limite(*x);
            (id, tr(nome).to_owned(), id == id_do_limite(l))
        })
        .collect()
}

pub(crate) fn numeros(o: &OpcoesFo, x: &Contexto) -> Vec<Numero> {
    let n = |chip, fabrica: f64, valor: u32| Numero {
        chip,
        step: 1.0,
        scrub: 1.0,
        fabrica,
        valor: f64::from(valor),
    };
    let ultimo = x.quadros.max(1);
    let (de, ate) = match o.folha.quadros {
        Quadros::Intervalo { de, ate } => (de as u32 + 1, ate as u32 + 1),
        _ => (1, ultimo),
    };
    vec![
        n(
            ids::LIMIT_N,
            f64::from(OpcoesFo::default().limite_n),
            o.limite_n,
        ),
        n(ids::FROM, 1.0, de),
        n(ids::TO, f64::from(ultimo), ate),
        n(ids::BORDER, 0.0, o.folha.border),
        n(ids::SHAPE, 0.0, o.folha.shape),
        n(ids::INNER, 0.0, o.folha.inner),
        n(ids::EXTRUDE, 0.0, o.folha.extrude),
    ]
}

pub(crate) fn prepare() {
    for caixa in [ids::SHEET_TYPE, ids::LIMIT, ids::LAYERS, ids::FRAMES] {
        dropdown::declare(caixa);
    }
}

/// O cartão da folha ao vivo.
fn cartao_da_previa(img: Option<(ImagemDoCartao, Imagem)>) -> CardDef<'static> {
    cartao(
        ids::SEC_PREVIEW,
        tr("panel.pixel.export.sec_preview"),
        None,
        move |c, y| {
            let Some((i, p)) = &img else {
                return c.hint(tr("panel.pixel.export.preview_wait"), y);
            };
            let y = imagem_no_cartao(c, i, None, y);
            let t = tr_with(
                "panel.pixel.export.preview_size",
                &[
                    ("w", &p.lados_reais.0.to_string()),
                    ("h", &p.lados_reais.1.to_string()),
                    ("n", &p.entradas.to_string()),
                ],
            );
            c.hint(&t, y)
        },
    )
}

pub(crate) fn cards(
    o: &OpcoesFo,
    x: &Contexto,
    img: Option<(ImagemDoCartao, Imagem)>,
) -> Vec<CardDef<'static>> {
    let f = o.folha.clone();
    let tipos = linhas_de_tipo(f.tipo);
    let limites = linhas_de_limite(f.tipo, f.limite);
    let camadas = linhas_de_camadas(x, f.camadas);
    let quadros = linhas_de_quadros(x, f.quadros);
    let intervalo = matches!(f.quadros, Quadros::Intervalo { .. });
    let tem_limite = f.limite != Limite::Nenhum;
    let tem_lista_de_limite = limites.len() > 1;
    let tem_tiles = !x.tilesets.is_empty();
    let tem_slices = !x.slices.is_empty();
    let (json_ligado, json) = (o.json_ligado, o.json);
    vec![
        cartao_da_previa(img),
        cartao(
            ids::SEC_LAYOUT,
            tr("panel.pixel.export.sec_layout"),
            None,
            move |c, y| {
                let lim = tem_lista_de_limite.then_some((
                    ids::LIMIT,
                    tr("panel.pixel.export.limit"),
                    &limites,
                ));
                let y = listas(
                    c,
                    (ids::SHEET_TYPE, tr("panel.pixel.export.sheet_type"), &tipos),
                    lim,
                    y,
                );
                let y = if tem_limite {
                    c.row(&[campo(ids::LIMIT_N, tr("panel.pixel.export.limit_n"))], y)
                } else {
                    y
                };
                caixas_aos_pares(
                    c,
                    &[
                        (ids::MERGE, "panel.pixel.export.merge", f.merge_duplicates),
                        (
                            ids::IGNORE_EMPTY,
                            "panel.pixel.export.ignore_empty",
                            f.ignore_empty,
                        ),
                        (ids::POW2, "panel.pixel.export.pow2", f.potencia_de_2),
                    ],
                    y,
                )
            },
        ),
        cartao(
            ids::SEC_SPRITE,
            tr("panel.pixel.export.sec_sprite"),
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
                let mut caixas = vec![
                    (
                        ids::SPLIT_LAYERS,
                        "panel.pixel.export.split_layers",
                        f.split_layers,
                    ),
                    (
                        ids::SPLIT_TAGS,
                        "panel.pixel.export.split_tags",
                        f.split_tags,
                    ),
                ];
                if tem_tiles {
                    caixas.push((
                        ids::SPLIT_GRID,
                        "panel.pixel.export.split_grid",
                        f.split_grid,
                    ));
                }
                if tem_slices {
                    caixas.push((
                        ids::SPLIT_SLICES,
                        "panel.pixel.export.split_slices",
                        f.split_slices,
                    ));
                }
                caixas_aos_pares(c, &caixas, y)
            },
        ),
        cartao(
            ids::SEC_BORDERS,
            tr("panel.pixel.export.sec_borders"),
            None,
            move |c, y| {
                let y = c.pair(
                    campo(ids::BORDER, tr("panel.pixel.export.border")),
                    campo(ids::SHAPE, tr("panel.pixel.export.shape")),
                    y,
                );
                let y = c.pair(
                    campo(ids::INNER, tr("panel.pixel.export.inner")),
                    campo(ids::EXTRUDE, tr("panel.pixel.export.extrude")),
                    y,
                );
                let mut caixas = vec![
                    (ids::TRIM_CELS, "panel.pixel.export.trim_cels", f.trim_cels),
                    (
                        ids::TRIM_SPRITE,
                        "panel.pixel.export.trim_sprite",
                        f.trim_sprite,
                    ),
                ];
                if tem_tiles && (f.trim_cels || f.trim_sprite) {
                    caixas.push((
                        ids::TRIM_BY_GRID,
                        "panel.pixel.export.trim_by_grid",
                        f.trim_by_grid,
                    ));
                }
                caixas_aos_pares(c, &caixas, y)
            },
        ),
        cartao(
            ids::SEC_OUTPUT,
            tr("panel.pixel.export.sec_output"),
            None,
            move |c, y| {
                let y = caixa(c, ids::JSON, "panel.pixel.export.json", json_ligado, y);
                if !json_ligado {
                    return y;
                }
                let segs = opcoes(
                    &[
                        (ids::JSON_HASH, tr("panel.pixel.export.json_hash")),
                        (ids::JSON_ARRAY, tr("panel.pixel.export.json_array")),
                    ],
                    usize::from(json.array),
                );
                let y = c.full(
                    Some(tr("panel.pixel.export.json_format")),
                    Widget::Choice(&segs),
                    y,
                );
                let y = caixas_aos_pares(
                    c,
                    &[
                        (ids::JSON_TAGS, "panel.pixel.export.json_tags", json.tags),
                        (
                            ids::JSON_LAYERS,
                            "panel.pixel.export.json_layers",
                            json.layers,
                        ),
                        (
                            ids::JSON_SLICES,
                            "panel.pixel.export.json_slices",
                            json.slices,
                        ),
                    ],
                    y,
                );
                c.row(
                    &[Cell::new(Widget::Text {
                        id: ids::ITEM_NAME,
                        placeholder: tr("panel.pixel.export.item_name_hint"),
                    })
                    .label(tr("panel.pixel.export.item_name"))],
                    y,
                )
            },
        ),
    ]
}
