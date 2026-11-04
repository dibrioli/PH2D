//! As peças comuns das linhas de cartão do painel Painter (`docs/CartoesUI/13_PAINTER_PLANO.md`): as listas de caixa do
//! cartão, a linha esmaecida da lei, e os pares.

use crate::paint_wetpaint::Numero;
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, corpo as linhas};
use ph2d_editor_core::widget::DropdownState;
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;
use ph2d_vector::{Affine, Compose, Mix, RoundedRect, VelloBlend};

/// As listas destes cartões: a caixa é a do cartão, e a lista que abre é a de sempre (a do Preset arrasta linhas).
pub(crate) fn registra_lista(store: &mut WidgetStore, id: NodeId, atual: usize) {
    store.register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(atual),
        },
    );
}

pub(crate) fn aberta(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// Os itens de dois em dois, na mesma linha, cada um na sua metade; o que sobra fica com a linha inteira.
pub(crate) fn em_pares<T>(
    c: &mut CardCtx<'_>,
    itens: &[T],
    y: f32,
    mut pinta: impl FnMut(&mut CardCtx<'_>, &T, f32) -> f32,
) -> f32 {
    let (x, w) = (c.x, c.w);
    let meia = ((w - m::FIELD_GAP) * 0.5).max(0.0);
    let mut topo = y;
    for par in itens.chunks(2) {
        let mut fundo = topo;
        for (i, item) in par.iter().enumerate() {
            (c.x, c.w) = if par.len() == 2 {
                (x + i as f32 * (meia + m::FIELD_GAP), meia)
            } else {
                (x, w)
            };
            fundo = fundo.max(pinta(c, item, topo));
        }
        topo = fundo;
    }
    c.x = x;
    c.w = w;
    topo
}

/// Quantos por linha: o maior número em que toda largura pedida cabe na coluna (05 §4, lei 7). Pelo menos um.
pub(crate) fn por_linha(w: f32, larguras: &[f32]) -> usize {
    (1..=larguras.len())
        .rev()
        .find(|n| {
            let vaos = m::FIELD_GAP * (*n as f32 - 1.0);
            let coluna = (w - vaos) / *n as f32;
            larguras.iter().all(|l| *l <= coluna)
        })
        .unwrap_or(1)
}

/// Botões de acção que nunca cortam o nome: se não cabem todos na linha, descem para mais linhas.
pub(crate) fn botoes<S: AsRef<str>>(c: &mut CardCtx<'_>, lista: &[(NodeId, S)], y: f32) -> f32 {
    let larguras: Vec<f32> = lista
        .iter()
        .map(|(_, t)| c.text_system.prefix_width(t.as_ref(), m::VALUE_PX) + 2.0 * m::FIELD_PAD_R)
        .collect();
    let n = por_linha(c.w, &larguras).max(1);
    lista.chunks(n).fold(y, |y, fatia| {
        let cells: Vec<Cell<'_>> = fatia
            .iter()
            .map(|(id, text)| {
                Cell::new(Widget::Button {
                    id: *id,
                    text: text.as_ref(),
                    primary: false,
                })
            })
            .collect();
        c.row(&cells, y)
    })
}

/// Um número das tabelas destes cartões.
pub(crate) fn numero(
    chave: &'static str,
    id: NodeId,
    valor: f32,
    (min, max): (f32, f32),
    passo: f64,
    casas: usize,
) -> Numero {
    Numero {
        chave,
        id,
        valor,
        min,
        max,
        passo,
        casas,
    }
}

/// A linha de grupo: um filete e o nome. A lei é «nunca um 2.º título», por isso o tom é o dos rótulos.
pub(crate) fn grupo(c: &mut CardCtx<'_>, chave: &str, y: f32) -> f32 {
    let filete = Rect::new(c.x, y + m::CARD_HEAD_BOTTOM, c.w, m::FIELD_BORDER);
    let cor = c.ink.color(ColorToken::CardField);
    fill_rounded_rect(c.scene, filete, 0.0, cor);
    c.hint(tr(chave), filete.y + filete.h)
}

/// O grupo que abre o corpo: por cima dele já está o cabeçalho, e o filete sobrava.
pub(crate) fn grupo_inicial(c: &mut CardCtx<'_>, chave: &str, y: f32) -> f32 {
    c.hint(tr(chave), y - m::FIELD_ROW_GAP)
}

/// O véu da opção que a ferramenta diz inerte: continua clicável, como a linha esmaecida.
fn opcoes_inertes(c: &mut CardCtx<'_>, segs: &[Seg<'_>]) {
    let inercias = state::current_inercias();
    let alfa = ph2d_editor_core::property_row::inativa::alfa_inativo(c.ink.theme);
    for s in segs {
        if inercias.dica_de(s.id).is_some()
            && let Some(r) = c.hit_index.rect_for(s.id)
        {
            let veu = c
                .ink
                .color(ColorToken::CardField)
                .multiply_alpha(1.0 - alfa);
            fill_rounded_rect(c.scene, r, m::SEG_RADIUS, veu);
        }
    }
}

/// Uma escolha com rótulo.
pub(crate) fn escolha(c: &mut CardCtx<'_>, rotulo: &str, segs: &[Seg<'_>], y: f32) -> f32 {
    let fim = c.full(Some(rotulo), Widget::Choice(segs), y);
    opcoes_inertes(c, segs);
    fim
}

