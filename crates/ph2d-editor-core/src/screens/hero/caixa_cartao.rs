//! O corpo das caixas de diálogo internas em linhas de cartão (`docs/CartoesUI/15_CHROME_CENSO.md`, M8):
//! título, rótulo em cima, caixas `FIELD_H`, irmãos na mesma linha, os mesmos ids. A altura sai da
//! função que pinta, num ensaio descartável: uma conta só. `PH2D_CARDS=0` não passa por aqui.

use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::paint_text;
use crate::panel::cards::{CardCtx, Cell, Seg, Widget};
use crate::widget::card_ink::CardInk;
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

/// O que distingue uma caixa centrada da outra, além do corpo.
pub(super) struct Molde<'t> {
    pub largura: f32,
    pub titulo: &'t str,
}

/// O que todo pintor de caixa recebe.
pub(super) struct Alvo<'a> {
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
    pub hit_index: &'a mut HitIndex,
    pub store: &'a WidgetStore,
    pub viewport: Rect,
}

/// O título: a linha de cima da caixa. Devolve o fim dele.
pub(super) fn titulo(c: &mut CardCtx<'_>, texto: &str, y: f32) -> f32 {
    paint_text(
        c.text_system,
        c.scene,
        texto,
        c.x,
        y + crate::paint::folga_do_texto(m::FIELD_H, m::VALUE_PX),
        m::VALUE_PX,
        c.w,
        c.ink.color(ColorToken::KanbanCardFg),
    );
    y + m::FIELD_H
}

fn passa(
    a: &mut Alvo<'_>,
    ink: CardInk,
    (x, w, topo): (f32, f32, f32),
    texto: &str,
    corpo: &impl Fn(&mut CardCtx<'_>, f32) -> f32,
) -> f32 {
    let mut c = CardCtx::flutuante(
        &mut *a.scene,
        &mut *a.text_system,
        a.store,
        &mut *a.hit_index,
        ink,
        x,
        w,
    );
    let y = titulo(&mut c, texto, topo + m::CARD_PAD_TOP);
    corpo(&mut c, y)
}

/// Pinta a caixa centrada no viewport. `false` no modo de antes: quem chama pinta a dele.
pub(super) fn centrada(
    molde: Molde<'_>,
    superficie: Superficie,
    a: &mut Alvo<'_>,
    corpo: impl Fn(&mut CardCtx<'_>, f32) -> f32,
) -> bool {
    let Some(ink) = superficie.tinta() else {
        return false;
    };
    let _tinta = superficie.veste();
    let w = molde.largura - m::CARD_PAD_X * 2.0;
    let (mut ensaio, mut alvos) = (VectorScene::new(), HitIndex::default());
    let fim = passa(
        &mut Alvo {
            scene: &mut ensaio,
            text_system: &mut *a.text_system,
            hit_index: &mut alvos,
            store: a.store,
            viewport: a.viewport,
        },
        ink,
        (m::CARD_PAD_X, w, 0.0),
        molde.titulo,
        &corpo,
    );
    let h = fim + m::CARD_PAD_BOTTOM;
    let v = a.viewport;
    let x = (v.x + (v.w - molde.largura) * 0.5).max(v.x);
    let y = (v.y + (v.h - h) * 0.5).max(v.y);
    superficie.moldura_de_caixa(a.scene, Rect::new(x, y, molde.largura, h));
    passa(a, ink, (x + m::CARD_PAD_X, w, y), molde.titulo, &corpo);
    true
}

/// A largura da caixa em que cada fila de `filas` cabe numa linha, nunca abaixo de `minimo`: derivada do texto medido.
pub(super) fn largura_das_filas(
    a: &mut Alvo<'_>,
    ink: CardInk,
    minimo: f32,
    filas: &[&[Seg<'_>]],
) -> f32 {
    let (mut ensaio, mut alvos) = (VectorScene::new(), HitIndex::default());
    let mut c = CardCtx::flutuante(
        &mut ensaio,
        &mut *a.text_system,
        a.store,
        &mut alvos,
        ink,
        0.0,
        minimo,
    );
    filas
        .iter()
        .map(|f| c.escolha_pede(f).ceil() + m::CARD_PAD_X * 2.0)
        .fold(minimo, f32::max)
}

/// As linhas `(id, texto, escolhida)` como segmentos.
pub(super) fn segs<'t>(linhas: &[(NodeId, &'t str, bool)]) -> Vec<Seg<'t>> {
    linhas
        .iter()
        .map(|&(id, text, on)| Seg {
            id,
            icon: None,
            text,
            on,
        })
        .collect()
}

/// `Cancel | OK`, na mesma linha.
pub(super) fn par(
    c: &mut CardCtx<'_>,
    (cancela, cancela_texto): (NodeId, &str),
    (ok, ok_texto): (NodeId, &str),
    y: f32,
) -> f32 {
    c.row(
        &[
            Cell::new(Widget::Button {
                id: cancela,
                text: cancela_texto,
                primary: false,
            }),
            Cell::new(Widget::Button {
                id: ok,
                text: ok_texto,
                primary: true,
            }),
        ],
        y,
    )
}

/// O botão de destaque, na linha inteira.
pub(super) fn acao(c: &mut CardCtx<'_>, id: NodeId, text: &str, y: f32) -> f32 {
    c.full(
        None,
        Widget::Button {
            id,
            text,
            primary: true,
        },
        y,
    )
}

#[cfg(test)]
#[path = "caixa_cartao_tests.rs"]
mod tests;
