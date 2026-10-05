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
//! (a corda implícita anda com a mão), e os acumuladores da aguada não se descascam sozinhos: sem
//! repor o que a mancha tapou, a união guardaria a mancha de TODOS os quadros (o defeito que a corda
//! do digital tinha nos acumuladores do traço, BUGS #34). O commit larga o registo
//! ([`PainterTool::commit_drag_preview`]). Nos editores de forma a aguada é reconstruída inteira a
//! cada quadro (`stamp_drag_preview_watercolor`), e a mancha entra sem registo.
//!
//! ⭐ **E a mancha refaz-se só onde MUDOU** (report do dono 2026-10-05: *«ficou lento numa mancha de
//! 1000px»*). Medido num laço de 1000 px em `2048²`: descascar e repor a caixa INTEIRA a cada evento
//! custava `~15–20 ms` por quadro (o composite da caixa `9,3` · o depósito `2,8`) contra `~1` sem
//! Solid. Entre dois eventos a mancha muda só no triângulo da corda e onde a tinta do evento caiu,
//! então o registo guarda o PAPEL SEM A MANCHA ([`ManchaNaAguada`]) ao longo do gesto:
//! 1. antes de um lote de dabs (e dos fios), a mancha sai SÓ sob a janela de escrita dele
//!    ([`PainterTool::descasca_a_mancha_sob`] — a `dab_batch_region`, a mesma em que o composite
//!    confia), e a janela fica marcada;
//! 2. no fecho do evento o papel sem mancha é refrescado nas janelas marcadas, e só os texels cuja
//!    cobertura mudou são repostos e re-depositados ([`PainterTool::atualiza_a_mancha`]); o quadro
//!    recompõe só a caixa deles.
//!
//! A rota de antes (descascar e repor a caixa inteira) fica em `WashCadence::mancha_inteira` — o
//! oráculo ao byte (`a_mancha_incremental_e_a_inteira_ao_byte`) e a metade A de uma medição A×B.

use super::Region;
use super::watercolor_accum::{WASH_DEPOSIT_PEAK, splat_keep};
use super::watercolor_fios::Cobertura;
use crate::tool::PainterTool;
use ph2d_painter_brush::solid;

/// Os planos da sessão molhada que a mancha escreve, na ordem de [`PainterTool::plano_da_aguada`].
const PLANOS: usize = 6;

/// O registo da mancha provisória do gesto à mão livre, sob a caixa `rect` (que cresce com a mancha
/// e não encolhe durante o gesto).
pub(crate) struct ManchaNaAguada {
    rect: Region,
    /// O PAPEL SEM A MANCHA, plano a plano (`None` = o plano não existia quando o registo nasceu, e
    /// a mancha não o escreve). Fora dos texels com cobertura ele É o plano vivo.
    planos: [Option<(usize, Vec<u8>)>; PLANOS],
    /// A cobertura que a mancha tem nos planos agora (`0` = fora ou descascada).
    cob: Vec<u8>,
    /// As janelas descascadas desde o último fecho de evento — onde a tinta do evento caiu sobre o
    /// papel sem mancha, que tem de ser refrescado dali.
    ///
    /// ⚠️ **Uma LISTA, nunca a caixa que as envolve:** entre a janela dos dabs e a dos fios do mesmo
    /// evento há texels onde a mancha NÃO saiu, e refrescar a caixa copiava a mancha para o papel
    /// sem ela (medido: cobertura `5` onde a rota inteira dá `0`, o gate ao byte apanhou-o).
    sujo: Vec<Region>,
}

/// Bytes por texel de cada plano: a cor é RGBA, o resto é um byte.
const fn bpp(k: usize) -> usize {
    if k == 1 { 4 } else { 1 }
}

