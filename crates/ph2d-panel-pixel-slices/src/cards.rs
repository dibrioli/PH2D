//! Os cartões: a lista e as propriedades da slice.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::ferramenta::{caixa, cartao, opcoes};
use ph2d_editor_core::panel::cards::user_data::{self, UserDataView};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::tr;
use ph2d_pixel_slices::campos::Vista;

use crate::canal::{Contexto, Linha};
use crate::ids;

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

/// O número à Figma: a letra mora dentro da caixa, e arrastá-la muda o número.
fn numero(id: NodeId, chave: &'static str) -> Cell<'static> {
    Cell::new(Widget::Field {
        id,
        glyph: Glyph::Text(tr(chave)),
    })
}

/// Os quatro números em duas linhas: `X` `Y`, depois `W` `H` (dono 08/11).
/// `rotulo` mora sobre a primeira caixa, à distância do rótulo (lei do ritmo), nunca como frase solta.
fn linha_xywh(c: &mut CardCtx<'_>, campos: [NodeId; 4], rotulo: Option<&str>, y: f32) -> f32 {
    let r = rotulos_xywh();
    let x = numero(campos[0], r[0]);
    let x = match rotulo {
        Some(t) => x.label(t),
        None => x,
    };
    let y = c.row(&[x, numero(campos[1], r[1])], y);
    c.row(&[numero(campos[2], r[2]), numero(campos[3], r[3])], y)
}

/// A caixa de uma propriedade que nem todas as selecionadas têm: marcada (todas), traço (algumas,
/// misto) ou vazia (nenhuma). Clicar a mista liga em todas, como no Figma e no Aseprite.
fn marca(id: NodeId, chave: &'static str, todas: bool, alguma: bool) -> Cell<'static> {
    let value = match (todas, alguma) {
        (true, _) => CheckboxValue::Checked,
        (false, true) => CheckboxValue::Indeterminate,
        (false, false) => CheckboxValue::Unchecked,
    };
    Cell::new(Widget::Check {
        id,
        label: Rotulo::vista(TextKey::new(chave)),
        value,
        off: false,
    })
}

fn propriedades(
    c: &mut CardCtx<'_>,
    v: &Vista,
    user: Option<&(UserDataView, bool)>,
    y: f32,
) -> f32 {
    let ud = ids::user_data();
    let nome = texto(ids::SLICE_NAME, tr("panel.pixel.slices.name"));
    let y = match user {
        Some(_) => c.row(&[nome, user_data::toggle_cell(&ud)], y),
        None => c.row(&[nome], y),
    };
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
    let y = match user {
        Some((vista, true)) => user_data::paint_body(c, &ud, vista, false, y),
        _ => y,
    };
    let mut y = linha_xywh(
        c,
        [ids::BOUNDS_X, ids::BOUNDS_Y, ids::BOUNDS_W, ids::BOUNDS_H],
        Some(tr("panel.pixel.slices.bounds")),
        y,
    );
    // Sem campos no meio, as duas caixas dividem a linha.
    let nove = marca(ids::NINE, "panel.pixel.slices.nine", v.nove, v.algum_nove);
    let pivot = marca(
        ids::PIVOT,
        "panel.pixel.slices.pivot",
        v.pivot,
        v.algum_pivot,
    );
    if v.algum_nove {
        y = c.row(&[nove], y);
        y = linha_xywh(
            c,
            [ids::CENTER_X, ids::CENTER_Y, ids::CENTER_W, ids::CENTER_H],
            None,
            y,
        );
        y = c.row(&[pivot], y);
    } else {
        y = c.compact(&[nove, pivot], y);
    }
    if v.algum_pivot {
        let r = rotulos_xywh();
        y = c.row(&[numero(ids::PIVOT_X, r[0]), numero(ids::PIVOT_Y, r[1])], y);
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
    let y = c.row(&[facto(tr(r[0])), facto(tr(r[1]))], y);
    let y = c.row(&[facto(tr(r[2])), facto(tr(r[3]))], y);
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
pub(crate) fn cartoes(x: &Contexto, user_aberto: bool) -> Vec<CardDef<'static>> {
    let vista = x.vista.clone();
    let user = x
        .user
        .as_ref()
        .map(|(_, u)| (ph2d_pixel_ponte::user::vista(u), user_aberto));
    vec![
        cartao_lista(x.linhas.clone(), x.transform),
        cartao(
            ids::SEC_PROPS,
            tr("panel.pixel.slices.sec_props"),
            None,
            move |c, y| match &vista {
                Some(v) => propriedades(c, v, user.as_ref(), y),
                None => sem_selecao(c, y),
            },
        ),
    ]
}
