//! **O `Style: Solid` na aguada** (doc 46 §2-7) — a região que o gesto cerca entra nos acumuladores
//! da sessão molhada como MAIS UM CARIMBO (cobertura por `max`, cor por `over`), e a borda escura, a
//! granulação e o papel agem nela no composite do quadro como em qualquer traço.
//!
//! ⚠️ **Sem a corda.** No digital a corda que fecha o laço é carimbada com o pincel porque a aresta
//! do rasterizador é um corte duro (`solid_deposit::closing_chord_dabs`); na aguada a fronteira da
//! cobertura JÁ ganha a orla da aquarela no composite, e uma corda de carimbos seria um segundo
//! traço por cima dela.
//!
//! ⚠️ **No gesto à mão livre a mancha é PROVISÓRIA até o pen-up** — o polígono muda a cada evento
//! (a corda implícita anda com a mão), e os acumuladores da aguada não se descascam: sem o recorte
//! de cada plano reposto no evento seguinte, a união guardaria a mancha de TODOS os quadros (o defeito
//! que a corda do digital tinha nos acumuladores do traço, BUGS #34). O descasque mora na porta do
//! rascunho ([`PainterTool::peel_drag_preview`]) e o commit larga o registo
//! ([`PainterTool::commit_drag_preview`]). Nos editores de forma a aguada é reconstruída inteira a
//! cada quadro (`stamp_drag_preview_watercolor`), e a mancha entra sem registo.

use super::Region;
use super::watercolor_accum::{WASH_DEPOSIT_PEAK, splat_keep};
use super::watercolor_fios::Cobertura;
use crate::tool::PainterTool;
use ph2d_painter_brush::solid;

/// Os planos da sessão molhada que a mancha escreve, na ordem de [`PainterTool::plano_da_aguada`].
const PLANOS: usize = 6;

/// O que a mancha provisória do gesto à mão livre tapou: o recorte de cada plano da sessão molhada
/// sob o retângulo dela (`None` = o plano não existia, a mancha não o escreveu).
pub(crate) struct ManchaNaAguada {
    rect: Region,
    planos: [Option<(usize, Vec<u8>)>; PLANOS],
}

/// Bytes por texel de cada plano: a cor é RGBA, o resto é um byte.
const fn bpp(k: usize) -> usize {
    if k == 1 { 4 } else { 1 }
}

impl PainterTool {
    /// O plano `k` da sessão molhada: cobertura · cor · densidade da ponta · nível da reserva · a
    /// proximidade dele · o dono do estilo.
    fn plano_da_aguada(&mut self, k: usize) -> &mut Vec<u8> {
        match k {
            0 => &mut self.paint.stroke_coverage,
            1 => &mut self.paint.stroke_color,
            2 => &mut self.paint.stroke_density,
            3 => &mut self.paint.stroke_deplete,
            4 => &mut self.paint.stroke_deplete_prox,
            _ => &mut self.paint.wet_styles.owner,
        }
    }

    /// **Desfaz a mancha provisória** do evento anterior: repõe o recorte de cada plano e marca a
    /// janela para o composite do quadro (onde a mancha encolheu, a aguada tem de voltar a ser a de
    /// antes). Sem registo é um no-op.
    pub(super) fn peel_mancha_na_aguada(&mut self) {
        let Some(m) = self.mancha_na_aguada.take() else {
            return;
        };
        let largura = self.source_size.0 as usize;
        let r = m.rect;
        for (k, salvo) in m.planos.into_iter().enumerate() {
            let Some((len, bytes)) = salvo else {
                continue;
            };
            let plano = self.plano_da_aguada(k);
            if plano.len() != len {
                continue; // o plano nasceu de novo depois (um carimbo o re-armou): nada a repor
            }
            let linha = r.w as usize * bpp(k);
            for (j, y) in (r.y as usize..(r.y + r.h) as usize).enumerate() {
                let i = (y * largura + r.x as usize) * bpp(k);
                plano[i..i + linha].copy_from_slice(&bytes[j * linha..(j + 1) * linha]);
            }
        }
        self.paint.wet_frame_dirty = Some(
            self.paint
                .wet_frame_dirty
                .map_or(r, |d| super::union_region(d, r)),
        );
    }

    /// **Deposita a região cercada na aguada viva.** `provisoria` = o gesto à mão livre, cuja mancha o
    /// evento seguinte desfaz ([`Self::peel_mancha_na_aguada`]); `false` = um editor de forma, que
    /// reconstrói a aguada inteira a cada quadro.
    ///
    /// ⚠️ O corpo por texel é a porta que o fio partilha ([`super::watercolor_fios::PlanosDaAguada`]),
    /// e a `Strength` não entra — a mesma lei do carimbo da aguada (`WASH_DEPOSIT_PEAK`, *«a Strength
    /// não alcança a lavagem»*).
    pub(super) fn mancha_na_aguada(&mut self, loops: &[Vec<[f32; 2]>], provisoria: bool) {
        let Some(rect) = self.solid_fill_rect(loops) else {
            return;
        };
        let (fw, fh) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if rect.w == 0
            || rect.h == 0
            || self.paint.stroke_coverage.len() != fw * fh
            || self.paint.stroke_color.len() != fw * fh * 4
        {
            return; // a aguada ainda não nasceu (nenhum carimbo): não há sessão onde a pôr
        }
        if provisoria {
            let mut planos: [Option<(usize, Vec<u8>)>; PLANOS] = Default::default();
            for (k, salvo) in planos.iter_mut().enumerate() {
                let plano = self.plano_da_aguada(k);
                if plano.is_empty() {
                    continue;
                }
                let linha = rect.w as usize * bpp(k);
                let mut bytes = Vec::with_capacity(linha * rect.h as usize);
                for y in rect.y as usize..(rect.y + rect.h) as usize {
                    let i = (y * fw + rect.x as usize) * bpp(k);
                    bytes.extend_from_slice(&plano[i..i + linha]);
                }
                *salvo = Some((plano.len(), bytes));
            }
            self.mancha_na_aguada = Some(ManchaNaAguada { rect, planos });
        }
        for dirty in [
            &mut self.paint.wet_frame_dirty,
            &mut self.paint.wet_cum_dirty,
            &mut self.paint.wet_stroke_dirty,
        ] {
            *dirty = Some(dirty.map_or(rect, |d| super::union_region(d, rect)));
        }
        #[allow(clippy::cast_precision_loss)]
        let origin = [rect.x as f32, rect.y as f32];
        let regiao = solid::fill_coverage(loops, rect.w as usize, rect.h as usize, origin);
        let (sel, prot, alock) = self.wet_splat_gates();
        let gated = sel.is_some() || prot.is_some() || alock.is_some();
        let mut planos = self.planos_da_aguada();
        for row in 0..rect.h as usize {
            let y = rect.y as usize + row;
            for cx in 0..rect.w as usize {
                let x = rect.x as usize + cx;
                let dentro = f32::from(regiao[row * rect.w as usize + cx]) / 255.0;
                if dentro <= 0.0 || x >= fw || y >= fh {
                    continue;
                }
                let idx = y * fw + x;
                let keep = if gated {
                    splat_keep(
                        sel.as_deref().map(Vec::as_slice),
                        prot.as_deref().map(Vec::as_slice),
                        alock.as_deref().map(Vec::as_slice),
                        idx,
                    )
                } else {
                    1.0
                };
                planos.deposita(idx, WASH_DEPOSIT_PEAK * keep * dentro, Cobertura::Max);
            }
        }
    }
}
