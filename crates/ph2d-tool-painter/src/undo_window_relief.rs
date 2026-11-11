//! **O RELEVO na janela** — filho de [`super`]: o empréstimo dos três journals de relevo e as perguntas
//! sobre eles (B52.R4, movido à letra de `undo_window.rs`).

#[cfg(doc)]
use super::ReliefJournals;
use super::WriteState;

/// **Os três journals de relevo, emprestados sob o guard** — ver [`WriteState::with_relief_before`].
///
/// Os três juntos e não um a um porque a rota é **tudo-ou-nada**: eles descrevem um passo, e um passo
/// que só souber dizer a altura deixaria a cobertura e o material saírem por outro caminho — dois
/// modelos do mesmo fato, que é o que o `SculptSnap` do doc 18 §10.4 já custou uma vez.
pub(crate) struct ReliefBefore<'a> {
    pub(crate) heights: &'a crate::undo::journal::TileJournal<f32>,
    pub(crate) covers: &'a crate::undo::journal::TileJournal<u8>,
    pub(crate) mats:
        &'a crate::undo::journal::TileJournal<ph2d_painter_brush::material::MaterialBytes>,
    /// Quais planos foram escritos **sem ter forma de canvas** — ver [`ReliefJournals::absent`].
    pub(crate) absent: [bool; 3],
}

/// Qual dos três planos de relevo — o índice que [`ReliefJournals::absent`] usa.
///
/// Existe porque as três portas fazem a MESMA pergunta sobre planos de tipos diferentes, e um enum é o
/// que deixa a resposta ser dada uma vez em vez de três.
///
/// ⚠️ **NÃO é gateado por `cfg`**, e a tentativa de gateá-lo não compilou em `--release`: ele atravessa a
/// assinatura de [`WriteState::note_absent_relief`], que as portas chamam em **qualquer** perfil. Um tipo
/// que só existe em debug daria duas formas às portas — o mesmo motivo do `note_untracked_write`.
/// ⚠️ E `cargo clippy -p` roda em **debug**: só o check de release pega isto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReliefPlane {
    Heights = 0,
    Covers = 1,
    Mats = 2,
}

impl WriteState {
    /// **O plano `which` da camada `layer` foi escrito sem ter forma de canvas.** Ver
    /// [`ReliefJournals::note_absent`], que decide qual das duas coisas isso significa.
    ///
    /// ⚠️ Não é gateado por `cfg` pela mesma razão do [`Self::note_untracked_write`]: as três portas o
    /// chamam em qualquer perfil, e um canal que só existisse em debug daria duas formas às portas.
    pub(crate) fn note_absent_relief(&self, layer: crate::layers::LayerId, which: ReliefPlane) {
        self.relief.borrow_mut().note_absent(layer, which);
    }

    /// Quais planos de relevo este passo escreveu **sem que houvesse um antes** — o que a rede confere
    /// contra o `before` (`audit_relief_absence`). Índices de [`ReliefPlane`].
    #[cfg(any(test, debug_assertions))]
    #[must_use]
    pub(crate) fn relief_absent(&self) -> [bool; 3] {
        self.relief.borrow().absent
    }

    /// O que os journals de relevo dizem sobre o elemento `i` da camada `layer`, ou `None` quando eles
    /// não falam por ela (mistura, plano sem nome, tile não capturado).
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn heights_before(&self, layer: crate::layers::LayerId, i: usize) -> Option<f32> {
        let r = self.relief.borrow();
        r.speaks_for(layer).then(|| r.heights.get(i)).flatten()
    }

    /// Ver [`Self::heights_before`].
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn covers_before(&self, layer: crate::layers::LayerId, i: usize) -> Option<u8> {
        let r = self.relief.borrow();
        r.speaks_for(layer).then(|| r.covers.get(i)).flatten()
    }

    /// Ver [`Self::heights_before`].
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn mats_before(
        &self,
        layer: crate::layers::LayerId,
        i: usize,
    ) -> Option<ph2d_painter_brush::material::MaterialBytes> {
        let r = self.relief.borrow();
        r.speaks_for(layer).then(|| r.mats.get(i)).flatten()
    }

