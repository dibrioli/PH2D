//! ⭐⭐⭐ **A PILHA É DO TRAÇO** — a ordem entre camadas deixa de ser por LOTE.
//!
//! ⛔⛔ **Report do dono, 2026-09-20, três vezes:** *«o brush de cima sempre deve ser desenhado por
//! cima do Brush de baixo. Atualmente o brush de baixo cobre o brush de cima do carimbo anterior»* ·
//! *«Blur em cima não consegue borrar os Brushes»* · *«o Z index não funciona também para eraser em
//! cima… tem que rever para todos»*.
//!
//! # O mecanismo do defeito, medido
//!
//! O [`super::composite::PainterTool::stamp_dabs_composite`] corria `for pos in (0..N).rev()` sobre a
//! fatia de dabs **daquele lote**. A ordem DENTRO de um lote estava certa; entre lotes estava
//! invertida — um traço chega em dezenas de eventos de ponteiro, e o lote `k+1` começa pela camada de
//! BAIXO, que aterra por cima do que a camada de CIMA do lote `k` acabou de pintar. Com o espaçamento
//! de fábrica dois dabs consecutivos sobrepõem-se ~90 %, logo *a camada de cima só sobrevive na lasca
//! da frente*. A régua universal (o mesmo traço numa tacada contra em N) lia `|Δ| médio 41,59` e
//! `pior 92` sobre 255 com dois Brushes de cores opostas.
//!
//! # A lei nova
//!
//! O que a tela mostra a meio de um traço é
//!
//! ```text
//!     tela = L₀( L₁( … L_N( pre ) … ) )
//! ```
//!
//! com **cada `L` aplicada sobre o TRAÇO INTEIRO** e `pre` = a tela no pen-down. Ou seja: a pilha
//! corre **por CAMADA e não por lote** — a camada de baixo carimba toda a história dela, depois a
//! seguinte carimba toda a dela por cima, e assim até ao topo. É a lei que o
//! [`super::stamp_color_cache::PerLayerStroke`] já declara para as camadas de Shape (*«o topo pinta
//! acima de TODA a cobertura acumulada da de baixo ao longo do traço INTEIRO — não por dab, que
//! enterra os realces anteriores»*), aqui um nível acima.
//!
//! # Porque isto NÃO é `O(n²)`
//!
//! Recompor o traço inteiro a cada lote seria `O(n²)` e está fora (medido: um Blur a `4×` custa
//! `~6 ms` por dab; replayar o traço todo dava dezenas de segundos). ⭐ **A recomposição é
//! REGIONAL**: as operações são locais, logo a tela só muda onde os dabs NOVOS caem. Recompõe-se essa
//! região a partir do `pre`, replayando **só os lotes cuja caixa a toca** — um número que não cresce
//! com o traço, ele vale `~2/spacing` (a sobreposição), e é o preço inteiro desta wave.
//!
//! ⚠️ **O que sobra da região fica INTACTO por save/restore**: os dabs replayados escrevem para fora
//! dela (um dab que toca a região tem corpo fora dela), então a tela é guardada antes, recomposta, e
//! só a região nova é escrita de volta. Sem isso cada lote depositaria uma segunda vez na cauda.
//!
//! # As três excepções, cada uma com o motivo
//!
//! * **`Smear`** não é replayado: ele já é um CAMPO por traço (o deslocamento acumulado, resolvido de
//!   uma vez a partir de [`super::warp::session::WarpSession::pre`]). ⭐ E é por isso que **a DOBRA
//!   morreu**: em vez de toda camada não-smear depositar DUAS vezes (na tela e na base congelada do
//!   smear, `lay_into_smear_base`, medido ×2,09), a base do smear é **refrescada da tela** no momento
//!   em que a vez dele chega na pilha. *Isso põe lá exactamente as camadas de BAIXO — que é a ordem
//!   que o report pedia — em vez de todas, que era o defeito.*
//! * **`Blur`** é replayado como um depósito. Ele lê a vizinhança, então a caixa da recomposição é
//!   crescida pelo raio do núcleo.
//! * **O RNG é por CAMADA** (`rng_camada`), e não um fluxo só consumido pela pilha em ordem de lote:
//!   com a recomposição por camada a ordem de consumo muda, e um fluxo partilhado daria realizações
//!   diferentes a cada recomposição — *o Grain Random a cintilar por baixo da mão*. Cada lote guarda
//!   a posição do fluxo de cada camada, e o replay repõe-na.
//!
//! ⛔ **Com MENOS DE DUAS camadas activas nada disto corre** — não há ordem para arrumar, e o caminho
//! é o de sempre, **byte-idêntico** (gate `uma_camada_so_e_byte_identica`).

