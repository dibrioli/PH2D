//! ⭐⭐⭐ **A PILHA ACUMULA — ela deixou de REPLAYAR a história.**
//!
//! # O defeito que isto cura, medido
//!
//! A wave da ordem por traço (ver [`super::composite_pilha`]) implementou a lei
//! `tela = L₀(L₁(…L_N(pre)…))` **replayando** os lotes passados a cada evento de ponteiro. O
//! cabeçalho dela afirma que o número de lotes replayados *«não cresce com o traço»*. Medido em
//! 2026-09-21, isso é **verdade para um traço RECTO e falso para um RABISCO** — e um rabisco é o
//! que pintar é: um traço que volta à própria vizinhança faz toda caixa tocar toda caixa.
//!
//! | caminho | ms/evento a 60 passos | a 480 passos | lotes replayados por lote |
//! |---|---|---|---|
//! | recto | `0,76` | `0,90` | `11,9` → `17,6` |
//! | rabisco | `2,99` | **`12,11`** | `16,7` → **`79,4`** |
//!
//! O total é **quadrático** (`179 ms` → `5 814 ms`), e a fase dominante é o replay dos depósitos
//! (Brush `49 %` + Blur `45`–`70 %`; o save/restore custa `< 1 %`).
//!
//! # A lei não mudou — a IMPLEMENTAÇÃO é que passou a ser a lei
//!
//! *«Cada `L` aplicada sobre o TRAÇO INTEIRO»* quer dizer que cada camada é aplicada **UMA vez**,
//! com tudo o que ela acumulou. Replayar dab a dab era uma maneira de chegar lá, não a lei.
//!
//! ⇒ cada camada tem um **PLANO** onde ela acumula ao longo do traço, e a composição corre uma vez
//! por evento sobre a região nova:
//!
//! ```text
//!     plano_k ← plano_k ⊕ (os dabs NOVOS da camada k)        O(dabs novos)
//!     tela|R  ← pre|R ⊕ plano_N ⊕ … ⊕ plano_0                 O(R × N)
//! ```
//!
//! **O custo por evento deixa de depender da história.** Medido pela ablação do replay antes de a
//! cura existir: `8,7×` (recto) a **`49,0×`** (rabisco longo), e o custo volta a crescer em LINHA
//! RECTA (`112 → 218 ms` ao dobrar o traço, contra `2 736 → 10 666`).
//!
//! # ⭐ O acumulador é o MESMO depósito, por troca de plano
//!
//! Nada aqui rasteriza um dab de novo: o plano de cada camada é posto no lugar do canvas pela porta
//! [`super::plane_fork::swap_canvas_plane`] e o depósito de sempre corre sobre ele. É o truque que
//! o ESCUDO da borracha de escopo `Traco` já usava — *uma lei, uma porta*.
//!
//! # Exacto em quatro das cinco operações, e a quinta é DECLARADA
//!
//! * **Brush** — acumular com `Mix` e compor uma vez é **idêntico** a depositar os dabs em ordem:
//!   o `over` é associativo, e a cor de cada dab viaja no plano (logo o *Randomize Color* fica
//!   vivo). ⚠️ A camada é composta com o blend do pincel **uma vez**; com `Mix` (o de fábrica) isso
//!   é exacto, e com os outros modos é a lei de CAMADA de um editor — que é o que o dono descreveu
//!   (*«o brush de cima sempre deve ser desenhado por cima do de baixo»*).
//! * **Erase** — a cobertura acumulada `c` é MEDIDA no escudo opaco, e as duas leis (`Tudo`:
//!   `α ← α(1−c)` · `Traco`: `lerp(tela, pre, c)`) são as mesmas de antes, com o `c` do traço
//!   inteiro em vez do lote. **Exacto** pela mesma álgebra.
//! * **Smear** — já era um campo por traço resolvido de uma vez. **Não muda.**
//! * **Blur** — ⛔ **DIVERGÊNCIA DECLARADA:** `N` passagens sequenciais compõem-se num borrão MAIOR
//!   (`σ√N`), e uma passagem com o peso acumulado mistura o mesmo borrão mais fundo. A lei escrita
//!   diz *uma* aplicação; o `N` era o artefacto. ⚠️ E ela é **autorizada pelo dono**, que pediu
//!   precisamente um Blur mais barato só no composite (2026-09-20: *«veja se abaixando a qualidade
//!   do blur não fica bem mais leve; mas só no Blur do composite»*).
//!
//! ⚠️ **A história (`LoteDaPilha`) MORREU com o replay** — e com ela a memória que crescia com o
//! traço.

