//! ⭐⭐⭐ **A PILHA COMPÕE UMA VEZ POR QUADRO — e acumula a cada evento do rato.**
//!
//! # O defeito, medido (report do dono, 2026-09-23: *«FPS cai para 1»*)
//!
//! Na cena `PH2D_COMPOSITE_SMOKE` o app lê `60 fps` parado (`PH2D_PAINT_PERF=1`); a queda é só a
//! pintar. A sonda [`super::diag_passo_do_rato`] corre a pilha do dono no mesmo traço de `720 px`
//! com o rato a andar `0,5` a `256 px` por evento:
//!
//! | passo | eventos | ms do traço | ms/evento |
//! |---|---|---|---|
//! | `0,5` a `16` | `1 440` a `45` | **`245`–`263`** | `0,17` a `5,85` |
//! | `64` | `12` | `130` | `10,8` |
//! | `256` | `3` | **`74`** | `24,7` |
//!
//! ⇒ **o custo não é por EVENTO, é por PÍXEL percorrido (`~0,35 ms/px`)** — a minha 1.ª hipótese
//! (*«o rato de 1 kHz manda mil composições por segundo»*) caiu na primeira coluna: um evento sem dab
//! novo não compõe nada. O que cresce com o traço é que **cada evento com dabs recompõe o DISCO
//! inteiro da camada maior** (o Blur a `2,048` do pincel: `~340 px` de lado), e um passo de `16 px`
//! refaz `16×` a mesma área que um passo de `256`. Um risco rápido de `3 000 px/s` pede ao segundo
//! `~1 s` de trabalho — o app deixa de acompanhar o rato e o quadro estica.
//!
//! # A cura: os planos acumulam e a TELA compõe-se UMA vez por quadro
//!
//! ⭐ **E desde 2026-10-01 o ACÚMULO também espera** (o parágrafo abaixo descreve a 1.ª redacção,
//! em que os planos acumulavam a cada evento). Os dabs do quadro ficam na fila da camada e a
//! drenagem acumula-os de uma vez antes de compor: o resultado é o mesmo ao byte (cada plano é a
//! soma dos dabs DELE em ordem, e nada lê um plano entre a fila e a drenagem), e um lote de quadro
//! alcança a rota em BANDA do depósito, onde o pingo de um evento nunca chegava (`4,4 → 2,0` ms por
//! quadro no rabisco do dono; ADR-0172, emenda). ⛔ Um lote que NÃO pode esperar esvazia a fila
//! primeiro, senão passava à frente dela.
//!
//! A composição lê só o `pre` e os planos ([`super::composite_acumulado`]), e os planos já têm todo
//! o lote no fim de cada evento — logo compor a UNIÃO das caixas de um quadro dá a imagem que as
//! composições dos eventos dariam, com a área da união em vez da soma das áreas. **Nenhum dab sai
//! do sítio**: o caminho do rato é o mesmo, dab a dab; só a escrita na tela espera pela drenagem.
//!
//! # ⛔ Quem lê a tela ANTES de ela ser composta — e por isso NÃO adia
//!
//! A composição adiada só é honesta se ninguém ler a tela entre o evento e a drenagem. Há quatro
//! leitores medidos no caminho do carimbo, e com qualquer um vivo a composição corre no evento:
//!
//! * os métodos de **re-carimbo** (Line / Drag Dot / Anchored / figuras): a shell já os entrega UMA
//!   vez por quadro, e o peel do quadro seguinte restaura a região que a composição escreveu;
//! * o **Style: Solid**, cuja mancha é escrita por cima do carimbo no fim do evento;
//! * os **fios** (Sketchy / Wire), carimbados na tela depois da pilha;
//! * os dois **portões** (máscara de protecção e selecção), que fazem `lerp` sobre a tela carimbada.
//!
//! E o hospedeiro tem de DRENAR por quadro: [`PainterTool::set_compor_por_quadro`] é
//! semeado pela ponte do app. ⚠️ Um hospedeiro que nunca drena (os gates desta crate) compõe no evento,
//! como sempre — *uma tela que espera por uma drenagem que não vem é uma tela que não pinta*.
//!
//! # As portas que compõem o pendente
//!
//! As duas drenagens ([`PainterTool::take_preview_arc`] e `take_preview_dirty`), e os três sítios que
//! fecham a pilha (o pen-down seguinte, o `close_stroke`, o `commit_reset_pilha`). O
//! `restamp_reset_pilha` **descarta**: ali a tela acabou de ser descascada para o `pre` e os planos
//! esvaziados, logo compor o pendente reescreveria uma figura que já não existe.

