//! **A FAMÍLIA DOS TAPS** — cortada do `lib.rs` no teto de LOC (HR-18) pela costura que
//! ela já tinha: o `lib.rs` corre o RELÓGIO do pump, e isto é a caixa de saída de quem
//! espreita um nó a meio do grafo (o readout inline do doc 43).
//!
//! ⚠️ **Uma delas NÃO é um acessor**: o `record_tap_fires` corre DENTRO do cozimento e é o
//! que dá sentido às outras quatro — deixá-lo no `lib.rs` teria partido a família ao meio
//! e escondido que a lista é preenchida por tique, não por chamada.

use crate::sink_style::o_que_o_sink_desenha;
use crate::{CookTarget, MotionCookPump, NodeId, Stream};
use ph2d_nodegraph::cook::{OpResolver, TimeScopes};
use ph2d_nodegraph::graph::Graph;

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

    /// ⭐⭐⭐ **COZINHA SÓ AS TOMADAS, sem marchar o tique** — a porta que faltava à rota
    /// **totalmente na GPU**.
    ///
    /// ⛔⛔ **O buraco, medido em 2026-09-08 por um report do dono** (*«sumiu com o gizmo do
    /// Bezier Warp»* → *«ainda invisível»*): as tomadas são cozidas dentro do
    /// [`MotionCookPump::cook_target_into`], que corre na MARCHA. E na rota `FullyGpu` a ponte
    /// **retorna antes da marcha** — o device produziu o quadro, a bomba não corre, e
    /// `tap_streams` fica vazio. Quem depende de uma tomada deixa de existir **em silêncio**.
    ///
    /// ⚠️ **A rota HÍBRIDA já estava coberta**, e o doc do [`Self::set_taps`] conta essa cura: as
    /// tomadas passaram a ser estado da bomba precisamente para cavalgarem a marcha que houver.
    /// *O que ninguém viu é que a rota `FullyGpu` não marcha marcha nenhuma* — e ali não há
    /// «a marcha que houver».
    ///
    /// ⇒ **vítimas: DUAS.** O gizmo de canvas dos deformadores de quadrilátero (que lê a caixa
    /// envolvente da tomada de montante) e os **sinais** de um documento inteiramente no device.
    ///
    /// ⚠️ **Não marcha o tique, e é isso que a torna segura de chamar depois do device:** ela
    /// COZINHA (uma pergunta pura sobre o grafo neste playhead, que bate no memo), nunca avança
    /// estado. Marchar aqui simularia o tique duas vezes — uma no device, outra na CPU — e um nó
    /// sequencial andaria a dobrar.
    ///
    /// ⚠️ **O preço é real e nomeado:** para a tomada do gizmo, isto cozinha na CPU a cadeia a
    /// montante do nó seleccionado. Ele só se paga **enquanto há tomada armada** — e a lista está
    /// vazia sem nó de warp seleccionado e sem `pulse.signal` no grafo, que é o caso comum.
    pub fn cook_taps_only(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        playhead: f64,
        scopes: &TimeScopes,
    ) {
        if self.taps.is_empty() {
            return; // o mundo anterior, byte a byte
        }
        self.tap_streams.clear();
        for i in 0..self.taps.len() {
            let node = self.taps[i];
            if self.tap_streams.iter().any(|(n, _)| *n == node) {
                continue;
            }
            // A MESMA política do laço da marcha: uma tomada que falha simplesmente não aparece.
            if let Ok(outputs) = self
                .cook
                .cook_scoped_fanned(graph, ops, node, playhead, scopes, &self.fans)
                && let Some(v) = outputs.first()
            {
                self.tap_streams.push((node, v.as_stream().clone()));
            }
        }
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

    /// **Cozinha as TOMADAS deste quadro** — a seguir ao alvo e no MESMO playhead, para os dois
    /// alvos.
    ///
    /// ⚠️ **Mudou de ficheiro por TECTO DE LOC, e a morada é por ASSUNTO:** este módulo já é o dono
    /// das tomadas (o `set_taps`, o `record_tap_fires`, o `cook_taps_only`), e o laço que as enche
    /// vivia no meio do `cook_target_into`. ⛔ O corte foi a cura do tecto — nunca uma entrada nova
    /// no `FILE_OVERAGE_OK`.
    pub(crate) fn cozinha_as_tomadas(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        target: &CookTarget,
        playhead: f64,
        scopes: &TimeScopes,
    ) {
        // ⚠️ Elas ficavam dentro do braço `Sinks`, e o preço foi medido no produto: a rota
        // HÍBRIDA marcha por `Boundaries`, então um documento com `pulse.signal` cozinhava,
        // desenhava e **não gritava nada** — com a suíte verde, porque todo gate dirigia a
        // porta de sinks. A tomada é do pump; ela cavalga a marcha que houver.
        //
        // Cozinhar aqui bate no MEMO de tudo o que a tomada compartilha com o alvo (o mesmo
        // argumento que o braço `Boundaries` acima explica): o `Fingerprint` carrega o tique,
        // e o tique só anda no `advance_tick_scoped`, que a marcha chama uma vez.
        // ⛔⛔⛔ **E uma tomada NUM SINK tem de ver o que o sink DESENHA, não o que ele cozinhou**
        // (doc 115 §16, report do dono com foto): o passe do fim reescreve o `P`, e uma tomada que
        // guarde a corrente CRUA entrega ao gizmo do colisor as posições de ANTES da separação.
        // *As formas saíam certas e o contorno azul ficava onde elas estavam* — duas respostas à
        // mesma pergunta, e o artista vê as duas ao mesmo tempo.
        //
        // ⚠️ **Só para quem é SINK neste quadro**, e o discriminador não pode ser o param: a
        // `source.shape` declara um `collide` com o MESMO nome do interruptor do sink (o botão
        // dela), logo perguntar o param à cega separaria a corrente da própria forma.
        let sinks_do_quadro: &[NodeId] = match *target {
            CookTarget::Sinks { sinks, .. } => sinks,
            CookTarget::Boundaries(_) => &[],
        };
        // ⚠️ **Sem `clear` aqui:** quem limpa é o braço `Sinks` acima (que publica o que desenhou)
        // e o braço `Boundaries`, que não tem rota de desenho. Limpar aqui apagaria a publicação.
        for i in 0..self.taps.len() {
            let node = self.taps[i];
            if self.tap_streams.iter().any(|(n, _)| *n == node) {
                continue;
            }
            // Uma tomada que falha ao cozinhar simplesmente NÃO APARECE — o chamador lê uma
            // lista mais curta, nunca um stream errado (a política do `boundary_streams`).
            if let Ok(outputs) = self
                .cook
                .cook_scoped_fanned(graph, ops, node, playhead, scopes, &self.fans)
                && let Some(v) = outputs.first()
            {
                let cozido = v.as_stream();
                let desenhado = sinks_do_quadro
                    .contains(&node)
                    .then(|| o_que_o_sink_desenha(graph, node, cozido))
                    .flatten();
                self.tap_streams
                    .push((node, desenhado.unwrap_or_else(|| cozido.clone())));
            }
        }
    }
}

impl MotionCookPump {
    /// **Este cozimento vai ser desenhado?** — ver [`MotionCookPump::separa_o_desenho`] (o campo).
    ///
    /// A shell põe-no a `false` nos tiques que ela recupera e a `true` no último, que é o que o
    /// artista vê. ⚠️ O valor de omissão é `true`.
    pub fn set_separa_o_desenho(&mut self, sim: bool) {
        self.separa_o_desenho = sim;
    }

    /// Quantas separações correram desde que a bomba existe — o readout que torna a economia
    /// acima observável (as duas rotas entregam o MESMO desenho).
    #[must_use]
    pub fn separacoes(&self) -> u64 {
        self.separacoes
    }
}
