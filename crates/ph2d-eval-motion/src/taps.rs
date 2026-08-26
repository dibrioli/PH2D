//! **A FAMÍLIA DOS TAPS** — cortada do `lib.rs` no teto de LOC (HR-18) pela costura que
//! ela já tinha: o `lib.rs` corre o RELÓGIO do pump, e isto é a caixa de saída de quem
//! espreita um nó a meio do grafo (o readout inline do doc 43).
//!
//! ⚠️ **Uma delas NÃO é um acessor**: o `record_tap_fires` corre DENTRO do cozimento e é o
//! que dá sentido às outras quatro — deixá-lo no `lib.rs` teria partido a família ao meio
//! e escondido que a lista é preenchida por tique, não por chamada.

use crate::{MotionCookPump, NodeId, Stream};

impl MotionCookPump {
    /// Carimba no livro-razão o que as tomadas disseram no tique que o chamador PEDIU.
    ///
    /// ⚠️ Chamado pelas duas marchas (a de avanço e a de scrub) e **uma vez cada**: o scrub
    /// re-cozinha o intervalo inteiro por dentro, e registrar cada passo faria um wrap de
    /// loop gritar a volta toda num quadro só.
    pub(crate) fn record_tap_fires(&mut self, tick: u64) {
        for (node, stream) in &self.tap_streams {
            self.tap_fires.push((tick, *node, stream.clone()));
        }
    }

    /// **Arma as TOMADAS deste quadro** — os nós cujo stream cru o host quer ler.
    ///
    /// ⚠️ **Elas são estado da BOMBA, e não argumento de uma chamada**, porque a marcha tem
    /// mais de uma porta: a rota da GPU HÍBRIDA marcha por
    /// [`Self::advance_or_scrub_to_nodes_scoped`], e enquanto a tomada era argumento da porta
    /// de sinks um documento híbrido cozinhava, desenhava e **não gritava nada** — medido no
    /// produto, com a suíte verde. Armada aqui, ela cavalga a marcha que houver, e a rota que
    /// nascer amanhã nasce coberta.
    ///
    /// Lista vazia é o mundo anterior byte a byte: nada é cozido e nada é guardado.
    pub fn set_taps(&mut self, taps: &[NodeId]) {
        self.taps.clear();
        self.taps.extend_from_slice(taps);
    }

    /// O que as tomadas disseram em cada tique MARCHADO desde a última limpeza — o livro-razão.
    ///
    /// ⚠️ **É ele que torna a leitura independente da ROTA:** o host limpa uma vez por quadro,
    /// as marchas carimbam (uma linha por tique PEDIDO, nunca por passo de re-simulação), e a
    /// leitura acontece UMA vez, depois de qualquer caminho de cook. Ler `tap_streams` dentro
    /// de um laço de marcha funcionava — para o laço que o autor lembrasse de instrumentar.
    #[must_use]
    pub fn tap_fires(&self) -> &[(u64, NodeId, Stream)] {
        &self.tap_fires
    }

    /// Zera o livro-razão — o host o chama uma vez por quadro, onde limpa o resto do que o
    /// quadro publica.
    pub fn clear_tap_fires(&mut self) {
        self.tap_fires.clear();
    }

    /// Os streams das TOMADAS da última marcha de sinks, rotulados por nó.
    ///
    /// ⚠️ **Vazio, e não obsoleto, quando o quadro não cozinhou** — o `pump` faz
    /// early-return num quadro pausado e inalterado, e nesse quadro esta lista guarda
    /// o que a última marcha deixou. Quem publica EVENTO a partir daqui tem de ler
    /// **dentro** do laço de tiques devidos, nunca depois dele: dois tiques devidos
    /// deixam só o último, e a perda é silenciosa.
    #[must_use]
    pub fn tap_streams(&self) -> &[(NodeId, Stream)] {
        &self.tap_streams
    }
}
