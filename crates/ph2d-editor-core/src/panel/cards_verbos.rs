//! ⭐ **Os verbos de um cartão**: botões baixos, sempre lado a lado e da mesma largura, com dois degraus
//! escolhidos pela largura MEDIDA do texto (dono, 05/11, cartão da miniatura da Sprite):
//! `[seta] Nome` ou, sem caber, `[seta] [ícone]`. Todos trocam juntos; nunca reticências, nunca empilhados.

use super::{CardCtx, censo, dicas};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_elided};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken};
use std::cell::RefCell;

/// A altura de um verbo: a caixa de um ícone de cabeçalho, mais baixa que um campo.
pub const VERBO_H: f32 = m::HEAD_ICON_BOX;
const VERBO_GLIFO: f32 = m::HEAD_ICON_GLYPH;

/// Um botão de verbo: a ação (`seta`) e o destino, por extenso (`nome`) ou em ícone (`icone`).
#[derive(Copy, Clone, Debug)]
pub struct Verbo<'a> {
    pub id: NodeId,
    pub seta: IconId,
    pub icone: IconId,
    pub nome: &'a str,
}

thread_local! {
    static DEGRAUS: RefCell<Vec<(NodeId, bool)>> = const { RefCell::new(Vec::new()) };
}

/// `(id, com nome)` de cada verbo da última fila pintada. Para os testes dos degraus.
#[doc(hidden)]
#[must_use]
pub fn degraus_pintados() -> Vec<(NodeId, bool)> {
    DEGRAUS.with(|d| d.borrow().clone())
}

impl CardCtx<'_> {
    /// A largura de `v` com o nome inteiro.
    fn verbo_pede(&mut self, v: &Verbo<'_>) -> f32 {
        let tw = self.text_system.prefix_width(v.nome, m::VALUE_PX).ceil();
        VERBO_GLIFO + Spacing::Xs.px() + tw + m::BUTTON_TEXT_PAD * 2.0
    }

    /// A fila em colunas iguais a partir de `topo`, na largura do contexto. Devolve o fundo.
    pub fn verbos(&mut self, verbos: &[Verbo<'_>], topo: f32) -> f32 {
        self.first = false;
        let vao = Spacing::Xs.px();
        let n = verbos.len().max(1) as f32;
        let w = (self.w - vao * (n - 1.0)) / n;
        let com_nome = verbos.iter().all(|v| self.verbo_pede(v) <= w);
        DEGRAUS.with(|d| d.borrow_mut().clear());
        for (i, v) in verbos.iter().enumerate() {
            let r = Rect::new(self.x + (w + vao) * i as f32, topo, w, VERBO_H);
            self.verbo(r, v, com_nome);
            DEGRAUS.with(|d| d.borrow_mut().push((v.id, com_nome)));
        }
        topo + VERBO_H
    }

    fn verbo(&mut self, r: Rect, v: &Verbo<'_>, com_nome: bool) {
        self.hit_index.register(v.id, r);
        let hover = self.store.hover_live(v.id) > 0.0;
        let fundo = if hover {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        let tinta = self.ink.color(ColorToken::Text1);
        let vao = Spacing::Xs.px();
        let tw = if com_nome {
            if censo::ligado() {
                let pede = self.verbo_pede(v);
                censo::texto(v.nome, pede, r.w);
            }
            self.text_system.prefix_width(v.nome, m::VALUE_PX)
        } else {
            dicas::icone(v.id);
            VERBO_GLIFO
        };
        let mut x = r.x + (r.w - (VERBO_GLIFO + vao + tw)) * 0.5;
        let y = r.y + (r.h - VERBO_GLIFO) * 0.5;
        let traco = StrokeToken::Default.px();
        let glifo = |x: f32| Rect::new(x, y, VERBO_GLIFO, VERBO_GLIFO);
        paint_icon(self.scene, v.seta, glifo(x), tinta, traco);
        x += VERBO_GLIFO + vao;
        if com_nome {
            paint_text_elided(
                self.text_system,
                self.scene,
                v.nome,
                x,
                r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
                m::VALUE_PX,
                (r.x + r.w - x).max(0.0),
                tinta,
            );
        } else {
            paint_icon(self.scene, v.icone, glifo(x), tinta, traco);
        }
    }
}
