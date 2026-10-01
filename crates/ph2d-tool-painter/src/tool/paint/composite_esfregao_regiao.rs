//! ⭐⭐ **O que o ESFREGÃO lê, e portanto o que muda quando a tinta de baixo muda** (report do dono,
//! 2026-09-30, com foto: *«cria rectângulos de cor»*). Cortado do [`super::composite_acumulado`] pelo
//! tecto de LOC, por responsabilidade: lá a composição usa a resposta, aqui mora a pergunta.

use super::Region;
use super::composite::{CompositeOp, N_CAMADAS};
use crate::tool::PainterTool;

impl PainterTool {
    /// ⭐ **Quem, na área que o esfregão já tocou, LÊ de dentro da `caixa`** — a caixa dos píxeis `p`
    /// cuja origem `p − disp(p)` cai nela (crescida de UM texel: a amostra é bilinear). São eles, e só
    /// eles, cuja cor muda quando a tinta de baixo muda na caixa.
    ///
    /// ⚠️ **Não é a área tocada inteira**, e isso está medido: reescrevê-la toda também curava os
    /// rectângulos e custava `1,54×` na pilha do dono e `10,5 → 16,0 ms` por quadro num rabisco, que
    /// é o limite de um quadro. A maior parte da área tocada lê de sítios que este lote não mudou.
    /// O que custa isto é UMA comparação por texel da área tocada.
    ///
    /// Duas escalas, porque há dois leitores: a cor lê em `p − disp`, o corpo em `p − Plow·disp`.
    /// `None` quando o esfregão não lê uma base que muda (ou não há sessão).
    ///
    /// ⛔ **Com um BORRÃO por baixo do esfregão devolve a área tocada INTEIRA**: aí a cerca aplica o
    /// borrão sobre o `alvo` e a orla dele lê, fora da região, a imagem FINAL (que já tem o esfregão
    /// por cima) — uma base contaminada que só a área inteira afasta de onde o esfregão lê. Medido:
    /// com o conjunto preciso, Smear por cima de Blur fica na dívida antiga (`26`, a §33.12 do
    /// diário dizia `25`–`28`); com a área inteira vai a `0`. É o arranjo que o roteiro da cena já
    /// avisa ser o caminho lento, e só ele paga.
    pub(super) fn quem_le_da_caixa(&self, caixa: Region) -> Option<Region> {
        let t = self.paint.warp.touched_all?;
        if !self.o_esfregao_le_uma_base_que_muda() {
            return None;
        }
        if self.ha_borrao_por_baixo_do_esfregao() {
            return Some(t);
        }
        let (w, h) = self.source_size;
        let disp = &self.paint.warp.disp;
        if disp.len() != (w as usize) * (h as usize) {
            return None;
        }
        // A margem de UM texel é da amostra BILINEAR. ⚠️ Ela é defensiva e não observável hoje (a
        // mutação que a encolhe sobrevive): a caixa do lote já é um superconjunto da pegada dos dabs,
        // e na borda dela a tinta não muda.
        let (x0, y0) = (caixa.x as f32 - 1.0, caixa.y as f32 - 1.0);
        let (x1, y1) = ((caixa.x + caixa.w) as f32, (caixa.y + caixa.h) as f32);
        let dentro = |x: f32, y: f32| x >= x0 && x <= x1 && y >= y0 && y <= y1;
        let ds = self.paint.warp.relief_disp_scale;
        let mut acc: Option<(u32, u32, u32, u32)> = None;
        for py in t.y..t.y + t.h {
            let linha = py as usize * w as usize;
            for px in t.x..t.x + t.w {
                let d = disp[linha + px as usize];
                if d == [0.0, 0.0] {
                    continue; // lê de si mesmo: se estiver na caixa, já lá está
                }
                let (fx, fy) = (px as f32, py as f32);
                if dentro(fx - d[0], fy - d[1]) || dentro(fx - ds * d[0], fy - ds * d[1]) {
                    acc = Some(acc.map_or((px, py, px, py), |(a, b, c, e)| {
                        (a.min(px), b.min(py), c.max(px), e.max(py))
                    }));
                }
            }
        }
        acc.map(|(a, b, c, e)| Region {
            x: a,
            y: b,
            w: c - a + 1,
            h: e - b + 1,
        })
    }

    /// Há um borrão vivo por BAIXO de um esfregão vivo?
    pub(super) fn ha_borrao_por_baixo_do_esfregao(&self) -> bool {
        (0..N_CAMADAS).any(|e| {
            self.camada_viva(e)
                && self.paint.composite[e].op == CompositeOp::Smear
                && (e + 1..N_CAMADAS)
                    .any(|b| self.camada_viva(b) && self.paint.composite[b].op == CompositeOp::Blur)
        })
    }

    /// **O esfregão vivo lê uma base que MUDA durante o traço?** — há uma camada viva por BAIXO dele
    /// que escreve tinta (Brush, Erase, Blur). Com ele no fundo a base é o `pre`, que não muda.
    pub(super) fn o_esfregao_le_uma_base_que_muda(&self) -> bool {
        (0..N_CAMADAS).any(|e| {
            self.camada_viva(e)
                && self.paint.composite[e].op == CompositeOp::Smear
                && (e + 1..N_CAMADAS).any(|b| {
                    self.camada_viva(b) && self.paint.composite[b].op != CompositeOp::Smear
                })
        })
    }
}
