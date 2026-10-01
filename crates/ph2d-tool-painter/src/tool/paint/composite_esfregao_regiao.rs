//! ⭐⭐ **O que o ESFREGÃO lê, e portanto o que muda quando a tinta de baixo muda** (report do dono,
//! 2026-09-30, com foto: *«cria rectângulos de cor»*). Cortado do [`super::composite_acumulado`] pelo
//! tecto de LOC, por responsabilidade: lá a composição usa a resposta, aqui mora a pergunta — e,
//! desde 2026-10-01 (outro corte pelo tecto), também o braço do esfregão na composição.

use super::Region;
use super::composite::{CompositeOp, N_CAMADAS};
#[cfg(test)]
use super::composite_acumulado::fases;
use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;

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
        let ds = self.paint.warp.relief_disp_scale;
        // ⭐ **As linhas da área tocada em paralelo** (2026-10-01, a pilha a Size `0.5`): a pergunta é
        // uma caixa envolvente — mínimos e máximos, associativos e comutativos —, logo repartir as
        // linhas muda QUEM as lê e nunca a resposta. Ela corria num núcleo sobre a área tocada
        // INTEIRA, a cada quadro, e era `~1` ms do quadro do rabisco do dono (ADR-0172, emenda (c)).
        // A lei de UMA linha é [`le_da_caixa_na_linha`], com o laço em série como oráculo do gate.
        use rayon::prelude::*;
        let acc = (t.y..t.y + t.h)
            .into_par_iter()
            .filter_map(|py| {
                le_da_caixa_na_linha(disp, w, py, t.x, t.x + t.w, ds, [x0, y0, x1, y1])
                    .map(|(a, c)| (a, py, c, py))
            })
            .reduce_with(|(a, b, c, e), (a2, b2, c2, e2)| {
                (a.min(a2), b.min(b2), c.max(c2), e.max(e2))
            });
        acc.map(|(a, b, c, e)| Region {
            x: a,
            y: b,
            w: c - a + 1,
            h: e - b + 1,
        })
    }

    /// O laço em SÉRIE de [`Self::quem_le_da_caixa`], linha a linha — o oráculo do gate do paralelo.
    #[cfg(test)]
    pub(super) fn quem_le_da_caixa_em_serie(&self, caixa: Region) -> Option<Region> {
        let t = self.paint.warp.touched_all?;
        let (w, _) = self.source_size;
        let disp = &self.paint.warp.disp;
        let (x0, y0) = (caixa.x as f32 - 1.0, caixa.y as f32 - 1.0);
        let (x1, y1) = ((caixa.x + caixa.w) as f32, (caixa.y + caixa.h) as f32);
        let ds = self.paint.warp.relief_disp_scale;
        let mut acc: Option<(u32, u32, u32, u32)> = None;
        for py in t.y..t.y + t.h {
            if let Some((a, c)) =
                le_da_caixa_na_linha(disp, w, py, t.x, t.x + t.w, ds, [x0, y0, x1, y1])
            {
                acc = Some(acc.map_or((a, py, c, py), |(a0, b0, c0, e0)| {
                    (a0.min(a), b0.min(py), c0.max(c), e0.max(py))
                }));
            }
        }
        acc.map(|(a, b, c, e)| Region {
            x: a,
            y: b,
            w: c - a + 1,
            h: e - b + 1,
        })
    }

    /// **A caixa do lote mais quem, na área já esfregada, LÊ de dentro dela** — o que muda na tela
    /// quando a tinta de baixo muda ([`Self::quem_le_da_caixa`]).
    pub(super) fn o_que_mudou(&self, caixa_nova: Region) -> Region {
        #[cfg(test)]
        let t_quem = std::time::Instant::now();
        let mudou = match self.quem_le_da_caixa(caixa_nova) {
            Some(d) => super::union_region(caixa_nova, d),
            None => caixa_nova,
        };
        #[cfg(test)]
        fases::soma_sub(10, t_quem);
        mudou
    }

    /// **O ESFREGÃO** — inalterado: ele já era um campo por traço resolvido de uma vez, e a base
    /// dele é refrescada com o que as camadas de BAIXO acabaram de deixar na região.
    pub(super) fn compoe_esfregao(&mut self, pos: usize, r: Region, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        #[cfg(test)]
        let t_sub = std::time::Instant::now();
        self.refresca_a_base_do_smear(r);
        #[cfg(test)]
        fases::soma_sub(6, t_sub);
        #[cfg(test)]
        let t_sub = std::time::Instant::now();
        #[cfg(test)]
        let limite =
            (!super::composite_pilha::SMEAR_SEM_LIMITE.with(std::cell::Cell::get)).then_some(r);
        #[cfg(not(test))]
        let limite = Some(r);
        self.paint.limite_do_smear = limite;
        // ⭐ O fluxo aleatório do esfregão é o DELE (`rng_camada[pos]`), como o de toda camada que
        // acumula — antes ele herdava o da última camada acumulada, e a Shape/Grain aleatória dele
        // dependia de quem acumulara antes. É também o que deixa o campo correr ao lado do acúmulo
        // ([`super::composite_por_quadro`]) com o mesmo fluxo.
        let saved_rng = self.paint.tex_rng;
        self.paint.tex_rng = self.paint.rng_camada[pos];
        self.aplica_camada(pos, dabs);
        self.paint.rng_camada[pos] = self.paint.tex_rng;
        self.paint.tex_rng = saved_rng;
        #[cfg(test)]
        fases::soma_sub(7, t_sub);
        self.paint.limite_do_smear = None;
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

/// **UMA linha da pergunta de [`PainterTool::quem_le_da_caixa`]:** o primeiro e o último `px` em
/// `[xa, xb)` cuja origem (`p − disp` para a cor, `p − ds·disp` para o corpo) cai na caixa
/// `[x0, x1] × [y0, y1]`. Um texel com `disp` nulo lê de si mesmo e não conta.
fn le_da_caixa_na_linha(
    disp: &[[f32; 2]],
    w: u32,
    py: u32,
    xa: u32,
    xb: u32,
    ds: f32,
    [x0, y0, x1, y1]: [f32; 4],
) -> Option<(u32, u32)> {
    let dentro = |x: f32, y: f32| x >= x0 && x <= x1 && y >= y0 && y <= y1;
    let linha = py as usize * w as usize;
    let mut acc: Option<(u32, u32)> = None;
    for px in xa..xb {
        let d = disp[linha + px as usize];
        if d == [0.0, 0.0] {
            continue; // lê de si mesmo: se estiver na caixa, já lá está
        }
        #[allow(clippy::cast_precision_loss)]
        let (fx, fy) = (px as f32, py as f32);
        if dentro(fx - d[0], fy - d[1]) || dentro(fx - ds * d[0], fy - ds * d[1]) {
            acc = Some(acc.map_or((px, px), |(a, c)| (a.min(px), c.max(px))));
        }
    }
    acc
}
