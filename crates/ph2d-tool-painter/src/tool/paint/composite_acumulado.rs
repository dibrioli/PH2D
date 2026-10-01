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
//! # Exacto em três das quatro operações, e a quarta é DECLARADA
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
        #[cfg(test)]
        let t_pre = std::time::Instant::now();
        if self.paint.pilha.pre.len() != len {
            self.paint.pilha.pre = (*self.canvas_rgba).clone();
            self.paint.pilha.lotes.clear();
            for p in &mut self.paint.pilha.planos {
                *p = Arc::new(Vec::new());
            }
            self.paint.pilha.relevo = Default::default();
        }
        #[cfg(test)]
        fases::soma(fases::PRE, t_pre);
        let Some(caixa_nova) =
            super::region::caixa_das_camadas(&camadas, (w, h), self.paint.tiling)
        else {
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
        #[cfg(test)]
        {
            super::composite_pilha::conta_evento();
            super::composite_pilha::conta_dabs(camadas.iter().map(Vec::len).sum::<usize>() as u64);
        }
        // 2. Cada camada acumula os dabs NOVOS dela no plano dela — e, quando o hospedeiro drena
        //    por quadro, NÃO AGORA: o lote espera na fila da camada e o quadro inteiro acumula de
        //    uma vez ([`super::composite_por_quadro`], 2026-10-01). A composição só lê os planos
        //    no quadro, logo ninguém vê a diferença — e um lote de quadro é o que alcança a rota
        //    em BANDAS do depósito, onde o pingo de um evento ficava sempre abaixo do piso dela.
        if self.pilha_pode_adiar() {
            #[cfg(test)]
            if ACUMULA_NO_EVENTO.with(std::cell::Cell::get) {
                self.acumula_as_camadas(&camadas);
            }
            self.adia_a_composicao(caixa_nova, camadas);
            return;
        }
        // ⛔ Um lote que não pode esperar NÃO passa à frente dos que esperam: o plano de cada
        //    camada é a soma dos dabs dela EM ORDEM, e acumular este antes dos de trás trocava-a.
        self.compoe_o_pendente();
        self.acumula_as_camadas(&camadas);
        self.compoe_a_regiao(caixa_nova, &camadas);
    }

    /// **O passo 2 da lei: cada camada viva acumula a fila dela no plano dela**, pela ordem da
    /// lista — o relevo primeiro, porque ele lê uma CÓPIA do fluxo aleatório da camada e a cor
    /// consome-o. A porta das duas rotas: o evento (sem drenagem por quadro) e a drenagem.
    pub(super) fn acumula_as_camadas(&mut self, camadas: &[Vec<Dab>; N_CAMADAS]) {
        #[cfg(test)]
        let t_acumular = std::time::Instant::now();
        for (pos, lista) in camadas.iter().enumerate() {
            if !self.camada_viva(pos) || lista.is_empty() {
                continue;
            }
            // O RELEVO primeiro: ele lê uma CÓPIA do fluxo aleatório da camada, a cor consome-o.
            #[cfg(test)]
            let t_sub = std::time::Instant::now();
            self.relevo_da_camada(pos, lista);
            #[cfg(test)]
            fases::soma_sub(0, t_sub);
            #[cfg(test)]
            let t_sub = std::time::Instant::now();
            self.acumula_camada(pos, lista);
            #[cfg(test)]
            fases::soma_sub(
                match self.paint.composite[pos].op {
                    CompositeOp::Brush => 1,
                    CompositeOp::Erase => 2,
                    _ => 3,
                },
                t_sub,
            );
        }
        #[cfg(test)]
        fases::soma(fases::ACUMULAR, t_acumular);
    }

    /// O pendente chegou à drenagem ainda por ACUMULAR? Sempre, no produto; o controlo de teste
    /// [`ACUMULA_NO_EVENTO`] devolve a rota de antes, que acumulava em cada evento.
    pub(super) fn o_acumulo_esperou(&self) -> bool {
        #[cfg(test)]
        if ACUMULA_NO_EVENTO.with(std::cell::Cell::get) {
            return false;
        }
        true
    }

    /// **Compor a caixa `caixa_nova` a partir do `pre` e dos planos** — os passos 3 e 4 da lei, e
    /// a porta que o [`Self::compoe_o_pendente`] também chama (a união de um quadro é uma caixa
    /// como outra qualquer: tudo o que ela lê já está nos planos).
    pub(super) fn compoe_a_regiao(&mut self, caixa_nova: Region, camadas: &[Vec<Dab>; N_CAMADAS]) {
        let (w, h) = self.source_size;
        // 3. A região da composição. ⚠️ O apron do Blur é o que impede a convolução de ler, na orla,
        //    bytes que a composição ainda não escreveu — e é por isso que só o miolo sobrevive.
        let pad = self.pad_do_borrao();
        // ⭐⭐ **A caixa ESCRITA é a do lote ALARGADA pelo alcance do borrão** (fila 44, item 9).
        //
        // Um pixel que o borrão já borrou num evento anterior, FORA da caixa deste, lê vizinhos que
        // este evento mudou: a saída dele mudou e ninguém a reescrevia. Era isso o «carimbo
        // rectangular» que sobrava (`12`–`20` de 255 com Blur na pilha, contra `0` recompondo o
        // canvas inteiro) — e **não** a base congelada do esfregão, a que a auditoria de 21/09 o
        // atribuiu: com a escrita alargada os seis arranjos medidos vão a `0`, com o Smear em baixo
        // E por cima de um Brush. ⇒ reescreve-se a caixa + o alcance, e compõe-se mais um alcance
        // à volta para ela ler. Sem borrão vivo o `pad` é `0` e nada muda.
        //
        // ⭐⭐ **E o que MUDOU inclui a área que o ESFREGÃO já tocou**, quando uma camada viva por
        // baixo dele muda a tinta que ele puxa (report do dono, 2026-09-30, com foto: *«cria
        // rectângulos de cor»*). O esfregão lê cada pixel de LONGE — `base(p − disp(p))` —, e um
        // pixel já esfregado fora da caixa deste lote continuava a mostrar a tinta que o Brush de
        // baixo tinha ali ANTES: o Brush mudou a origem, e ninguém reescrevia o destino. Medido
        // (Smear/Brush, rabisco, a região contra o canvas inteiro): pior `255` em `18 569` px ⇒
        // `0`. Reescreve-se quem LÊ da caixa (`quem_le_da_caixa`), não a área tocada inteira (que
        // também curava e custava `10,5 → 16,0 ms` por quadro num rabisco; o preciso custa
        // `11,1 → 12,7`). O alcance do borrão aplica-se DEPOIS da união, senão um borrão por cima
        // fica velho na orla dela (`47` ⇒ `0`). Com o esfregão no fundo a base é o `pre` e não muda.
        // ⚠️ Sem borrão por baixo do esfregão a tinta que ele lê só muda na caixa do lote (com um,
        // `quem_le_da_caixa` devolve a área tocada inteira — ver lá porquê).
        let mudou = match self.quem_le_da_caixa(caixa_nova) {
            Some(d) => super::union_region(caixa_nova, d),
            None => caixa_nova,
        };
        #[cfg(test)]
        let mudou = if ESFREGAO_SO_O_LOTE.with(std::cell::Cell::get) {
            caixa_nova
        } else {
            mudou
        };
        let escrita = super::region::grow_region(mudou, pad, w, h).unwrap_or(mudou);
        #[cfg(test)]
        let escrita = if ESCRITA_ESTREITA.with(std::cell::Cell::get) {
            caixa_nova
        } else {
            escrita
        };
        let caixa_nova = escrita;
        let alvo = super::region::grow_region(caixa_nova, pad, w, h).unwrap_or(caixa_nova);
        // 4. Guardar a orla, compor, e devolver tudo o que não é a caixa nova.
        #[cfg(test)]
        let t_copias = std::time::Instant::now();
        let guardado = self.save_region(&alvo);
        #[cfg(test)]
        fases::soma(fases::COPIAS, t_copias);
        #[cfg(test)]
        let t_compor = std::time::Instant::now();
        self.compoe_a_pilha(alvo, caixa_nova, camadas);
        // O CORPO do traço, recomposto de baixo para cima quando uma borracha o apaga (8b) ou um
        // esfregão o arrasta — DEPOIS da cor, que é onde o esfregão acumula o deslocamento do lote.
        self.compoe_o_corpo(caixa_nova, camadas);
        #[cfg(test)]
        fases::soma(fases::COMPOR, t_compor);
        #[cfg(test)]
        let t_copias2 = std::time::Instant::now();
        let composto = self.save_region(&caixa_nova);
        self.escreve_regiao(alvo, &guardado);
        self.escreve_regiao(caixa_nova, &composto);
        #[cfg(test)]
        fases::soma(fases::COPIAS, t_copias2);
        #[cfg(test)]
        fases::conta_area(alvo, w, h);
        #[cfg(test)]
        fases::conta_cobertura(camadas, pad, alvo, w, h);
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
        #[cfg(test)]
        fases::conta_acumulo();
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
        self.paint.brush.strength = layer.strength;
        self.paint.brush.hardness = layer.hardness.unwrap_or(saved_hardness);
        // ⚠️ A TINTA acumula-se com `Mix` e **não** com o blend do pincel: o blend é da CAMADA e
        // corre uma vez, na composição. Acumular com ele aplicá-lo-ia uma vez por dab.
        self.paint.brush.blend = match modo {
            Acumulo::Tinta => BrushBlend::Mix,
            _ => BrushBlend::EraseAlpha,
        };
        // ⛔ O relevo não entra num plano de medição — ele é da TELA, e quem o impede é o
        // `acumulando_no_plano` no despacho. O `Draw To` do artista FICA: é ele que corta o pigmento
        // de um pincel que deposita corpo num FILME, e forçá-lo a `Color` aqui pintava a camada com a
        // tinta cheia do Digital (sonda `diag_a_cor_da_pilha_no_impasto`: pior `178` contra o avulso).
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
        self.paint.brush.blend = saved_blend;
        self.paint.brush.strength = saved_strength;
        self.paint.brush.hardness = saved_hardness;
    }

    /// **Uma camada está VIVA** — a pergunta que o laço da composição, a acumulação, a cerca do
    /// borrão e o avental fazem, e que tem de ter UMA resposta.
    ///
    /// ⚠️ Ela era escrita quatro vezes, duas como `strength <= 0.0` (salta) e duas como
    /// `strength > 0.0` (conta), e as duas formas **discordam em `NaN`**: o laço compunha uma camada
    /// em `NaN` enquanto a cerca e o avental a ignoravam — um esfregão em `NaN` seria composto sem
    /// a orla borrada, e um borrão em `NaN` correria com avental `0`. O painel não produz um `NaN`
    /// (o commit do número recusa não-finitos), mas o `set_composite_layer_strength` é público e o
    /// `clamp` deixa-o passar. ⇒ uma porta, e `NaN` está MORTA (`NaN > 0` é falso).
    pub(super) fn camada_viva(&self, pos: usize) -> bool {
        self.paint.composite[pos].strength > 0.0
    }

    /// ⭐⭐⭐ **O BORRÃO só precisa de LER a orla; ESCREVÊ-LA é trabalho que se deita fora.**
    ///
    /// A composição corre sobre `alvo = caixa_nova + 2·pad` porque o borrão tem de ler vizinhança
    /// já composta — e depois **só `caixa_nova` é guardada**. Medido na pilha do dono (2026-09-22,
    /// report *«ainda está lenta e engasgando com pincel grande»*): `alvo` mede `857 px` de lado
    /// contra `343` de `caixa_nova`, ou seja **`6,2×` da área é calculada e deitada fora**, e o
    /// borrão é a operação que a paga mais cara (`26,3 Mpx` contra `2,0` do mesmo borrão sozinho).
    ///
    /// ⇒ o borrão passa a ser **APLICADO** sobre `caixa_nova`. O avental que ele precisa de ler
    /// continua a ser lido — a [`ph2d_painter_brush::blur_region_por_peso`] lê a vizinhança dela
    /// própria, a partir da tela, que os passos de baixo compuseram sobre `alvo`.
    ///
    /// ⛔⛔ **A CERCA:** quem lê a SAÍDA do borrão fora de `caixa_nova` é uma camada ACIMA dele que
    /// leia VIZINHANÇA — outro borrão, ou um esfregão, que desloca píxeis. Com uma dessas acima, a
    /// orla borrada é entrada de alguém e tem de existir ⇒ ali fica o `alvo` de sempre. *Uma camada
    /// por-pixel (Brush, Erase) nunca lê o vizinho, logo não vê a diferença.*
    ///
    /// ⚠️ **A cura NÃO é byte-idêntica ao código de antes, e isto é declarado** (esta nota dizia
    /// o contrário até à auditoria de 2026-09-23): o borrão de uma sub-região não é o miolo do
    /// borrão da região maior — a soma corrente carrega o sítio onde começou (`~6e-4` em `f32`) —,
    /// logo um pixel em `~0,06 %` pode mudar UM byte. O que a cerca garante é que a orla que alguém
    /// LÊ foi borrada; a barra do `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` é esse byte.
    ///
    /// ⚠️ O braço `Blur` é hoje inalcançável pelo painel (a quota é de UM borrão), e fica porque é
    /// a resposta certa no dia em que a quota subir — com o gate `dois_borroes_empilhados_…` a
    /// segurá-lo, já que o avental também passou a ser a SOMA.
    fn alguem_acima_le_vizinhanca(&self, pos: usize) -> bool {
        #[cfg(test)]
        match CERCA_DO_BORRAO.with(std::cell::Cell::get) {
            CERCA_DESLIGADA => return false,
            CURA_REVERTIDA => return true,
            _ => {}
        }
        (0..pos).any(|p| {
            self.camada_viva(p)
                && matches!(
                    self.paint.composite[p].op,
                    CompositeOp::Blur | CompositeOp::Smear
                )
        })
    }

    /// **A composição: `pre` e depois cada camada UMA vez, de baixo para cima.**
    fn compoe_a_pilha(&mut self, r: Region, caixa_nova: Region, camadas: &[Vec<Dab>; N_CAMADAS]) {
        #[cfg(test)]
        let t_op = std::time::Instant::now();
        self.escreve_do_pre(r);
        #[cfg(test)]
        fases::soma_op(fases::OP_PRE, t_op);
        for pos in (0..N_CAMADAS).rev() {
            let layer = self.paint.composite[pos];
            if !self.camada_viva(pos) {
                continue;
            }
            #[cfg(test)]
            let t_op = std::time::Instant::now();
            match layer.op {
                CompositeOp::Brush => self.compoe_tinta(pos, r),
                CompositeOp::Erase => self.compoe_borracha(pos, r, layer.erase_scope),
                CompositeOp::Blur => {
                    let alvo_do_borrao = if self.alguem_acima_le_vizinhanca(pos) {
                        r
                    } else {
                        caixa_nova
                    };
                    self.compoe_borrao(pos, alvo_do_borrao);
                }
                CompositeOp::Smear => self.compoe_esfregao(pos, r, &camadas[pos]),
            }
            #[cfg(test)]
            fases::soma_op(fases::op_de(layer.op), t_op);
        }
    }
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
            super::composite_linhas::tinta(buf, &plano, stride, r, blend, alpha_locked);
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
            super::composite_linhas::borracha(buf, &plano, &pre, stride, r, escopo);
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
            #[cfg(test)]
            let t_sub = std::time::Instant::now();
            let (peso, algum) = super::composite_linhas::peso_do_borrao(&plano, stride, r);
            #[cfg(test)]
            fases::soma_sub(4, t_sub);
            #[cfg(test)]
            let t_sub = std::time::Instant::now();
            if algum {
                let raio = self.paint.brush.radius_px * self.tamanho_da_camada(pos);
                // ⭐⭐ **UMA passagem de raio `P·k`, e não `P` passagens de raio `k`.** A variância
                //    do núcleo é LINEAR no raio (`σ² = k/2`, ver `blur_caixa`), logo `P` passagens
                //    compõem-se numa de raio `P·k` — e o núcleo de CAIXA custa o mesmo em qualquer
                //    raio (três somas correntes). ⇒ o espalhamento é o mesmo e o trabalho é `1/P`.
                let k = ph2d_painter_brush::kernel_radius(raio)
                    * passagens_do_borrao(self.paint.brush.spacing) as usize;
                let tiling = self.paint.tiling;
                let buf = super::plane_fork::fork_canvas(
                    &mut self.canvas_rgba,
                    &self.undo.write_state,
                    w,
                    Some(r),
                );
                // A convolução e a mistura de volta em DOIS passos (2026-10-01): a mistura é por
                // pixel em linhas disjuntas e corre na equipa de threads (ADR-0172); a lei dela é
                // UMA, a [`ph2d_painter_brush::mistura_linha_por_peso`], que a porta em série
                // [`ph2d_painter_brush::blur_region_por_peso`] também chama.
                let borrada = ph2d_painter_brush::blur_region_borrado(
                    buf,
                    w,
                    h,
                    i64::from(r.x),
                    i64::from(r.y),
                    r.w as usize,
                    r.h as usize,
                    k,
                    tiling,
                    ph2d_painter_brush::BlurKernel::Caixa,
                );
                #[cfg(test)]
                fases::soma_sub(5, t_sub);
                #[cfg(test)]
                let t_sub = std::time::Instant::now();
                super::composite_linhas::mistura_do_borrao(buf, &borrada, &peso, stride, r);
                #[cfg(test)]
                fases::soma_sub(8, t_sub);
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
        self.aplica_camada(pos, dabs);
        #[cfg(test)]
        fases::soma_sub(7, t_sub);
        self.paint.limite_do_smear = None;
    }
}