/// O índice de `(x, y)` (em pixels de canvas) dentro da caixa `r`.
fn em(r: Region, x: usize, y: usize) -> usize {
    (y - r.y as usize) * r.w as usize + (x - r.x as usize)
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

    /// Copia o texel `(x, y)` entre o plano vivo e o papel sem mancha do registo `m`: `repoe` =
    /// do registo para o plano; senão do plano para o registo.
    fn troca_o_texel(&mut self, m: &mut ManchaNaAguada, x: usize, y: usize, repoe: bool) {
        let largura = self.source_size.0 as usize;
        let j = em(m.rect, x, y);
        for k in 0..PLANOS {
            let Some((len, bytes)) = m.planos[k].as_mut() else {
                continue;
            };
            let plano = self.plano_da_aguada(k);
            if plano.len() != *len {
                continue; // o plano nasceu de novo depois (um carimbo o re-armou): nada a trocar
            }
            let b = bpp(k);
            let (i, jb) = ((y * largura + x) * b, j * b);
            if repoe {
                plano[i..i + b].copy_from_slice(&bytes[jb..jb + b]);
            } else {
                bytes[jb..jb + b].copy_from_slice(&plano[i..i + b]);
            }
        }
    }

    /// **Desfaz a mancha provisória INTEIRA** e larga o registo: repõe o papel sem mancha onde ela
    /// estava e marca a caixa para o composite. A porta do Esc, dos rascunhos e da rota inteira.
    pub(super) fn peel_mancha_na_aguada(&mut self) {
        let Some(mut m) = self.mancha_na_aguada.take() else {
            return;
        };
        let r = m.rect;
        for y in r.y as usize..(r.y + r.h) as usize {
            for x in r.x as usize..(r.x + r.w) as usize {
                if m.cob[em(r, x, y)] > 0 {
                    self.troca_o_texel(&mut m, x, y, true);
                }
            }
        }
        self.marca_o_quadro(r);
    }

    /// **Tira a mancha só sob a janela `r`**, antes de a tinta do evento cair lá (um lote de dabs, os
    /// fios): ela cai sobre o papel sem mancha, como na rota inteira, e a janela fica marcada para o
    /// fecho do evento refrescar o registo dali. Sem registo é um no-op.
    ///
    /// ⚠️ Na rota inteira (`wash.mancha_inteira`) a mancha sai TODA, aqui também: os fios caem no
    /// `park_stroke`, e num tique que costura sem carimbar (sem o descasque do lote) eles caíam por
    /// cima da mancha e o descasque seguinte, com o recorte de antes deles, APAGAVA-OS.
    pub(super) fn descasca_a_mancha_sob(&mut self, r: Region) {
        if self.wash.mancha_inteira {
            if self.mancha_na_aguada.is_some() {
                self.peel_mancha_na_aguada();
                self.paint.solid_fill_owed = true;
            }
            return;
        }
        let Some(mut m) = self.mancha_na_aguada.take() else {
            return;
        };
        if let Some(i) = super::region::intersect_region(m.rect, r) {
            let mut tirou = None;
            for y in i.y as usize..(i.y + i.h) as usize {
                for x in i.x as usize..(i.x + i.w) as usize {
                    let j = em(m.rect, x, y);
                    if m.cob[j] > 0 {
                        self.troca_o_texel(&mut m, x, y, true);
                        m.cob[j] = 0;
                        tirou = Some(());
                    }
                }
            }
            m.sujo.push(i);
            if tirou.is_some() {
                self.marca_o_quadro(i);
            }
            // A mancha tem de voltar NESTE evento (os fios podem cair num evento sem dabs).
            self.paint.solid_fill_owed = true;
        }
        self.mancha_na_aguada = Some(m);
    }

    fn marca_o_quadro(&mut self, r: Region) {
        self.paint.wet_frame_dirty = Some(
            self.paint
                .wet_frame_dirty
                .map_or(r, |d| super::union_region(d, r)),
        );
    }

    /// Os acumuladores da aguada existem? (Nenhum carimbo ainda = não há sessão onde a pôr.)
    fn aguada_nasceu(&self) -> bool {
        let (fw, fh) = (self.source_size.0 as usize, self.source_size.1 as usize);
        self.paint.stroke_coverage.len() == fw * fh && self.paint.stroke_color.len() == fw * fh * 4
    }

    /// **A mancha provisória do gesto à mão livre, refeita só onde mudou** (ver o cabeçalho). O
    /// registo nasce na 1.ª chamada e cresce com a caixa; o quadro recompõe só os texels que
    /// mudaram de cobertura e as janelas onde a tinta do evento caiu.
    pub(super) fn atualiza_a_mancha(&mut self, loops: &[Vec<[f32; 2]>]) {
        if !self.aguada_nasceu() {
            return;
        }
        let nova = self.solid_fill_rect(loops).filter(|r| r.w > 0 && r.h > 0);
        let mut m = match self.mancha_na_aguada.take() {
            Some(m) => m,
            None => {
                let Some(r) = nova else {
                    return;
                };
                self.registo_novo(r)
            }
        };
        if let Some(r) = nova {
            let u = super::union_region(m.rect, r);
            if u != m.rect {
                m = self.cresce_o_registo(m, u);
            }
        }
        // 1. O papel sem mancha é refrescado onde a tinta do evento caiu (lá a mancha foi tirada).
        for s in std::mem::take(&mut m.sujo) {
            let Some(i) = super::region::intersect_region(m.rect, s) else {
                continue;
            };
            for y in i.y as usize..(i.y + i.h) as usize {
                for x in i.x as usize..(i.x + i.w) as usize {
                    self.troca_o_texel(&mut m, x, y, false);
                }
            }
        }
        // 2. A cobertura de AGORA, e só os texels que mudaram: repor o papel e depositar de novo.
        let regiao = nova.map(|r| {
            #[allow(clippy::cast_precision_loss)]
            let origin = [r.x as f32, r.y as f32];
            (
                r,
                solid::fill_coverage(loops, r.w as usize, r.h as usize, origin),
            )
        });
        let a_de = |x: usize, y: usize| -> u8 {
            regiao.as_ref().map_or(0, |(r, c)| {
                let dentro = x >= r.x as usize
                    && y >= r.y as usize
                    && x < (r.x + r.w) as usize
                    && y < (r.y + r.h) as usize;
                if dentro { c[em(*r, x, y)] } else { 0 }
            })
        };
        let rect = m.rect;
        let mut mudou: Option<(usize, usize, usize, usize)> = None;
        let mut depositar = Vec::new();
        for y in rect.y as usize..(rect.y + rect.h) as usize {
            for x in rect.x as usize..(rect.x + rect.w) as usize {
                let j = em(rect, x, y);
                let c = a_de(x, y);
                if c == m.cob[j] {
                    continue;
                }
                if m.cob[j] > 0 {
                    self.troca_o_texel(&mut m, x, y, true);
                }
                m.cob[j] = c;
                if c > 0 {
                    depositar.push((x, y, c));
                }
                mudou = Some(mudou.map_or((x, y, x, y), |(a, b, cx, cy)| {
                    (a.min(x), b.min(y), cx.max(x), cy.max(y))
                }));
            }
        }
        self.mancha_na_aguada = Some(m);
        self.deposita_a_mancha(&depositar);
        if let Some((x0, y0, x1, y1)) = mudou {
            #[allow(clippy::cast_possible_truncation)]
            self.marca_o_quadro(Region {
                x: x0 as u32,
                y: y0 as u32,
                w: (x1 - x0 + 1) as u32,
                h: (y1 - y0 + 1) as u32,
            });
        }
        if let Some(r) = nova {
            for dirty in [
                &mut self.paint.wet_cum_dirty,
                &mut self.paint.wet_stroke_dirty,
            ] {
                *dirty = Some(dirty.map_or(r, |d| super::union_region(d, r)));
            }
        }
    }

    /// Um registo vazio sob `r`: o papel sem mancha é o plano vivo.
    fn registo_novo(&mut self, r: Region) -> ManchaNaAguada {
        let fw = self.source_size.0 as usize;
        let mut planos: [Option<(usize, Vec<u8>)>; PLANOS] = Default::default();
        for (k, salvo) in planos.iter_mut().enumerate() {
            let plano = self.plano_da_aguada(k);
            if plano.is_empty() {
                continue;
            }
            let linha = r.w as usize * bpp(k);
            let mut bytes = Vec::with_capacity(linha * r.h as usize);
            for y in r.y as usize..(r.y + r.h) as usize {
                let i = (y * fw + r.x as usize) * bpp(k);
                bytes.extend_from_slice(&plano[i..i + linha]);
            }
            *salvo = Some((plano.len(), bytes));
        }
        ManchaNaAguada {
            rect: r,
            planos,
            cob: vec![0; r.w as usize * r.h as usize],
            sujo: Vec::new(),
        }
    }

    /// O registo `m` crescido para a caixa `u ⊇ m.rect`: o que já guardava fica, e o resto é o plano
    /// vivo (a mancha nunca esteve lá).
    fn cresce_o_registo(&mut self, m: ManchaNaAguada, u: Region) -> ManchaNaAguada {
        let mut novo = self.registo_novo(u);
        let r = m.rect;
        for (k, salvo) in novo.planos.iter_mut().enumerate() {
            let (Some((_, dst)), Some((_, src))) = (salvo.as_mut(), m.planos[k].as_ref()) else {
                continue;
            };
            let b = bpp(k);
            let linha = r.w as usize * b;
            for (row, y) in (r.y as usize..(r.y + r.h) as usize).enumerate() {
                let d = em(u, r.x as usize, y) * b;
                dst[d..d + linha].copy_from_slice(&src[row * linha..(row + 1) * linha]);
            }
        }
        for (row, y) in (r.y as usize..(r.y + r.h) as usize).enumerate() {
            let d = em(u, r.x as usize, y);
            novo.cob[d..d + r.w as usize]
                .copy_from_slice(&m.cob[row * r.w as usize..(row + 1) * r.w as usize]);
        }
        novo.sujo = m.sujo;
        novo
    }

    /// O depósito da mancha nos texels `(x, y, cobertura)`, pela porta que o fio partilha
    /// ([`super::watercolor_fios::PlanosDaAguada`]). A `Strength` não entra — a mesma lei do carimbo
    /// da aguada (`WASH_DEPOSIT_PEAK`, *«a Strength não alcança a lavagem»*).
    fn deposita_a_mancha(&mut self, texels: &[(usize, usize, u8)]) {
        if texels.is_empty() {
            return;
        }
        let fw = self.source_size.0 as usize;
        let (sel, prot, alock) = self.wet_splat_gates();
        let gated = sel.is_some() || prot.is_some() || alock.is_some();
        let mut planos = self.planos_da_aguada();
        for &(x, y, c) in texels {
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
            let dentro = f32::from(c) / 255.0;
            planos.deposita(idx, WASH_DEPOSIT_PEAK * keep * dentro, Cobertura::Max);
        }
    }

    /// **Deposita a região cercada na aguada viva, sem registo** — os editores de forma, que
    /// reconstroem a aguada inteira a cada quadro (`stamp_drag_preview_watercolor`).
    pub(super) fn mancha_na_aguada(&mut self, loops: &[Vec<[f32; 2]>]) {
        let Some(rect) = self.solid_fill_rect(loops) else {
            return;
        };
        let (fw, fh) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if rect.w == 0 || rect.h == 0 || !self.aguada_nasceu() {
            return;
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
        let mut texels = Vec::new();
        for row in 0..rect.h as usize {
            let y = rect.y as usize + row;
            for cx in 0..rect.w as usize {
                let x = rect.x as usize + cx;
                let c = regiao[row * rect.w as usize + cx];
                if c > 0 && x < fw && y < fh {
                    texels.push((x, y, c));
                }
            }
        }
        self.deposita_a_mancha(&texels);
    }
}