use super::Region;
use super::composite::{CompositeOp, EscopoDaBorracha, N_CAMADAS};
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushBlend, Dab};
use std::sync::Arc;

/// O que uma camada acumula ao longo do traço.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) enum Acumulo {
    /// A TINTA, em RGBA recta — o plano nasce transparente e o depósito compõe nele com `Mix`.
    Tinta,
    /// A COBERTURA, medida no escudo opaco — o plano nasce a `255` e o depósito come-lhe o alfa.
    Cobertura,
    /// Nada: a operação tem estado próprio por traço.
    Nenhum,
}

impl CompositeOp {
    /// O que esta operação acumula. ⭐ É a porta ÚNICA da pergunta, lida pela acumulação **e** pela
    /// composição — duas respostas divergiriam no dia da quinta operação.
    pub(super) fn acumula(self) -> Acumulo {
        match self {
            Self::Brush => Acumulo::Tinta,
            Self::Erase | Self::Blur => Acumulo::Cobertura,
            Self::Smear => Acumulo::Nenhum,
        }
    }
}

impl PainterTool {
    /// **O corpo da lei: acumular o lote novo, depois compor a região UMA vez.**
    pub(super) fn acumula_e_compoe(&mut self, camadas: [Vec<Dab>; N_CAMADAS]) {
        let (w, h) = self.source_size;
        let len = (w as usize) * (h as usize) * 4;
        if len == 0 || self.canvas_rgba.len() != len {
            return;
        }
        // 1. A pilha abre no primeiro lote: `pre` é a tela ANTES de qualquer camada deste traço.
        if self.paint.pilha.pre.len() != len {
            self.paint.pilha.pre = (*self.canvas_rgba).clone();
            self.paint.pilha.lotes.clear();
            for p in &mut self.paint.pilha.planos {
                *p = Arc::new(Vec::new());
            }
        }
        let Some(caixa_nova) = super::composite_pilha::caixa_das_camadas(&camadas, w, h) else {
            return;
        };
        // A bissecção do §3.1 da auditoria, agora sobre a rota de acumulação: com a região a ser o
        // canvas inteiro nada é recortado, e a diferença contra a rota normal É o que o limite
        // regional custa à imagem.
        #[cfg(test)]
        let caixa_nova = if super::composite_pilha::RECOMPOSICAO_GLOBAL.with(std::cell::Cell::get) {
            Region { x: 0, y: 0, w, h }
        } else {
            caixa_nova
        };
        // 2. Cada camada acumula os dabs NOVOS dela no plano dela. `O(dabs novos)`.
        for pos in 0..N_CAMADAS {
            if self.paint.composite[pos].strength <= 0.0 || camadas[pos].is_empty() {
                continue;
            }
            self.acumula_camada(pos, &camadas[pos]);
        }
        // 3. A região da composição. ⚠️ O apron do Blur é o que impede a convolução de ler, na orla,
        //    bytes que a composição ainda não escreveu — e é por isso que só o miolo sobrevive.
        let pad = self.pad_do_borrao();
        let alvo = super::region::grow_region(caixa_nova, pad, w, h).unwrap_or(caixa_nova);
        // 4. Guardar a orla, compor, e devolver tudo o que não é a caixa nova.
        let guardado = self.save_region(&alvo);
        self.compoe_a_pilha(alvo, &camadas);
        let composto = self.save_region(&caixa_nova);
        self.escreve_regiao(alvo, &guardado);
        self.escreve_regiao(caixa_nova, &composto);
        self.declare_wrote(Some(alvo));
        self.mark_dirty(alvo);
    }