use super::Region;
use super::composite::N_CAMADAS;
use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;

impl PainterTool {
    /// **O hospedeiro drena a pré-visualização uma vez por quadro** — liga a composição por quadro.
    ///
    /// ⚠️ Semeado pela ponte do app (`ph2d-app-painter`); um hospedeiro que não drena por quadro
    /// deixa-o desligado, e a pilha compõe no evento como antes.
    pub fn set_compor_por_quadro(&mut self, on: bool) {
        if !on {
            self.compoe_o_pendente();
        }
        self.paint.pilha.por_quadro = on;
    }

    /// Pode ESTE lote esperar pela drenagem? Só se ninguém ler a tela antes dela — ver o cabeçalho.
    pub(super) fn pilha_pode_adiar(&self) -> bool {
        self.paint.pilha.por_quadro
            && !self.coalesces_canvas_motion()
            && !self.freehand_solid_fill_live()
            && !self.threads_own_the_gesture()
            && !self.mask_protection_active()
            && !self.selection_restricts_paint()
    }

    /// Guardar o lote para a composição do quadro: a caixa entra na união, e os dabs (o esfregão
    /// lê-os) na fila da camada deles, em ordem.
    pub(super) fn adia_a_composicao(&mut self, caixa: Region, camadas: [Vec<Dab>; N_CAMADAS]) {
        let pilha = &mut self.paint.pilha;
        pilha.pendente = Some(
            pilha
                .pendente
                .map_or(caixa, |acc| super::union_region(acc, caixa)),
        );
        for (fila, lote) in pilha.pendente_dabs.iter_mut().zip(camadas) {
            fila.extend(lote);
        }
        // A drenagem só corre com a pré-visualização suja; quem a suja de facto é a composição.
        self.preview_dirty = true;
    }

    /// **Compor o que os eventos deste quadro deixaram por compor.** Sem pendente é um no-op, logo
    /// as portas que o chamam não pagam nada fora de um traço da pilha.
    pub(crate) fn compoe_o_pendente(&mut self) {
        let Some(caixa) = self.paint.pilha.pendente.take() else {
            return;
        };
        let dabs: [Vec<Dab>; N_CAMADAS] =
            std::array::from_fn(|pos| std::mem::take(&mut self.paint.pilha.pendente_dabs[pos]));
        #[cfg(test)]
        conta_composicao();
        let mut adiantado = None;
        let mut mudou = None;
        if self.o_acumulo_esperou() {
            // ⚠️ Quem lê da caixa pergunta ao campo do quadro ANTERIOR, e o adiantamento empresta-o
            // ao trabalho — logo a pergunta faz-se ANTES. Esquecê-lo faz a região encolher e voltar
            // os rectângulos (`o_esfregao_nao_deixa_rectangulos_de_cor` reprovou com `255`).
            mudou = Some(self.o_que_mudou(caixa));
            match self.adianta_o_campo(&dabs) {
                // ⭐ O campo do esfregão corre noutra thread ENQUANTO as outras camadas acumulam:
                // ele não lê nenhum plano nem a tela (só os seus pingos, a Selecção e o campo), e
                // elas não leem o campo — ver [`Self::adianta_o_campo`].
                Some((pos, trabalho)) => {
                    let lista = &dabs[pos];
                    let feito = std::thread::scope(|s| {
                        let h = s.spawn(move || trabalho.corre(lista));
                        self.acumula_as_camadas(&dabs);
                        h.join().expect("o campo do esfregão entrou em pânico")
                    });
                    self.paint.pilha.campo_adiantado = Some(feito);
                    adiantado = Some(pos);
                    #[cfg(test)]
                    ADIANTADOS.with(|c| c.set(c.get() + 1));
                }
                None => self.acumula_as_camadas(&dabs),
            }
        }
        self.compoe_a_regiao(caixa, &dabs, mudou);
        // A composição consome o campo no braço do esfregão; um que sobrasse levaria o campo da
        // sessão com ele, logo aplica-se aqui (não acontece: o braço corre para toda camada viva).
        if let (Some(feito), Some(pos)) = (self.paint.pilha.campo_adiantado.take(), adiantado) {
            debug_assert!(false, "o campo adiantado não foi consumido pela composição");
            let lista = std::mem::take(&mut self.paint.pilha.pendente_dabs[pos]);
            let _ = self.conclui_campo_do_esfregao(&lista, feito);
            self.paint.pilha.pendente_dabs[pos] = lista;
        }
        // As filas voltam vazias mas com a capacidade — um quadro não re-aloca.
        for (fila, mut usada) in self.paint.pilha.pendente_dabs.iter_mut().zip(dabs) {
            usada.clear();
            *fila = usada;
        }
    }