// ⚠️ **As duas ablações da cerca do borrão, só de teste** — o CONTROLO dos gates de valor da cura:
// `CERCA_DESLIGADA` aplica sempre na `caixa_nova` (a cerca apagada), `CURA_REVERTIDA` aplica sempre
// no `alvo` (o código de antes de 2026-09-22). *Um gate de valor sem o controlo que reprova não
// prova que a fixtura contém o fenómeno.*
#[cfg(test)]
pub(super) const CERCA_DESLIGADA: u8 = 1;
#[cfg(test)]
pub(super) const CURA_REVERTIDA: u8 = 2;
#[cfg(test)]
thread_local! {
    pub(super) static CERCA_DO_BORRAO: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
    /// `true` = o avental de antes de 2026-09-23 (`k·P + 1`) em toda pilha — o CONTROLO do gate
    /// que prova que o avental estreito dá a mesma imagem.
    pub(super) static AVENTAL_LARGO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `true` = escrever só a caixa do lote (o código de antes do item 9) — o CONTROLO do gate
    /// `a_regiao_nao_deixa_rectangulo`.
    pub(super) static ESCRITA_ESTREITA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `true` = o que mudou é só a caixa do lote, mesmo com o esfregão a ler uma base que muda
    /// (o código de antes de 2026-09-30) — o CONTROLO do gate dos rectângulos do esfregão.
    pub(super) static ESFREGAO_SO_O_LOTE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `true` = acumular cada lote no EVENTO, mesmo com a composição adiada (o código de antes de
    /// 2026-10-01) — o CONTROLO do gate que prova que acumular por quadro dá a mesma imagem.
    pub(super) static ACUMULA_NO_EVENTO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

impl PainterTool {
    /// A porta do gate — o produto lê o irmão privado.
    #[cfg(test)]
    pub(super) fn pad_do_borrao_para_teste(&self) -> u32 {
        self.pad_do_borrao()
    }

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
    ///
    /// ⛔ **É a SOMA dos núcleos e não o MÁXIMO** (auditoria de 2026-09-23): com dois borrões
    /// empilhados, a saída do de baixo só é exacta até ao alcance do de cima a partir da caixa, e
    /// cada um desses píxeis lê mais o alcance do de baixo — os avental compõem-se. Com a quota de
    /// UM borrão (`quota_da_operacao`) a soma É o máximo, ao bit; o que a troca compra é que subir
    /// a quota deixa de partir isto em silêncio.
    ///
    /// ⭐⭐⭐ **E ele é o ALCANCE do núcleo de CAIXA, não o `k·P` do binomial** (2026-09-23, achado
    /// da auditoria por uma mutação sobrevivente). O composite borra com
    /// [`ph2d_painter_brush::BlurKernel::Caixa`], e a caixa lê `Σ box_radii(k·P)` à volta da
    /// região — `~1,5·√(2kP)`, contra os `k·P` que o avental pagava. Na pilha do dono:
    /// `k·P = 32·8 = 256` ⇒ avental **`257 px`** contra um alcance real de **`32`**, e a composição
    /// de TODAS as camadas corria sobre essa área (`92,7 %` do traço).
    ///
    /// ⛔⛔ **O avental ESTREITO só vale quando ninguém acima de um borrão lê vizinhança.** Com um
    /// esfregão (ou outro borrão) acima, a cerca devolve ao borrão o `alvo` inteiro, e quem está
    /// acima lê a SAÍDA dele deslocada — o esfregão não tem tecto de transporte, logo nenhum alcance
    /// o limita, e hoje é a folga do avental largo que o cobre. Ali fica o avental de antes, **ao
    /// bit** (gate `com_um_esfregao_por_cima_a_cura_nao_muda_um_bit`).
    ///
    /// ⭐ **Porque o estreito é EXACTO no outro caso:** abaixo do borrão só há leis por-pixel (Brush,
    /// Erase) e o esfregão; o borrão lê a pegada mais o alcance, e é tudo o que tem de estar composto.
    /// A base do esfregão fora do `alvo` não fica velha: ela só muda onde os planos por baixo dele
    /// mudam, que é a `caixa_nova` de cada evento — sempre dentro do `alvo` desse evento.
    fn pad_do_borrao(&self) -> u32 {
        let passagens = passagens_do_borrao(self.paint.brush.spacing) as usize;
        let borroes: Vec<usize> = (0..N_CAMADAS)
            .filter(|&p| {
                self.camada_viva(p) && matches!(self.paint.composite[p].op, CompositeOp::Blur)
            })
            .collect();
        let largo = borroes.iter().any(|&p| self.alguem_acima_le_vizinhanca(p));
        #[cfg(test)]
        let largo = largo || AVENTAL_LARGO.with(std::cell::Cell::get);
        let soma: usize = borroes
            .iter()
            .map(|&p| {
                let k = ph2d_painter_brush::kernel_radius(
                    self.paint.brush.radius_px * self.tamanho_da_camada(p),
                ) * passagens;
                if largo {
                    k
                } else {
                    ph2d_painter_brush::BlurKernel::Caixa.alcance(k)
                }
            })
            .sum();
        if soma == 0 {
            return 0;
        }
        soma as u32 + 1
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
pub(super) fn passagens_do_borrao(spacing: f32) -> u32 {
    const TECTO: u32 = 8;
    let n = 1.0 / spacing.max(1e-3);
    (n.round().max(1.0) as u32).min(TECTO)
}

/// **As fases de UM evento da pilha que acumula, em µs** — a sonda que atribui o custo que o report
/// de *«performance ruim»* (2026-09-21) nomeia sem o localizar.
///
/// ⚠️ Ela mede o que o código que SHIPA faz, e não um laço próprio: o precedente é o
/// `stamp_banded::diag`, cujo cabeçalho explica porque uma sonda com laço próprio fica cega à
/// porta. ⚠️ Como ela ZERA ao ler, há **um leitor só por thread**.
#[cfg(test)]
#[path = "composite_acumulado_fases.rs"]
pub(super) mod fases;