    /// Depositar os dabs novos de uma camada **no plano dela**, pela porta de troca de plano.
    fn acumula_camada(&mut self, pos: usize, dabs: &[Dab]) {
        let layer = self.paint.composite[pos];
        let modo = layer.op.acumula();
        if matches!(modo, Acumulo::Nenhum) {
            return;
        }
        let len = self.canvas_rgba.len();
        if self.paint.pilha.planos[pos].len() != len {
            // ⚠️ O estado inicial é a lei do acumulador: a tinta compõe-se sobre o TRANSPARENTE, a
            // cobertura MEDE-SE num plano opaco a que o depósito come o alfa.
            let semente = match modo {
                Acumulo::Tinta => vec![0u8; len],
                _ => vec![255u8; len],
            };
            self.paint.pilha.planos[pos] = Arc::new(semente);
        }
        let saved_strength = self.paint.brush.strength;
        let saved_blend = self.paint.brush.blend;
        let saved_hardness = self.paint.brush.hardness;
        let saved_draw = self.paint.brush.impasto_draw_to;
        self.paint.brush.strength = layer.strength;
        self.paint.brush.hardness = layer.hardness.unwrap_or(saved_hardness);
        // ⚠️ A TINTA acumula-se com `Mix` e **não** com o blend do pincel: o blend é da CAMADA e
        // corre uma vez, na composição. Acumular com ele aplicá-lo-ia uma vez por dab.
        self.paint.brush.blend = match modo {
            Acumulo::Tinta => BrushBlend::Mix,
            _ => BrushBlend::EraseAlpha,
        };
        // ⛔ O relevo não entra num plano de medição — ele é da TELA.
        self.paint.brush.impasto_draw_to = ph2d_painter_brush::DrawTo::Color;
        // O fluxo de RNG desta camada é dela: um fluxo partilhado daria realizações diferentes
        // conforme a ordem em que as camadas correm.
        self.paint.tex_rng = self.paint.rng_camada[pos];
        std::mem::swap(
            &mut self.paint.stroke_mask,
            &mut self.paint.composite_mask[pos],
        );
        self.paint.acumulando_no_plano = true;
        let mut plano = std::mem::take(&mut self.paint.pilha.planos[pos]);
        super::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plano,
            &self.undo.write_state,
        );
        self.aplica_deposito(dabs);
        super::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plano,
            &self.undo.write_state,
        );
        self.paint.pilha.planos[pos] = plano;
        self.paint.acumulando_no_plano = false;
        std::mem::swap(
            &mut self.paint.stroke_mask,
            &mut self.paint.composite_mask[pos],
        );
        self.paint.rng_camada[pos] = self.paint.tex_rng;
        self.paint.brush.impasto_draw_to = saved_draw;
        self.paint.brush.blend = saved_blend;
        self.paint.brush.strength = saved_strength;
        self.paint.brush.hardness = saved_hardness;
    }

    /// **A composição: `pre` e depois cada camada UMA vez, de baixo para cima.**
    fn compoe_a_pilha(&mut self, r: Region, camadas: &[Vec<Dab>; N_CAMADAS]) {
        self.escreve_do_pre(r);
        for pos in (0..N_CAMADAS).rev() {
            let layer = self.paint.composite[pos];
            if layer.strength <= 0.0 {
                continue;
            }
            match layer.op {
                CompositeOp::Brush => self.compoe_tinta(pos, r),
                CompositeOp::Erase => self.compoe_borracha(pos, r, layer.erase_scope),
                CompositeOp::Blur => self.compoe_borrao(pos, r),
                CompositeOp::Smear => self.compoe_esfregao(pos, r, &camadas[pos]),
            }
        }
    }
}