/// Uma escolha longa: desce em linhas de pílula, com o mesmo número por linha, em vez de cortar os nomes (lei 7).
pub(crate) fn escolha_em_linhas(
    c: &mut CardCtx<'_>,
    rotulo: &str,
    segs: &[Seg<'_>],
    y: f32,
) -> f32 {
    let larguras: Vec<f32> = segs
        .iter()
        .map(|s| {
            c.text_system.prefix_width(s.text, m::SEG_TEXT_PX) + 2.0 * m::SEG_INSET + m::SEG_PAD
        })
        .collect();
    let n = por_linha(c.w - 2.0 * m::SEG_PAD, &larguras).max(1);
    let mut y = y;
    for (i, fatia) in segs.chunks(n).enumerate() {
        let nome = (i == 0).then_some(rotulo);
        y = c.full(nome, Widget::Choice(fatia), y);
        opcoes_inertes(c, fatia);
    }
    y
}

/// Um número e a cor irmã na mesma linha (Wax + Wax Color, Intensity + Color).
pub(crate) fn numero_e_cor(
    c: &mut CardCtx<'_>,
    n: &Numero,
    chave_da_cor: &'static str,
    id_da_cor: NodeId,
    y: f32,
) -> f32 {
    let [r, g, b, a] = c
        .store
        .widget_color(id_da_cor)
        .unwrap_or([0, 0, 0, u8::MAX]);
    let codigo = format!("{r:02X}{g:02X}{b:02X}");
    let rotulo = tr(n.chave);
    let cells = [
        Cell::new(Widget::Field {
            id: n.id,
            glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
        })
        .label(rotulo)
        .key(std::slice::from_ref(&n.id)),
        Cell::new(Widget::Swatch {
            id: id_da_cor,
            rgba: Some([r, g, b, a]),
            text: &codigo,
            edit: None,
            detach: None,
        })
        .label(tr(chave_da_cor))
        .key(std::slice::from_ref(&id_da_cor)),
    ];
    c.row(&cells, y)
}

/// A altura de uma linha com rótulo, de sobra: a camada só recorta, nunca empurra.
fn altura_de_linha() -> f32 {
    m::LABEL_TOP + 2.0 * (m::LABEL_PX + m::LABEL_BOTTOM) + m::FIELD_ROW_GAP + m::FIELD_H
}

/// Esmaecida como a linha de antes: continua editável, salvo a do Color inativo, que o chamador tira do índice.
pub(crate) fn esmaecida(
    c: &mut CardCtx<'_>,
    y: f32,
    pintar: impl FnOnce(&mut CardCtx<'_>, f32) -> f32,
) -> f32 {
    let forma = RoundedRect::new(
        f64::from(c.x),
        f64::from(y),
        f64::from(c.x + c.w),
        f64::from(y + altura_de_linha()),
        0.0,
    );
    c.scene.push_layer_shape(
        VelloBlend::new(Mix::Normal, Compose::SrcOver),
        ph2d_editor_core::property_row::inativa::alfa_inativo(c.ink.theme),
        Affine::IDENTITY,
        &forma,
    );
    let fim = pintar(c, y);
    c.scene.pop_layer();
    fim
}

/// A linha do id `id`, esmaecida se a ferramenta o diz inerte.
pub(crate) fn com_lei(
    c: &mut CardCtx<'_>,
    id: NodeId,
    y: f32,
    pintar: impl FnOnce(&mut CardCtx<'_>, f32) -> f32,
) -> f32 {
    if state::current_inercias().dica_de(id).is_some() {
        esmaecida(c, y, pintar)
    } else {
        pintar(c, y)
    }
}
