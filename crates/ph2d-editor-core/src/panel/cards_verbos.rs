//! ⭐ **Os verbos de um cartão**: botões baixos `[seta] Nome`, nunca com reticências (dono, 05/11, cartão
//! da miniatura da Sprite): lado a lado, da mesma largura, ou empilhados na largura da coluna. Quem pinta
//! escolhe o arranjo pela largura MEDIDA do texto ([`CardCtx::verbos_cabem_lado_a_lado`]).

use super::{CardCtx, censo};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_elided};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken};

/// A altura de um verbo: a caixa de um ícone de cabeçalho, mais baixa que um campo.
pub const VERBO_H: f32 = m::HEAD_ICON_BOX;
const VERBO_GLIFO: f32 = m::HEAD_ICON_GLYPH;

/// O vão entre dois verbos, nos dois arranjos.
fn vao_entre() -> f32 {
    Spacing::Xs.px()
}

/// Um botão de verbo: a ação (`seta`) e o destino.
#[derive(Copy, Clone, Debug)]
pub struct Verbo<'a> {
    pub id: NodeId,
    pub seta: IconId,
    pub nome: &'a str,
}

impl CardCtx<'_> {
    /// A largura de `v` com o nome inteiro.
    pub fn verbo_pede(&mut self, v: &Verbo<'_>) -> f32 {
        let tw = self.text_system.prefix_width(v.nome, m::VALUE_PX).ceil();
        VERBO_GLIFO + Spacing::Xs.px() + tw + m::BUTTON_TEXT_PAD * 2.0
    }

    /// Os `verbos` inteiros cabem em colunas iguais na largura `w`?
    pub fn verbos_cabem_lado_a_lado(&mut self, verbos: &[Verbo<'_>], w: f32) -> bool {
        let n = verbos.len().max(1) as f32;
        let coluna = (w - vao_entre() * (n - 1.0)) / n;
        verbos.iter().all(|v| self.verbo_pede(v) <= coluna)
    }

    /// A fila em colunas iguais a partir de `topo`, na largura do contexto.
    pub fn verbos_lado_a_lado(&mut self, verbos: &[Verbo<'_>], topo: f32) {
        self.first = false;
        let n = verbos.len().max(1) as f32;
        let w = (self.w - vao_entre() * (n - 1.0)) / n;
        for (i, v) in verbos.iter().enumerate() {
            let r = Rect::new(self.x + (w + vao_entre()) * i as f32, topo, w, VERBO_H);
            self.verbo(r, v);
        }
    }

    /// A pilha na largura do contexto, a acabar em `fundo`.
    pub fn verbos_em_pilha(&mut self, verbos: &[Verbo<'_>], fundo: f32) {
        self.first = false;
        let n = verbos.len() as f32;
        let topo = fundo - VERBO_H * n - vao_entre() * (n - 1.0).max(0.0);
        for (i, v) in verbos.iter().enumerate() {
            let y = topo + (VERBO_H + vao_entre()) * i as f32;
            self.verbo(Rect::new(self.x, y, self.w, VERBO_H), v);
        }
    }

    fn verbo(&mut self, r: Rect, v: &Verbo<'_>) {
        self.hit_index.register(v.id, r);
        let hover = self.store.hover_live(v.id) > 0.0;
        let fundo = if hover {
            ColorToken::CardFieldHover
        } else {
            ColorToken::KanbanCardRaised
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        let tinta = self.ink.color(ColorToken::Text1);
        let vao = Spacing::Xs.px();
        if censo::ligado() {
            let pede = self.verbo_pede(v);
            censo::texto(v.nome, pede, r.w);
        }
        let tw = self.text_system.prefix_width(v.nome, m::VALUE_PX);
        let mut x = r.x + (r.w - (VERBO_GLIFO + vao + tw)) * 0.5;
        let y = r.y + (r.h - VERBO_GLIFO) * 0.5;
        paint_icon(
            self.scene,
            v.seta,
            Rect::new(x, y, VERBO_GLIFO, VERBO_GLIFO),
            tinta,
            StrokeToken::Default.px(),
        );
        x += VERBO_GLIFO + vao;
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
    }
}