    /// As três irmãs de release das capturas acima — no-ops, como a do canvas. Sem elas as portas
    /// nomeadas teriam de carregar o `cfg` dentro do corpo, e uma porta com duas formas é o começo de
    /// duas respostas.
    /// Ver [`Self::capture_heights`].
    /// Ver [`Self::capture_heights`].
    /// **Por que os journals de relevo não descrevem o passo** — o readout que separa *"não havia o que
    /// descrever"* de *"havia e eles não sabem"*, e só o segundo é dívida de migração.
    #[must_use]
    pub(crate) fn relief_state(&self) -> &'static str {
        let r = self.relief.borrow();
        match () {
            () if r.mixed => "MISTURADO", // duas camadas num passo
            // ⚠️ **A causa deste estado MUDOU e a frase que morava aqui virou falsa.** Ela dizia
            // *"alguém escreveu pela porta genérica"*, e a porta genérica é `cfg(test)` desde que os
            // dez sítios de sculpt/warp migraram: em produção ela não existe. O único produtor que
            // sobrou é o `else` das três portas nomeadas — o plano não tinha forma de canvas na hora
            // da escrita (a primeira pincelada de uma camada, ou um plano de outro documento).
            //
            // ⚠️ Medido: **202 passos da suíte**, todos por essa via. Não é a mesma coisa que a antiga,
            // e a diferença decide o S3: um plano que **não existia** no começo do passo não tem
            // *antes* a descrever — o motor de delta já chama isso de `OnlyAfter`. Separar as duas é o
            // próximo degrau; enquanto elas dividem um estado, este número parece dívida e não é.
            () if r.incomplete => "INCOMPLETO",
            () if r.layer.is_none() => "SEM-RELEVO", // o passo não tocou relevo nenhum
            () => "DESCREVE",
        }
    }

    /// **Empresta os três journals de relevo, SE eles descrevem o passo** — a porta única por onde o
    /// motor de delta os alcança (doc 28 §5.58.2, degrau 2).
    ///
    /// ⚠️ **A pergunta e o empréstimo são a MESMA chamada, de propósito.** Perguntar num lugar e pegar
    /// os journals noutro abriria espaço para um chamador futuro pegar sem perguntar — e um journal que
    /// não descreve o passo devolve bytes de um passado que não é o dele, que é o modo de falha exato
    /// que a proveniência ([`Self::journal_describes_step_at`]) existe para impedir.
    ///
    /// `None` = não descrevem; o chamador cai no caminho de sempre (*lento nunca, errado jamais*).
    pub(crate) fn with_relief_before<R>(
        &self,
        writes: u64,
        layer: crate::layers::LayerId,
        f: impl FnOnce(ReliefBefore<'_>) -> R,
    ) -> Option<R> {
        if !self.relief_describes_step_at(writes, layer) {
            return None;
        }
        let r = self.relief.borrow();
        Some(f(ReliefBefore {
            heights: &r.heights,
            covers: &r.covers,
            mats: &r.mats,
            absent: r.absent,
        }))
    }

    /// **Ninguém escreveu relevo desde o último commit.**
    ///
    /// É o que autoriza usar o cursor REIDRATADO como base do relevo: com o journal em silêncio, o
    /// plano vivo (o mesmo objeto que a testemunha do cursor aponta) ainda carrega os bytes da era
    /// dele. Uma única captura o derruba — aí o objeto pode ser o mesmo e o conteúdo não.
    #[must_use]
    pub(crate) fn relief_untouched(&self) -> bool {
        let r = self.relief.borrow();
        r.layer.is_none() && !r.mixed && !r.incomplete
    }

    /// **Os journals de relevo descrevem o passo que começou em `writes`, para a camada `layer`?**
    #[must_use]
    pub(crate) fn relief_describes_step_at(
        &self,
        writes: u64,
        layer: crate::layers::LayerId,
    ) -> bool {
        self.journal_describes_step_at(writes) && self.relief.borrow().speaks_for(layer)
    }
}