/// `0..255` → `0..1`.
#[inline]
fn dec(b: u8) -> f32 {
    f32::from(b) / 255.0
}
/// `0..1` → `0..255`, com o mesmo arredondamento do resto da casa.
#[inline]
fn enc(v: f32) -> u8 {
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

impl PainterTool {
    /// **A TINTA de uma camada Brush, composta UMA vez** com o blend do pincel.
    ///
    /// ⭐ Com `Mix` isto é **idêntico** a ter depositado os dabs em ordem sobre esta base: o
    /// `over` é associativo, e o plano guarda exactamente `over(dₙ, … over(d₁, transparente))`.
    fn compoe_tinta(&mut self, pos: usize, r: Region) {
        let plano = std::mem::take(&mut self.paint.pilha.planos[pos]);
        if plano.len() == self.canvas_rgba.len() {
            let blend = self.paint.brush.blend;
            let alpha_locked = self.o_alfa_esta_trancado();
            let stride = self.source_size.0 as usize * 4;
            let buf = super::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                self.source_size.0,
                Some(r),
            );
            for row in 0..r.h as usize {
                let base = (r.y as usize + row) * stride + r.x as usize * 4;
                for col in 0..r.w as usize {
                    let i = base + col * 4;
                    let a = dec(plano[i + 3]);
                    if a <= 0.0 {
                        continue;
                    }
                    let dst = [
                        dec(buf[i]),
                        dec(buf[i + 1]),
                        dec(buf[i + 2]),
                        dec(buf[i + 3]),
                    ];
                    let mut out = ph2d_painter_brush::blend_over(
                        blend,
                        dst,
                        [dec(plano[i]), dec(plano[i + 1]), dec(plano[i + 2])],
                        a,
                    );
                    if alpha_locked {
                        out[3] = dst[3];
                    }
                    for k in 0..4 {
                        buf[i + k] = enc(out[k]);
                    }
                }
            }
        }
        self.paint.pilha.planos[pos] = plano;
    }

    /// **A BORRACHA, com a cobertura do traço inteiro.** As duas leis são as de sempre; o que mudou
    /// é que `c` é medido uma vez sobre tudo o que a camada cobriu, em vez de lote a lote.
    fn compoe_borracha(&mut self, pos: usize, r: Region, escopo: EscopoDaBorracha) {
        let plano = std::mem::take(&mut self.paint.pilha.planos[pos]);
        let pre = std::mem::take(&mut self.paint.pilha.pre);
        if plano.len() == self.canvas_rgba.len() && pre.len() == self.canvas_rgba.len() {
            let stride = self.source_size.0 as usize * 4;
            let buf = super::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                self.source_size.0,
                Some(r),
            );
            for row in 0..r.h as usize {
                let base = (r.y as usize + row) * stride + r.x as usize * 4;
                for col in 0..r.w as usize {
                    let i = base + col * 4;
                    // O escudo nasce opaco e o depósito come-lhe o alfa ⇒ `c = 1 − α`.
                    let c = 1.0 - dec(plano[i + 3]);
                    if c <= 0.0 {
                        continue;
                    }
                    match escopo {
                        EscopoDaBorracha::Tudo => {
                            buf[i + 3] = enc(dec(buf[i + 3]) * (1.0 - c));
                        }
                        EscopoDaBorracha::Traco => {
                            for k in 0..4 {
                                let d = f32::from(buf[i + k]);
                                let p = f32::from(pre[i + k]);
                                buf[i + k] = (d + (p - d) * c).round().clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
                }
            }
        }
        self.paint.pilha.pre = pre;
        self.paint.pilha.planos[pos] = plano;
    }

    /// **O BORRÃO, numa passagem só.** ⛔ Divergência declarada — ver o cabeçalho do módulo.
    fn compoe_borrao(&mut self, pos: usize, r: Region) {
        let plano = std::mem::take(&mut self.paint.pilha.planos[pos]);
        if plano.len() == self.canvas_rgba.len() {
            let (w, h) = self.source_size;
            let stride = w as usize * 4;
            // O peso é da REGIÃO, e é `1 − α` do escudo.
            let mut peso = vec![0u8; r.w as usize * r.h as usize];
            let mut algum = false;
            for row in 0..r.h as usize {
                let base = (r.y as usize + row) * stride + r.x as usize * 4;
                for col in 0..r.w as usize {
                    let v = 255 - plano[base + col * 4 + 3];
                    peso[row * r.w as usize + col] = v;
                    algum |= v > 0;
                }
            }
            if algum {
                let raio = self.paint.brush.radius_px * self.tamanho_da_camada(pos);
                let k = ph2d_painter_brush::kernel_radius(raio);
                let passagens = passagens_do_borrao(self.paint.brush.spacing);
                // ⭐ Cada passagem leva a fracção da cobertura que a compõe: `P` passagens de peso
                //   `1 − (1−c)^{1/P}` dão exactamente a cobertura total `c`, e espalham `√P`.
                let inv = 1.0 / passagens as f32;
                let fatia: Vec<u8> = peso
                    .iter()
                    .map(|&c| {
                        let c = f32::from(c) / 255.0;
                        enc(1.0 - (1.0 - c).powf(inv))
                    })
                    .collect();
                let tiling = self.paint.tiling;
                let buf = super::plane_fork::fork_canvas(
                    &mut self.canvas_rgba,
                    &self.undo.write_state,
                    w,
                    Some(r),
                );
                for _ in 0..passagens {
                    ph2d_painter_brush::blur_region_por_peso(
                        buf,
                        w,
                        h,
                        i64::from(r.x),
                        i64::from(r.y),
                        r.w as usize,
                        r.h as usize,
                        k,
                        &fatia,
                        tiling,
                        ph2d_painter_brush::BlurKernel::Caixa,
                    );
                }
            }
        }
        self.paint.pilha.planos[pos] = plano;
    }

    /// **O ESFREGÃO** — inalterado: ele já era um campo por traço resolvido de uma vez, e a base
    /// dele é refrescada com o que as camadas de BAIXO acabaram de deixar na região.
    fn compoe_esfregao(&mut self, pos: usize, r: Region, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        self.refresca_a_base_do_smear(r);
        self.paint.limite_do_smear = Some(r);
        self.aplica_camada(pos, dabs);
        self.paint.limite_do_smear = None;
    }
}

impl PainterTool {
    /// **O avental que a composição tem de cobrir para o borrão ler vizinhança já composta.**
    ///
    /// ⛔⛔ Ele é `k × P` e **não** `k`, e isso custou um gate VERMELHO: com `P` passagens a
    /// convolução alcança `P` raios de núcleo, logo um avental de um raio deixa a orla a ler bytes
    /// que a composição ainda não escreveu — e o resultado passa a depender de **em quantos lotes**
    /// o traço chegou, que é precisamente o que o
    /// `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` proíbe.
    ///
    /// ⚠️ O irmão [`PainterTool::pad_do_nucleo`] (a rota de replay) fica em `k` **e está certo**:
    /// lá cada dab é uma chamada com o avental dela.
    fn pad_do_borrao(&self) -> u32 {
        let k = (0..N_CAMADAS)
            .filter(|&p| {
                self.paint.composite[p].strength > 0.0
                    && matches!(self.paint.composite[p].op, CompositeOp::Blur)
            })
            .map(|p| {
                ph2d_painter_brush::kernel_radius(
                    self.paint.brush.radius_px * self.tamanho_da_camada(p),
                )
            })
            .max()
            .unwrap_or(0);
        if k == 0 {
            return 0;
        }
        (k as u32) * passagens_do_borrao(self.paint.brush.spacing) + 1
    }

    /// O trinco de alfa da camada ACTIVA do documento. ⚠️ Ele é propriedade da camada do
    /// documento, não da posição da pilha — e corre **uma vez**, na composição.
    fn o_alfa_esta_trancado(&self) -> bool {
        self.layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| l.alpha_locked)
    }
}

/// **Quantas passagens o borrão acumulado corre** — a metade que reproduz o ESPALHAMENTO.
///
/// ⛔⛔ Uma passagem só com a cobertura acumulada tem a mistura certa e o espalhamento ERRADO: `N`
/// borrões sequenciais compõem-se num borrão de `σ√N`, e ao olho isso é a borda do traço a esfumar
/// muito mais longe. Medido: com uma passagem só a borda saía visivelmente mais DURA que a do
/// replay — ⚠️ e a régua de nitidez MÉDIA não o via, porque a diferença mora na borda e a média é
/// dominada pelo miolo chapado (*uma régua que agrega a região inteira não vê a borda*).
///
/// ⛔⛔ **E o `n` NÃO se deriva da cobertura, apesar de a álgebra o prometer.** Com `c = 1−(1−w)ⁿ`
/// sairia `n = ln(1−c)/ln(1−w)` — e a força de fábrica leva `c` a **SATURAR** ao primeiro dab, que
/// apaga a informação: medido, a derivação lia `n = 2` onde o traço tinha uma dezena de passagens.
/// *Uma grandeza saturada não se inverte.*
///
/// ⭐ Ele lê-se da GEOMETRIA do pincel: dois dabs consecutivos distam `spacing × diâmetro`, logo um
/// pixel do miolo é coberto por `1/spacing` deles. É um número do PINCEL, não do traço — e é por
/// isso que este custo não cresce com o traço, que é a propriedade inteira desta wave.
///
/// ⚠️ O tecto de `8` é do RELÓGIO e está declarado: cada passagem é `O(região)` com o núcleo de
/// caixa (custo independente do raio).
fn passagens_do_borrao(spacing: f32) -> u32 {
    const TECTO: u32 = 8;
    let n = 1.0 / spacing.max(1e-3);
    (n.round().max(1.0) as u32).min(TECTO)
}
