//! **O GRAFO da booleana viva** — a operação deixa de ser do grupo e passa a ser da LIGAÇÃO.
//!
//! Irmão do [`crate::VecBoolGroup`], e ele responde a pergunta que aquele não sabe responder:
//! *"esta forma SOMA com aquela e SUBTRAI desta outra"*. O grupo tem **uma** operação para todos os
//! filhos; aqui cada ligação tem a sua, e tem **direção**.
//!
//! # Presença = o grupo é um GRAFO; ausência = o grupo é o de sempre
//!
//! ⚠️ Um documento que nunca abriu o diagrama é **byte-idêntico** ao de antes desta feature — a
//! mesma lei do [`crate::VecClipContent`]. E é isso que permite a migração ser um NO-OP visível:
//! `ph2d_vec_boolean::derive_star` escreve o grafo equivalente ao grupo de hoje, e o gate
//! `a_estrela_derivada_desenha_o_que_o_grupo_de_hoje_desenha` prova que ele desenha o mesmo.
//! Materializar as ligações ao abrir a janela não pode mover a arte um pixel.
//!
//! # Os ids são `VecPathId` crus, e é o que sobrevive ao undo
//!
//! ⚠️ **Nunca `Entity::to_bits()`.** O undo é snapshot-based: ele respawna tudo com bits NOVOS, e
//! bits guardados dentro dos bytes de um componente envenenariam o próprio undo — a mesma armadilha
//! que o `stable_name_id` existe para evitar. O `VecPathId` vive no `VecScene`, que é capturado e
//! restaurado inteiro, então ele atravessa undo e save intacto. É o mesmo `u64` que o
//! [`crate::VecPathRef`] carrega, e pela mesma razão: `ph2d-ecs` não depende do documento vetorial.
//!
//! # A lista é CANÓNICA, não histórica
//!
//! ⚠️ [`VecBoolEdges::new`] ordena e desduplica, e isso é load-bearing: o undo regista um passo por
//! **DIFF de bytes**, então duas listas com o mesmo significado e ordens diferentes seriam dois
//! estados — e reordená-la (uma edição do painel, um merge) viraria um passo de undo que não mudou
//! nada na tela. Quem resolve o grafo ordena por z na hora de dobrar
//! (`ph2d_vec_boolean::resolve_graph`), então a ordem guardada **não pode** ter significado, e há
//! gate a prová-lo dos dois lados.
//!
//! # Uma ligação por par ORDENADO
//!
//! Duas ligações `A → B` com operações diferentes não são desenháveis no diagrama (é uma linha só
//! entre dois círculos) e o resolvedor dobraria `A` em `B` duas vezes. [`VecBoolEdges::set`] é a
//! porta que garante a unicidade; `A → B` e `B → A` continuam a ser ligações **distintas** — a
//! direção é o dado.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **Uma ligação dirigida**: `from` OPERA sobre `to`, que RECEBE.
///
/// A ordem dos campos é a de comparação (`from`, `to`, `op`), e é ela que dá a forma canónica da
/// lista — ver o doc do módulo.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct VecBoolEdge {
    /// O `VecPathId` de quem OPERA (o operando consumido).
    pub from: u64,
    /// O `VecPathId` de quem RECEBE.
    pub to: u64,
    /// O discriminante de `ph2d_vec_boolean::PathfinderOp`, o mesmo `u8` cru que o
    /// [`crate::VecBoolGroup`] carrega — e pela mesma razão (`ph2d-ecs` não vê o vetor).
    ///
    /// ⚠️ Só as **quatro operações de conjunto** são válidas numa ligação; as quatro receitas são
    /// afirmações sobre a pilha inteira. Um código inválido faz o grafo RECUSAR, e o grupo desenha
    /// como grupo comum — degradar para *os filhos aparecem* é a única leitura que não perde arte.
    pub op: u8,
}

/// **As ligações do grafo de um grupo booleano.** A entidade que o carrega é o mesmo grupo que
/// carrega o [`crate::VecBoolGroup`].
///
/// ⚠️ **Ele não age sozinho:** quem varre a cena procura o [`crate::VecBoolGroup`], então uma
/// entidade com ligações e sem ele é inerte. É de propósito — *"isto é uma booleana?"* continua a
/// ser UMA pergunta, com UMA resposta, e tirar o componente da booleana desliga a feature inteira
/// em vez de a deixar meio ligada.
///
/// ⚠️ **A PRESENÇA é que decide, e a lista vazia NÃO é o mesmo que a ausência.**
/// - Ausente ⇒ o grupo é o de sempre: os filhos combinam pela operação única do
///   [`crate::VecBoolGroup`].
/// - Presente ⇒ o grupo é um GRAFO, e a lista é a verdade inteira. **Vazia ⇒ nenhuma relação**, e
///   cada forma desenha-se a si própria.
///
/// A distinção é load-bearing: no diagrama, cortar a última ligação tem de deixar as formas
/// separadas. Se a lista vazia caísse de volta na operação única, cortar o último elo faria as
/// formas **fundirem-se** — o oposto exato do gesto. Voltar à operação única é remover o
/// componente, que é um gesto diferente e deliberado.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecBoolEdges {
    /// As ligações, em forma canónica (ordenadas, uma por par ordenado). ⚠️ Escreva por
    /// [`VecBoolEdges::new`] ou [`VecBoolEdges::set`], nunca por `push` — ver o doc do módulo.
    pub edges: Vec<VecBoolEdge>,
}

impl VecBoolEdges {
    /// A lista em forma **canónica**: ordenada, e uma ligação por par ordenado (a de menor código
    /// de operação sobrevive a um par repetido — a escolha tem de ser DETERMINISTA, senão o mesmo
    /// documento desenharia coisas diferentes).
    #[must_use]
    pub fn new(mut edges: Vec<VecBoolEdge>) -> Self {
        edges.sort_unstable();
        edges.dedup_by(|a, b| a.from == b.from && a.to == b.to);
        Self { edges }
    }

    /// A operação desta ligação, se ela existe.
    #[must_use]
    pub fn get(&self, from: u64, to: u64) -> Option<u8> {
        self.edges
            .iter()
            .find(|e| e.from == from && e.to == to)
            .map(|e| e.op)
    }

    /// Liga `from → to`, SUBSTITUINDO a operação se a ligação já existia. Mantém a forma canónica.
    pub fn set(&mut self, from: u64, to: u64, op: u8) {
        match self.edges.iter_mut().find(|e| e.from == from && e.to == to) {
            Some(slot) => slot.op = op,
            None => {
                let e = VecBoolEdge { from, to, op };
                let at = self.edges.partition_point(|x| *x < e);
                self.edges.insert(at, e);
            }
        }
    }

    /// Corta a ligação `from → to`. Devolve se havia alguma.
    pub fn remove(&mut self, from: u64, to: u64) -> bool {
        let n = self.edges.len();
        self.edges.retain(|e| !(e.from == from && e.to == to));
        self.edges.len() != n
    }

    /// Esquece toda ligação que toque `id` — o que uma forma apagada exige.
    ///
    /// ⚠️ Sem esta limpeza a ligação órfã sobrevive e o grafo inteiro passa a RECUSAR (o resolvedor
    /// não inventa uma resposta com um operando a menos), o que apaga a booleana de um grupo que o
    /// artista nem tocou.
    pub fn forget(&mut self, id: u64) {
        self.edges.retain(|e| e.from != id && e.to != id);
    }
}

impl SimComponent for VecBoolEdges {}

#[cfg(test)]
#[path = "vec_bool_edges_tests.rs"]
mod tests;
