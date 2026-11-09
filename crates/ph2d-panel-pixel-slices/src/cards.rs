//! Os cartões: a lista e as propriedades da slice.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::ferramenta::{caixa, cartao, opcoes};
use ph2d_editor_core::panel::cards::user_data;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::pixel_panel::user_data::UserDataView;
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

/// O texto da lista vazia: sem Pixel Sprite em Draw não há Slice Tool a pegar (foto `P12_fecho_05`).
pub(crate) fn vazio(x: &Contexto) -> &'static str {
    match (x.com_sprite, x.na_mao) {
        (false, _) => "panel.pixel.slices.no_document",
        (true, true) => "panel.pixel.slices.empty",
        (true, false) => "panel.pixel.slices.empty_tool",
    }
}

fn cartao_lista(linhas: Vec<Linha>, transform: bool, vazio: &'static str) -> CardDef<'static> {
    cartao(
        ids::SLICES_SEC_LIST,
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
                y = c.hint(tr(vazio), y);
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
    collider: Option<bool>,
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
    let y = c.row(
        &[
            botao(ids::DUPLICATE, "panel.pixel.slices.duplicate"),
            botao(ids::DELETE, "panel.pixel.slices.delete"),
        ],
        y,
    );
    match collider {
        Some(false) => c.row(
            &[botao(ids::ADD_COLLIDER, "panel.pixel.slices.add_collider")],
            y,
        ),
        Some(true) => c.row(
            &[Cell::new(Widget::ButtonOff {
                text: tr("panel.pixel.slices.collider_added"),
            })],
            y,
        ),
        None => y,
    }
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
    let (vista, collider) = (x.vista.clone(), x.collider);
    let user = x
        .user
        .as_ref()
        .map(|(_, u)| (ph2d_pixel_ponte::user::vista(u), user_aberto));
    vec![
        cartao_lista(x.linhas.clone(), x.transform, vazio(x)),
        cartao(
            ids::SEC_PROPS,
            tr("panel.pixel.slices.sec_props"),
            None,
            move |c, y| match &vista {
                Some(v) => propriedades(c, v, user.as_ref(), collider, y),
                None => sem_selecao(c, y),
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::vazio;
    use crate::canal::Contexto;

    /// ⭐ Foto `P12_fecho_05`: sem Pixel Sprite em Draw a lista vazia mandava pegar a Slice Tool, que
    /// fora do Draw não existe. Cada estado tem o seu texto, e todos estão na tabela.
    #[test]
    fn a_lista_vazia_diz_o_que_fazer_em_cada_estado() {
        let com = |na_mao| Contexto {
            com_sprite: true,
            na_mao,
            ..Contexto::default()
        };
        let sem = Contexto::default();
        let sem_na_mao = Contexto {
            na_mao: true,
            ..Contexto::default()
        };
        assert_eq!(vazio(&sem), "panel.pixel.slices.no_document");
        assert_eq!(vazio(&sem_na_mao), "panel.pixel.slices.no_document");
        assert_eq!(vazio(&com(false)), "panel.pixel.slices.empty_tool");
        assert_eq!(vazio(&com(true)), "panel.pixel.slices.empty");
        for x in [sem, com(false), com(true)] {
            let k = vazio(&x);
            assert_ne!(ph2d_i18n::tr(k), k, "{k} sem texto");
        }
    }
}
