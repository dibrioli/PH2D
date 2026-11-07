//! Os cartões: a lista e as propriedades da slice.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::ferramenta::{caixa, cartao, opcoes};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_i18n::tr;
use ph2d_pixel_slices::campos::Vista;

use crate::canal::{Contexto, Linha};
use crate::ids;

/// O texto de um número: misto aparece vazio (§10).
pub(crate) fn valor(v: Option<i32>) -> String {
    v.map_or_else(String::new, |n| n.to_string())
}

/// O código de uma cor no cartão.
fn hex(c: [u8; 4]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

fn texto<'a>(id: NodeId, rotulo: &'a str) -> Cell<'a> {
    Cell::new(Widget::Text {
        id,
        placeholder: "",
    })
    .label(rotulo)
}

fn botao(id: NodeId, chave: &'static str) -> Cell<'static> {
    Cell::new(Widget::Button {
        id,
        text: tr(chave),
        primary: false,
    })
}

fn cartao_lista(linhas: Vec<Linha>, transform: bool) -> CardDef<'static> {
    cartao(
        ids::SEC_LIST,
        tr("panel.pixel.slices.sec_list"),
        None,
        move |c, mut y| {
            for (i, l) in linhas.iter().enumerate() {
                let o = opcoes(
                    &[(ids::linha(i), l.nome.as_str())],
                    usize::from(!l.selecionada),
                );
                y = c.full(None, Widget::Choice(&o), y);
            }
            if linhas.is_empty() {
                y = c.hint(tr("panel.pixel.slices.empty"), y);
            } else {
                y = c.row(
                    &[
                        botao(ids::ALL, "panel.pixel.slices.all"),
                        botao(ids::NONE, "panel.pixel.slices.none"),
                    ],
                    y,
                );
            }
            caixa(
                c,
                ids::TRANSFORM,
                "panel.pixel.slices.transform",
                transform,
                y,
            )
        },
    )
}

const fn rotulos_xywh() -> [&'static str; 4] {
    [
        "panel.pixel.slices.x",
        "panel.pixel.slices.y",
        "panel.pixel.slices.w",
        "panel.pixel.slices.h",
    ]
}

/// Quatro números na mesma linha (dono: irmãos dividem a linha).
fn linha_xywh(c: &mut CardCtx<'_>, campos: [NodeId; 4], y: f32) -> f32 {
    let r = rotulos_xywh();
    let cells = [
        texto(campos[0], tr(r[0])),
        texto(campos[1], tr(r[1])),
        texto(campos[2], tr(r[2])),
        texto(campos[3], tr(r[3])),
    ];
    c.row(&cells, y)
}

fn propriedades(c: &mut CardCtx<'_>, v: &Vista, y: f32) -> f32 {
    let y = c.row(&[texto(ids::SLICE_NAME, tr("panel.pixel.slices.name"))], y);
    let codigo = v.cor.map_or_else(String::new, hex);
    let y = c.row(
        &[
            Cell::new(Widget::Swatch {
                id: ids::COLOR,
                rgba: v.cor,
                text: &codigo,
                edit: None,
                detach: None,
            })
            .label(tr("panel.pixel.slices.color")),
            texto(ids::USER_DATA, tr("panel.pixel.slices.user_data")),
        ],
        y,
    );
    let y = c.hint(tr("panel.pixel.slices.bounds"), y);
    let mut y = linha_xywh(
        c,
        [ids::BOUNDS_X, ids::BOUNDS_Y, ids::BOUNDS_W, ids::BOUNDS_H],
        y,
    );
    y = caixa(c, ids::NINE, "panel.pixel.slices.nine", v.nove, y);
    if v.algum_nove {
        y = linha_xywh(
            c,
            [ids::CENTER_X, ids::CENTER_Y, ids::CENTER_W, ids::CENTER_H],
            y,
        );
    }
    y = caixa(c, ids::PIVOT, "panel.pixel.slices.pivot", v.pivot, y);
    if v.algum_pivot {
        y = c.row(
            &[
                texto(ids::PIVOT_X, tr("panel.pixel.slices.x")),
                texto(ids::PIVOT_Y, tr("panel.pixel.slices.y")),
            ],
            y,
        );
    }
    c.row(
        &[
            botao(ids::DUPLICATE, "panel.pixel.slices.duplicate"),
            botao(ids::DELETE, "panel.pixel.slices.delete"),
        ],
        y,
    )
}

/// Sem seleção: os mesmos rótulos, sem caixa nem clique, e a frase do que falta.
fn sem_selecao(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let vazio = tr("panel.pixel.slices.no_value");
    let facto = |rotulo| Cell::new(Widget::Readout { text: vazio }).label(rotulo);
    let y = c.hint(tr("panel.pixel.slices.no_selection"), y);
    let y = c.row(&[facto(tr("panel.pixel.slices.name"))], y);
    let r = rotulos_xywh();
    let y = c.row(
        &[
            facto(tr(r[0])),
            facto(tr(r[1])),
            facto(tr(r[2])),
            facto(tr(r[3])),
        ],
        y,
    );
    let apagado = |id, chave| {
        Cell::new(Widget::Check {
            id,
            label: tr(chave).into(),
            value: CheckboxValue::Unchecked,
            off: true,
        })
    };
    let y = c.row(
        &[
            apagado(ids::NINE_OFF, "panel.pixel.slices.nine"),
            apagado(ids::PIVOT_OFF, "panel.pixel.slices.pivot"),
        ],
        y,
    );
    c.row(
        &[
            Cell::new(Widget::ButtonOff {
                text: tr("panel.pixel.slices.duplicate"),
            }),
            Cell::new(Widget::ButtonOff {
                text: tr("panel.pixel.slices.delete"),
            }),
        ],
        y,
    )
}

/// Os cartões da janela.
pub(crate) fn cartoes(x: &Contexto) -> Vec<CardDef<'static>> {
    let vista = x.vista.clone();
    vec![
        cartao_lista(x.linhas.clone(), x.transform),
        cartao(
            ids::SEC_PROPS,
            tr("panel.pixel.slices.sec_props"),
            None,
            move |c, y| match &vista {
                Some(v) => propriedades(c, v, y),
                None => sem_selecao(c, y),
            },
        ),
    ]
}
