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
//! # A cura: os planos acumulam por evento, a TELA compõe-se por quadro
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
        self.compoe_a_regiao(caixa, &dabs);
        // As filas voltam vazias mas com a capacidade — um quadro não re-aloca.
        for (fila, mut usada) in self.paint.pilha.pendente_dabs.iter_mut().zip(dabs) {
            usada.clear();
            *fila = usada;
        }
    }

    /// Esquecer o pendente sem o compor — só depois de um DESCASCAR (ver o cabeçalho).
    pub(super) fn descarta_o_pendente(&mut self) {
        self.paint.pilha.pendente = None;
        for fila in &mut self.paint.pilha.pendente_dabs {
            fila.clear();
        }
    }
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
