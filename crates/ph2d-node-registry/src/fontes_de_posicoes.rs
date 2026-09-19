//! ⭐⭐⭐ **QUEM SÓ ENTREGA POSIÇÕES** — a bandeira que o cartão do nó lê para escrever o aviso
//! *«sem um Duplicator e um objecto a copiar, isto é invisível»* (ordem do dono, 2026-09-19).
//!
//! Módulo irmão do `lib.rs` por TETO DE LOC (HR-18, 700 para `crates/`), e o corte é por
//! RESPONSABILIDADE: os canais de side-metadata do `lib.rs` são REGISTOS (alguém declara, alguém
//! lê), e isto é uma **CLASSIFICAÇÃO** — uma pergunta sobre a forma dos manifestos, respondida
//! numa passagem só. O campo fica na struct; a lei mora aqui.

use crate::NodeRegistry;
use ph2d_nodegraph::node::NodeTypeId;

impl NodeRegistry {
    /// ⭐⭐⭐ **ESTE NÓ SÓ ENTREGA POSIÇÕES — sem um `motion.duplicator` e um objecto a copiar,
    /// ele é INVISÍVEL.**
    ///
    /// > **Ordem do dono, 2026-09-19:** *«coloque um alerta de que se não forem usados com
    /// > duplicator e um objeto a ser copiado, são invisíveis»*.
    ///
    /// É um facto sobre o **TIPO** do nó, e é por isso que ele vive aqui e não num diagnóstico.
    ///
    /// ⛔⛔ **A rota do DIAGNÓSTICO foi construída e MEDIDA e recusada** (`Deficit::NeverDrawn`,
    /// 2026-09-19): um déficit calculado acende em **todo** grafo de posições que existe — as 111
    /// cenas do produto incluídas, porque nenhuma delas está migrada — e `14` fixturas do
    /// diagnosticador ficaram vermelhas na primeira corrida. *Uma queixa que soa sempre é ruído a
    /// competir com os avisos a sério, exactamente quando um deles passar a ser verdade.*
    ///
    /// ⚠️ **DERIVADO do manifesto, nunca uma lista escrita à mão:** *emite instâncias e não recebe
    /// instâncias*. Quem o regista é o `register_all_nodes`, numa passagem só, logo uma fonte de
    /// posições que nasça amanhã entra sozinha — e uma lista à mão ficaria muda no primeiro nó
    /// novo, que é como o censo por prefixo desta casa já falhou (CLAUDE.md §5.0).
    ///
    /// Aditivo; idempotente.
    pub fn register_so_posicoes(&mut self, id: NodeTypeId) {
        self.so_posicoes.insert(id);
    }

    /// `id` só entrega posições? Ausente ⇒ não — o valor de omissão, que é o de todo nó que
    /// recebe uma corrente de instâncias ou que não emite nenhuma.
    #[must_use]
    pub fn so_posicoes(&self, id: NodeTypeId) -> bool {
        self.so_posicoes.contains(&id)
    }

    /// ⭐⭐⭐ **CLASSIFICA todo nó já registado — a passagem que preenche [`Self::so_posicoes`].**
    ///
    /// A regra, derivada do MANIFESTO e das bandeiras que os nós já declaram — **três cláusulas, e
    /// as duas últimas foram MEDIDAS contra a lei antes de escritas** (ver a sonda
    /// `a_bandeira_contra_a_lei`, que coze cada candidato e pergunta a
    /// `ph2d_eval_motion::tem_aparencia` à corrente que ele entrega):
    ///
    /// 1. **emite POSIÇÕES** — `Instances` **e `Vec2`**, que é a coluna `P`. ⛔ Sem o `Vec2` a
    ///    regra apanha o `value.number` e o `debug.const`, que emitem `Instances`/`Scalar`: são
    ///    NÚMEROS por elemento, não pontos no espaço, e um aviso sobre duplicadores num nó de
    ///    valor não quer dizer nada. *É a mesma pergunta que o `has_preview_slot` do painel já
    ///    fazia para decidir se um cartão tem retrato.*
    /// 2. **não recebe instâncias** — senão é um nó de PASSAGEM, e o aviso acenderia em todo
    ///    `motion.move` do grafo.
    /// 3. ⛔⛔ **não é uma ORIGEM DE APARÊNCIA** ([`Self::is_object_source`] ·
    ///    [`Self::is_live_vector_source`]) — o `source.object`, o `source.shape` e o `source.text`
    ///    passam nas duas primeiras e **são exactamente o que o aviso manda ir buscar**. Medido:
    ///    no valor de FÁBRICA eles não trazem aparência nenhuma (nada escolhido ⇒ `tem_aparencia`
    ///    responde `false`), logo o aviso seria *verdadeiro por um instante e MENTIRA a partir do
    ///    clique seguinte* — e a mentira estaria no cartão do nó que resolve o problema.
    ///
    /// ⚠️⚠️ **A cláusula 3 lê bandeiras que os `register()` de cada nó põem, logo a ORDEM é
    /// load-bearing:** esta passagem tem de correr DEPOIS de todos eles. É por isso que ela é
    /// chamada no fim do `register_all_nodes` e não de dentro de um `register` — e há gate.
    ///
    /// ⚠️ **Corre UMA vez, depois de todo nó estar registado** (o `register_all_nodes` chama-a a
    /// seguir à região gerada) — uma fonte de posições que nasça amanhã entra sozinha, sem ninguém
    /// se lembrar dela. Idempotente.
    ///
    /// ⚠️ **Os ids colhem-se ANTES de escrever** porque [`Self::manifests`] empresta `self`; o
    /// `BTreeSet` de destino torna a ordem da colheita irrelevante.
    pub fn marca_as_fontes_de_posicoes(&mut self) {
        use ph2d_nodegraph::node::PortSpec;
        use ph2d_nodegraph::port::{Dim, Domain};
        let recebe = |p: &PortSpec| p.ty.domain == Domain::Instances;
        let emite_posicoes =
            |p: &PortSpec| p.ty.domain == Domain::Instances && p.ty.dim == Dim::Vec2;
        let fontes: Vec<NodeTypeId> = self
            .manifests()
            .filter(|m| m.outputs.iter().any(emite_posicoes) && !m.inputs.iter().any(recebe))
            .map(|m| m.id)
            .filter(|id| !self.is_object_source(*id) && !self.is_live_vector_source(*id))
            .collect();
        for id in fontes {
            self.so_posicoes.insert(id);
        }
    }
}