    /// **Pode o campo do esfregão deste quadro correr AO LADO do acúmulo?** Devolve a posição e o
    /// trabalho já tirado da ferramenta, com o pincel no estado em que o braço do esfregão o poria
    /// (força e dureza da camada — o [`Self::aplica_camada`]) e o fluxo aleatório DELA.
    ///
    /// Só quando:
    /// * há UM esfregão vivo com pingos neste quadro (dois disputariam o mesmo campo adiantado).
    ///
    /// ⚠️ No 1.º quadro do traço a preparação ABRE a sessão, que fotografa a tela e o relevo ANTES
    /// do acúmulo em vez de a meio da composição — e é o mesmo retrato: o acúmulo só escreve nos
    /// planos das camadas (a tela entra e sai por troca de `Arc`), nunca no relevo assente (que só
    /// o corpo escreve, DEPOIS da pilha), e a região composta é refrescada na base antes de o
    /// esfregão a ler. Medido: com uma guarda «só com a sessão já aberta» e sem ela, a suíte inteira
    /// da crate fica verde (`1 414`). ⛔ Numa 1.ª leitura eu atribuí-lhe dois vermelhos
    /// (`o_esfregao_nao_deixa_rectangulos_de_cor` · `o_esfregao_arrasta_o_corpo_com_a_cor`) que
    /// eram do campo EMPRESTADO lido pela pergunta de quem lê da caixa — ver o chamador.
    ///
    /// ⚠️ É exacto porque as duas metades não se leem: o acúmulo escreve nos planos e na tela
    /// emprestada, e o laço dos pingos lê só o que o trabalho levou (pingos, Selecção, imagens,
    /// campo). Gate `o_campo_ao_lado_do_acumulo_da_a_imagem_da_serie`.
    fn adianta_o_campo(
        &mut self,
        dabs: &[Vec<Dab>; N_CAMADAS],
    ) -> Option<(usize, super::smear_warp::TrabalhoDoCampo)> {
        #[cfg(test)]
        if CAMPO_EM_SERIE.with(std::cell::Cell::get) {
            return None;
        }
        let mut vivos = (0..N_CAMADAS).filter(|&p| {
            self.camada_viva(p)
                && self.paint.composite[p].op == super::composite::CompositeOp::Smear
        });
        let pos = vivos.next()?;
        if vivos.next().is_some() || dabs[pos].is_empty() {
            return None;
        }
        let layer = self.paint.composite[pos];
        let (saved_strength, saved_hardness) =
            (self.paint.brush.strength, self.paint.brush.hardness);
        let saved_rng = self.paint.tex_rng;
        self.paint.brush.strength = layer.strength;
        self.paint.brush.hardness = layer.hardness.unwrap_or(saved_hardness);
        self.paint.tex_rng = self.paint.rng_camada[pos];
        let (w, h) = self.source_size;
        let trabalho = self.prepara_campo_do_esfregao(w, h);
        self.paint.brush.strength = saved_strength;
        self.paint.brush.hardness = saved_hardness;
        self.paint.tex_rng = saved_rng;
        trabalho.map(|t| (pos, t))
    }

    /// Esquecer o pendente sem o compor — só depois de um DESCASCAR (ver o cabeçalho).
    pub(super) fn descarta_o_pendente(&mut self) {
        self.paint.pilha.pendente = None;
        for fila in &mut self.paint.pilha.pendente_dabs {
            fila.clear();
        }
    }
}

// `true` = o campo do esfregão corre em SÉRIE, depois do acúmulo (o código de antes de
// 2026-10-01) — o CONTROLO do gate que prova que correr ao lado dá a mesma imagem.
#[cfg(test)]
thread_local! {
    pub(super) static CAMPO_EM_SERIE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Quantas vezes o campo correu AO LADO do acúmulo — o controlo de que o gate mede a rota.
    pub(super) static ADIANTADOS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

// Quantas composições de pendente correram — a régua do gate que prova UMA por quadro.
#[cfg(test)]
thread_local! {
    static COMPOSICOES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn conta_composicao() {
    COMPOSICOES.with(|c| c.set(c.get() + 1));
}

/// Lê e zera o contador de composições de pendente.
#[cfg(test)]
pub(super) fn composicoes_e_zera() -> u64 {
    COMPOSICOES.with(|c| c.replace(0))
}