use super::composite::{CompositeOp, EscopoDaBorracha, N_CAMADAS};
use super::{Region, region::dabs_bounds, region::grow_region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;
use std::sync::Arc;

// ⚠️ **A CONTA da recomposição, por THREAD** — quantos lotes o replay tocou e que área ele
// reescreveu. Um átomo global reprovaria na suíte em paralelo enquanto passa sozinho.
#[cfg(test)]
thread_local! {
    pub(super) static CONTA_DA_PILHA: std::cell::Cell<(u64, u64, u64)> =
        const { std::cell::Cell::new((0, 0, 0)) };
}

// ⚠️ **O interruptor de BISSECÇÃO: recompor o CANVAS INTEIRO em vez da região.**
//
// É o *forced full recomposite* que o [handoff do Per-Layer
// Color](../../../../../docs/Painter/handoffs/HANDOFF_per_layer_color_perf_artifacts.md) §3-1
// prescreve: *«se a recomposição cheia remove a listra, o defeito é um LIMITE rectangular — e é
// preciso descobrir qual»*. Ligado, a janela é o traço inteiro, nada é recortado e a base do
// Smear é refrescada em toda parte. É a referência contra a qual a rota regional se mede.
#[cfg(test)]
thread_local! {
    pub(super) static RECOMPOSICAO_GLOBAL: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

// ⚠️ **As DUAS ablações do Smear, uma de cada vez** — é a atribuição que separa *«o render foi
// recortado»* de *«a base congelada é um mosaico»*. As duas leis vivem no mesmo passo da pilha e
// medi-las juntas responderia sobre as duas ao mesmo tempo.
#[cfg(test)]
thread_local! {
    // `true` = não prender o render do knife à `caixa_nova`.
    pub(super) static SMEAR_SEM_LIMITE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
    // `true` = refrescar a base do knife sobre a `caixa_grande` inteira, não só a `caixa_nova`.
    pub(super) static SMEAR_BASE_GRANDE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

// ⚠️ **O RELÓGIO POR FASE** — `[guardar+pre, Brush, Erase, Smear, Blur, restaurar]`, em
// microssegundos. A fase dominante é quem decide a cura; sem ela optimiza-se a aritmética errada.
#[cfg(test)]
thread_local! {
    pub(super) static FASES_DA_PILHA: std::cell::Cell<[f64; 6]> =
        const { std::cell::Cell::new([0.0; 6]) };
}

// ⚠️ **A ablação do REPLAY** — a janela passa a ser só o lote novo. A imagem fica ERRADA de
// propósito: o que ela mede é o RELÓGIO, ou seja quanto custa replayar a história. É o tecto de
// qualquer cura que substitua o replay por acumulação.
#[cfg(test)]
thread_local! {
    pub(super) static JANELA_SO_O_NOVO: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn marca(i: usize, t: std::time::Instant) {
    FASES_DA_PILHA.with(|c| {
        let mut v = c.get();
        v[i] += t.elapsed().as_secs_f64() * 1e6;
        c.set(v);
    });
}

/// Um lote do traço, **já resolvido por camada**.
///
/// ⚠️ As listas são as do [`PainterTool::camada_dabs`] (escala, cor e a subamostragem por arco), e
/// guardá-las RESOLVIDAS é o que torna o replay possível: aquele acumulador de arco é estado por
/// traço, e recalculá-lo no replay entregaria outra lista de dabs a cada recomposição.
pub(super) struct LoteDaPilha {
    pub(super) camadas: [Vec<Dab>; N_CAMADAS],
    /// A posição do fluxo de RNG **de cada camada** quando este lote correu pela primeira vez.
    pub(super) rng: [u64; N_CAMADAS],
    /// A caixa que este lote escreve (a união das footprints de todas as camadas dele).
    pub(super) caixa: Region,
}

/// O estado da pilha ao longo de UM traço. `pre` vazio = a pilha não está aberta.
#[derive(Default)]
pub(super) struct PilhaDoTraco {
    /// A tela no pen-down — a base de que toda recomposição parte.
    pub(super) pre: Vec<u8>,
    pub(super) lotes: Vec<LoteDaPilha>,
    /// O plano opaco em que a cobertura de uma borracha de escopo `Traco` é MEDIDA (ver
    /// [`PainterTool::aplica_borracha_do_traco`]). Fica aqui para não alocar por lote.
    pub(super) escudo: Arc<Vec<u8>>,
    /// ⭐ **O ACUMULADO de cada camada ao longo do traço** — a rota de omissão desde 2026-09-21.
    /// Ver [`super::composite_acumulado`]; os `lotes` acima só alimentam a rota de bissecção.
    pub(super) planos: [Arc<Vec<u8>>; N_CAMADAS],
}

impl PilhaDoTraco {
    /// O traço acabou (ou nunca abriu): larga tudo.
    pub(super) fn fecha(&mut self) {
        self.pre = Vec::new();
        self.lotes.clear();
        self.escudo = Arc::new(Vec::new());
        for p in &mut self.planos {
            *p = Arc::new(Vec::new());
        }
    }
}

impl PainterTool {
    /// Quantas posições da pilha de facto fazem alguma coisa (`strength > 0`).
    pub(super) fn camadas_activas(&self) -> usize {
        self.paint
            .composite
            .iter()
            .filter(|l| l.strength > 0.0)
            .count()
    }

    /// **A porta da pilha** — ela escolhe entre o motor de ACUMULAÇÃO (a rota de omissão desde
    /// 2026-09-21, [`super::composite_acumulado`]) e o REPLAY regional que ele substituiu.
    ///
    /// ⚠️ A escolha é um CAMPO e não a variável de ambiente: *um gate que lê o ambiente mede a
    /// máquina*. O ambiente só o semeia, uma vez, no arranque.
    pub(super) fn recompoe_a_pilha(&mut self, camadas: [Vec<Dab>; N_CAMADAS]) {
        if self.paint.pilha_por_replay {
            self.recompoe_por_replay(camadas);
        } else {
            self.acumula_e_compoe(camadas);
        }
    }

    /// **A recomposição regional por REPLAY** — a rota que a acumulação substituiu, mantida como
    /// porta de BISSECÇÃO (`PH2D_COMPOSITE_REPLAY=1`).
    ///
    /// ⛔ Ela é **quadrática num rabisco** (medido: `ms/evento` de `2,99` a `12,11` enquanto a
    /// janela vai de `16,7` a `79,4` lotes) — ver [`super::composite_acumulado`] e a
    /// [auditoria](../../../../../docs/Painter/40_auditoria_da_pilha_2026-09-21.md).
    ///
    /// ⚠️ A ordem dos seis passos é load-bearing e cada um está comentado no sítio.
    pub(super) fn recompoe_por_replay(&mut self, camadas: [Vec<Dab>; N_CAMADAS]) {
        let (w, h) = self.source_size;
        let len = (w as usize) * (h as usize) * 4;
        if len == 0 || self.canvas_rgba.len() != len {
            return;
        }
        // 1. A pilha abre no primeiro lote do traço. ⚠️ `pre` é a tela ANTES de qualquer camada
        //    deste traço tocar nela — é dela que toda recomposição parte, e é ela que a borracha de
        //    escopo `Traco` devolve.
        if self.paint.pilha.pre.len() != len {
            self.paint.pilha.pre = (*self.canvas_rgba).clone();
            self.paint.pilha.lotes.clear();
        }
        let Some(caixa_nova) = caixa_das_camadas(&camadas, w, h) else {
            return;
        };
        #[cfg(test)]
        let caixa_nova = if RECOMPOSICAO_GLOBAL.with(std::cell::Cell::get) {
            Region { x: 0, y: 0, w, h }
        } else {
            caixa_nova
        };
        // 2. O lote novo entra na história com a posição do fluxo de RNG de CADA camada.
        let lote = LoteDaPilha {
            rng: self.paint.rng_camada,
            caixa: caixa_nova,
            camadas,
        };
        self.paint.pilha.lotes.push(lote);
        // 3. A JANELA do replay: os lotes cuja caixa toca a do lote novo. ⚠️ O crescimento pelo raio
        //    do núcleo do Blur é o que impede a convolução de ler bytes que a recomposição ainda não
        //    escreveu — sem ele a orla da região lê a tela velha e fica uma costura.
        let pad = self.pad_do_nucleo();
        let alvo = grow_region(caixa_nova, pad, w, h).unwrap_or(caixa_nova);
        let janela: Vec<usize> = (0..self.paint.pilha.lotes.len())
            .filter(|&i| toca(self.paint.pilha.lotes[i].caixa, alvo))
            .collect();
        #[cfg(test)]
        let janela: Vec<usize> = if JANELA_SO_O_NOVO.with(std::cell::Cell::get) {
            vec![self.paint.pilha.lotes.len() - 1]
        } else {
            janela
        };
        let mut caixa_grande = alvo;
        for &i in &janela {
            caixa_grande = union_region(caixa_grande, self.paint.pilha.lotes[i].caixa);
        }
        let caixa_grande = grow_region(caixa_grande, pad, w, h).unwrap_or(caixa_grande);
        #[cfg(test)]
        CONTA_DA_PILHA.with(|c| {
            let (n, lotes, area) = c.get();
            c.set((
                n + 1,
                lotes + janela.len() as u64,
                area + u64::from(caixa_grande.w) * u64::from(caixa_grande.h),
            ));
        });
        // 4. Guardar o que lá está, e pôr o `pre` no lugar dentro da caixa grande.
        //
        // ⚠️ **Guardar a `caixa_grande` chega porque NENHUMA camada escreve fora dela** — e isso
        // custou uma medição: o knife re-resolve o campo acumulado sobre a **união do traço
        // inteiro** a cada lote, logo ele escreveria na cauda a partir de uma base que ali é a de um
        // lote antigo. É o [`Self::limite_do_smear`] que o prende à região recomposta; guardar a
        // união em vez de o limitar também curava a imagem e custava `O(área do traço)` por lote.
        #[cfg(test)]
        let t0 = std::time::Instant::now();
        let guardado = self.save_region(&caixa_grande);
        self.escreve_do_pre(caixa_grande);
        #[cfg(test)]
        marca(0, t0);
        // 5. Correr a pilha POR CAMADA (baixo → topo), cada uma sobre toda a história dela na janela.
        for pos in (0..N_CAMADAS).rev() {
            if self.paint.composite[pos].strength <= 0.0 {
                continue;
            }
            self.limpa_cap_da_camada(pos, caixa_grande);
            if matches!(self.paint.composite[pos].op, CompositeOp::Smear) {
                // ⭐ A base congelada do knife passa a ser **o que está por baixo dele**, e mais nada:
                //    a tela, agora, tem exactamente as camadas inferiores sobre o traço inteiro.
                #[cfg(test)]
                let base_r = if SMEAR_BASE_GRANDE.with(std::cell::Cell::get) {
                    caixa_grande
                } else {
                    caixa_nova
                };
                #[cfg(not(test))]
                let base_r = caixa_nova;
                self.refresca_a_base_do_smear(base_r);
                let ultimo = *janela.last().unwrap_or(&0);
                let dabs = std::mem::take(&mut self.paint.pilha.lotes[ultimo].camadas[pos]);
                #[cfg(test)]
                let limite = (!SMEAR_SEM_LIMITE.with(std::cell::Cell::get)).then_some(caixa_nova);
                #[cfg(not(test))]
                let limite = Some(caixa_nova);
                self.paint.limite_do_smear = limite;
                #[cfg(test)]
                let ts = std::time::Instant::now();
                self.aplica_camada(pos, &dabs);
                #[cfg(test)]
                marca(3, ts);
                self.paint.limite_do_smear = None;
                self.paint.pilha.lotes[ultimo].camadas[pos] = dabs;
                continue;
            }
            #[cfg(test)]
            let fase = match self.paint.composite[pos].op {
                CompositeOp::Brush => 1,
                CompositeOp::Erase => 2,
                CompositeOp::Smear => 3,
                CompositeOp::Blur => 4,
            };
            for &i in &janela {
                let dabs = std::mem::take(&mut self.paint.pilha.lotes[i].camadas[pos]);
                self.paint.tex_rng = self.paint.pilha.lotes[i].rng[pos];
                #[cfg(test)]
                let tc = std::time::Instant::now();
                self.aplica_camada(pos, &dabs);
                #[cfg(test)]
                marca(fase, tc);
                self.paint.pilha.lotes[i].camadas[pos] = dabs;
            }
            // O fluxo desta camada fica onde o lote NOVO o deixou.
            self.paint.rng_camada[pos] = self.paint.tex_rng;
        }
        // 6. Só a região nova sobrevive; o resto volta a ser o que era.
        #[cfg(test)]
        let t5 = std::time::Instant::now();
        let recomposto = self.save_region(&caixa_nova);
        self.escreve_regiao(caixa_grande, &guardado);
        self.escreve_regiao(caixa_nova, &recomposto);
        #[cfg(test)]
        marca(5, t5);
        self.declare_wrote(Some(caixa_grande));
        self.mark_dirty(caixa_grande);
    }

    /// O apron que a recomposição tem de cobrir para o Blur ler vizinhança já recomposta.
    pub(super) fn pad_do_nucleo(&self) -> u32 {
        let maior = (0..N_CAMADAS)
            .filter(|&p| {
                self.paint.composite[p].strength > 0.0
                    && matches!(self.paint.composite[p].op, CompositeOp::Blur)
            })
            .map(|p| self.paint.brush.radius_px * self.tamanho_da_camada(p))
            .fold(0.0f32, f32::max);
        // O raio do núcleo é fracção do raio do dab; o `+1` é a borda de amostragem.
        (maior * 0.5).ceil().max(0.0) as u32 + 1
    }

    /// Escrever o `pre` dentro de `r` — o passo que apaga o traço inteiro daquela região para ele
    /// ser reconstruído na ordem certa.
    pub(super) fn escreve_do_pre(&mut self, r: Region) {
        let stride = self.source_size.0 as usize * 4;
        let rw = r.w as usize * 4;
        let pre = std::mem::take(&mut self.paint.pilha.pre);
        {
            let buf = super::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                self.source_size.0,
                Some(r),
            );
            for row in 0..r.h as usize {
                let o = (r.y as usize + row) * stride + r.x as usize * 4;
                buf[o..o + rw].copy_from_slice(&pre[o..o + rw]);
            }
        }
        self.paint.pilha.pre = pre;
    }

    /// Escrever de volta os bytes que [`PainterTool::save_region`] tirou.
    pub(super) fn escreve_regiao(&mut self, r: Region, pixels: &[u8]) {
        let stride = self.source_size.0 as usize * 4;
        let rw = r.w as usize * 4;
        let buf = super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            Some(r),
        );
        for row in 0..r.h as usize {
            let dst = (r.y as usize + row) * stride + r.x as usize * 4;
            buf[dst..dst + rw].copy_from_slice(&pixels[row * rw..row * rw + rw]);
        }
    }

    /// O cap de Accumulate desta camada é reconstruído pelo replay ⇒ ele tem de começar a ZERO
    /// dentro da região recomposta.
    ///
    /// ⚠️ Fora dela o cap fica como estava, e **está certo**: um pixel que sobrevive fora da região
    /// guarda a tinta que o cap dele descreve. A cada lote a região limpa-se e reconstrói-se do
    /// replay, então *o cap está exacto exactamente onde o resultado é conservado*.
    fn limpa_cap_da_camada(&mut self, pos: usize, r: Region) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let m = &mut self.paint.composite_mask[pos];
        if m.len() != n {
            return;
        }
        for row in 0..r.h as usize {
            let o = (r.y as usize + row) * w as usize + r.x as usize;
            m[o..o + r.w as usize].fill(0);
        }
    }

    /// A base congelada do Smear passa a ser a tela **como as camadas de baixo a deixaram**.
    ///
    /// ⭐⭐ Isto SUBSTITUI a DOBRA (`lay_into_smear_base`, apagada nesta wave): ela depositava toda
    /// camada não-smear duas vezes — na tela e na base —, o que custava ×2,09 num Brush e ×1,92 num
    /// Blur **e punha na base também as camadas de CIMA**, que é o defeito de ordem do report.
    pub(super) fn refresca_a_base_do_smear(&mut self, r: Region) {
        if !self.paint.warp.active || self.paint.warp.pre.len() != self.canvas_rgba.len() {
            return;
        }
        let stride = self.source_size.0 as usize * 4;
        let rw = r.w as usize * 4;
        let base = Arc::make_mut(&mut self.paint.warp.pre);
        for row in 0..r.h as usize {
            let o = (r.y as usize + row) * stride + r.x as usize * 4;
            base[o..o + rw].copy_from_slice(&self.canvas_rgba[o..o + rw]);
        }
    }

    /// **A borracha de escopo `Traco`** — ela devolve o pixel ao que ele era ANTES do traço, em vez
    /// de comer a imagem por baixo.
    ///
    /// ⭐ Ordem do dono, 2026-09-20: *«uma opção em erase: se a borracha atua só no próprio traço do
    /// Brush ou se ela apaga também a camada da imagem abaixo»*.
    ///
    /// # A aritmética, e porque `lerp` para o `pre` é EXACTO
    ///
    /// No ponto da pilha em que a borracha corre, a tela é `pre ⊕ tinta`. O que se quer é
    /// `pre ⊕ (tinta·(1−c))`, com `c` a cobertura da borracha. Desenvolvendo o `over` (alfa recta,
    /// `pre` com alfa `p`, tinta com alfa `t`), o resultado é **idêntico** a
    /// `lerp(tela, pre, c)` — canal a canal, alfa incluído. *Uma lei que vale por álgebra, não por
    /// aproximação* (gate `a_borracha_do_traco_devolve_o_pre`).
    ///
    /// # Porque a cobertura é MEDIDA e não recuperada do alfa
    ///
    /// Recuperá-la de `c = 1 − α_depois/α_antes` funciona… menos onde `α_antes = 0`, que é
    /// exactamente o pixel que uma borracha de escopo `Tudo` numa camada de baixo acabou de esvaziar.
    /// ⇒ ela corre no **ESCUDO**: um plano opaco onde o MESMO depósito escreve, e `c = 1 − α`. Uma
    /// lei, uma porta, e nenhum caso degenerado.
    pub(super) fn aplica_borracha_do_traco(&mut self, dabs: &[Dab]) {
        let (w, h) = self.source_size;
        let len = (w as usize) * (h as usize) * 4;
        if self.paint.pilha.pre.len() != len || self.canvas_rgba.len() != len {
            // Sem `pre` não há para onde devolver — a camada cai no escopo de sempre.
            self.aplica_deposito(dabs);
            return;
        }
        let Some(r) = dabs_bounds(dabs, w, h) else {
            return;
        };
        // 1. O escudo opaco, limpo dentro da região.
        if self.paint.pilha.escudo.len() != len {
            self.paint.pilha.escudo = Arc::new(vec![255u8; len]);
        }
        {
            let stride = w as usize * 4;
            let e = Arc::make_mut(&mut self.paint.pilha.escudo);
            for row in 0..r.h as usize {
                let o = (r.y as usize + row) * stride + r.x as usize * 4;
                e[o..o + r.w as usize * 4].fill(255);
            }
        }
        // 2. O MESMO depósito, sobre o escudo. ⚠️ Pela porta de troca de plano, senão o journal do
        //    undo captura os bytes do escudo achando que são a tela — e a primeira captura de cada
        //    tile é a que vale, logo a poluição seria permanente.
        let mut plano = std::mem::take(&mut self.paint.pilha.escudo);
        super::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plano,
            &self.undo.write_state,
        );
        let saved_draw = self.paint.brush.impasto_draw_to;
        self.paint.brush.impasto_draw_to = ph2d_painter_brush::DrawTo::Color;
        self.aplica_deposito(dabs);
        self.paint.brush.impasto_draw_to = saved_draw;
        super::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plano,
            &self.undo.write_state,
        );
        self.paint.pilha.escudo = plano;
        // 3. `tela := lerp(tela, pre, c)`, com `c = 1 − α_escudo`.
        let stride = w as usize * 4;
        let escudo = std::mem::take(&mut self.paint.pilha.escudo);
        let pre = std::mem::take(&mut self.paint.pilha.pre);
        {
            let buf = super::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                w,
                Some(r),
            );
            for row in 0..r.h as usize {
                let base = (r.y as usize + row) * stride + r.x as usize * 4;
                for col in 0..r.w as usize {
                    let i = base + col * 4;
                    let c = 255.0 - f32::from(escudo[i + 3]);
                    if c <= 0.0 {
                        continue;
                    }
                    let c = c / 255.0;
                    for k in 0..4 {
                        let d = f32::from(buf[i + k]);
                        let p = f32::from(pre[i + k]);
                        buf[i + k] = (d + (p - d) * c).round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
        self.paint.pilha.pre = pre;
        self.paint.pilha.escudo = escudo;
        self.declare_wrote(Some(r));
        self.mark_dirty(r);
    }

    /// O depósito nu (a rota do Brush), com o Tiling aplicado — partilhado pela camada Brush, pela
    /// borracha de escopo `Tudo` e pela medição do escudo.
    pub(super) fn aplica_deposito(&mut self, dabs: &[Dab]) {
        let tiling = self.paint.tiling;
        if tiling[0] || tiling[1] {
            let wrapped = super::tiling::tiled_dabs(dabs, self.source_size, tiling);
            self.stamp_dabs_inner(&wrapped);
        } else {
            self.stamp_dabs_inner(dabs);
        }
    }

    /// Correr UMA camada sobre a lista dela — a porta ÚNICA de *«o que esta posição faz»*, lida
    /// tanto pelo caminho de sempre como pela recomposição.
    pub(super) fn aplica_camada(&mut self, pos: usize, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        let layer = self.paint.composite[pos];
        let (w, h) = self.source_size;
        let saved_strength = self.paint.brush.strength;
        let saved_blend = self.paint.brush.blend;
        // ⭐ A dureza é reposta como a força: ela vive no `BrushSpec` e é lida no CARIMBO (o perfil
        //   do dab), logo trocá-la à volta da passagem chega — não há nada assado na lista de dabs.
        let saved_hardness = self.paint.brush.hardness;
        self.paint.brush.strength = layer.strength;
        // `None` = segue o pincel, e aí a linha é um no-op AO BIT (escreve o que já lá estava).
        self.paint.brush.hardness = layer.hardness.unwrap_or(saved_hardness);
        // ⚠️ O blend da camada de apagar é forçado AQUI e não na rota: a rota é a do depósito, e
        // ela já sabe ler `brush.blend`. Uma rota própria seria a segunda resposta.
        self.paint.brush.blend = if matches!(layer.op, CompositeOp::Erase) {
            ph2d_painter_brush::BrushBlend::EraseAlpha
        } else {
            saved_blend
        };
        std::mem::swap(
            &mut self.paint.stroke_mask,
            &mut self.paint.composite_mask[pos],
        );
        match layer.op {
            CompositeOp::Brush => self.aplica_deposito(dabs),
            CompositeOp::Erase => match layer.erase_scope {
                EscopoDaBorracha::Tudo => self.aplica_deposito(dabs),
                EscopoDaBorracha::Traco => self.aplica_borracha_do_traco(dabs),
            },
            CompositeOp::Smear => self.stamp_dabs_smear(dabs, w, h),
            CompositeOp::Blur => {
                // ⭐ **E aqui — e SÓ aqui — o núcleo é o de CAIXA** (ordem do dono, 2026-09-20:
                // *«veja se abaixando a qualidade do blur não fica bem mais leve; mas só no Blur do
                // composite»*). A ferramenta Blur isolada entra pela porta sem argumento, que crava
                // o binomial.
                self.stamp_dabs_blur_com(dabs, w, h, ph2d_painter_brush::BlurKernel::Caixa);
            }
        }
        std::mem::swap(
            &mut self.paint.stroke_mask,
            &mut self.paint.composite_mask[pos],
        );
        self.paint.brush.blend = saved_blend;
        self.paint.brush.strength = saved_strength;
        self.paint.brush.hardness = saved_hardness;
    }
}

/// A união das footprints de todas as camadas de um lote.
pub(super) fn caixa_das_camadas(camadas: &[Vec<Dab>; N_CAMADAS], w: u32, h: u32) -> Option<Region> {
    let mut acc: Option<Region> = None;
    for lista in camadas {
        if let Some(r) = dabs_bounds(lista, w, h) {
            acc = Some(acc.map_or(r, |a| union_region(a, r)));
        }
    }
    acc
}

/// Dois rectângulos meio-abertos que se intersectam.
fn toca(a: Region, b: Region) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}
