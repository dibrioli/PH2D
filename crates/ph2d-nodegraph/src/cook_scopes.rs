//! **OS TRÊS CANAIS LATERAIS DO COOK** — o que ele precisa de saber e o documento não diz.
//!
//! ⚠️ **O corte foi FORÇADO pelo tecto de LOC** (700, `architecture_workspace_file_loc_cap`) e a
//! costura é por assunto, a mesma que o `cook_lazy` já fez: o pai fica com *como um nó se
//! cozinha*, e isto com as DECLARAÇÕES que a shell lhe entrega — os escopos e os leques (*em que
//! instante uma sub-árvore é lida?*). O terceiro, o plano de preguiça, mora no `cook_lazy`
//! porque ele traz a lei que o executa junto.
//!
//! Os nomes re-exportam-se do pai, então nenhum chamador muda de endereço.

use super::{NodeId, TimeMap};
use std::collections::BTreeMap;

/// Identifies the chain of [`TimeMap`]s a node is being cooked under (plan
/// §1.5). `0` = the outer clock, i.e. no remap — the only key a graph without
/// time scopes ever uses, so its behaviour and memo are exactly as before.
///
/// A node reached through two different scope chains in one frame (a diamond
/// where one arm crosses a remapper) is cooked once **per chain**: the memo is
/// keyed by `(NodeId, ScopeKey)`. Keying by `NodeId` alone would let the second
/// arm read the first arm's stream, silently sampled at the wrong time.
pub type ScopeKey = u64;

/// The `ScopeKey` of the outer clock.
pub const SCOPE_ROOT: ScopeKey = 0;

/// Time scopes to apply while cooking: `node -> map` for each remapper node.
/// The map rewrites the clock of that node's **upstream subtree**, never of the
/// node itself. Built by the domain layer (which knows its node types) — the
/// substrate stays type-agnostic.
pub type TimeScopes = BTreeMap<NodeId, TimeMap>;

/// **Leques de tempo**: `node -> [map]`. A **porta 0** do nó é cozida uma vez por
/// mapa, e as N saídas chegam ao `eval` como um LEQUE
/// ([`crate::cook_eval_ctx::EvalCtx::fan`]) — a mesma sub-árvore, em N instantes.
///
/// É a capacidade que separa *lembrar* de *re-cozinhar*: um rastro que guarda um
/// ring só pode desenhar o PASSADO, porque é isso que um ring contém. Um rastro
/// que re-cozinha a entrada em `t ± k·s` desenha os dois lados, e é exato sob
/// scrub porque nada nele é estado.
///
/// ⚠️ **É um LEQUE, não um escopo.** Um [`TimeScopes`] reescreve o relógio da
/// sub-árvore de um nó **uma vez**; aqui a mesma sub-árvore é cozida **N vezes**,
/// cada uma na sua faixa de memo.
///
/// ⚠️ **E o que a faixa própria compra é MENOS do que parece, medido por
/// mutação:** os valores sairiam certos mesmo com todas as fatias na mesma faixa
/// (dentro do laço cada leitura segue a própria cozedura). O que ela compra é o
/// instante repetido **fora de ordem** — pedir `t−1` depois de `t−2` responde do
/// memo em vez de recomputar —, que é o caso do espaçamento NÃO-UNIFORME. Ver
/// `repeating_an_instant_out_of_order_still_hits_the_memo`.
///
/// ⚠️ **Só a porta 0.** Um leque sobre uma porta de estado não teria significado
/// (um `pre` é o tique anterior, não um instante pedido), e um leque sobre TODAS
/// as portas multiplicaria o custo por uma coisa que nenhum nó pediu. A porta 0
/// é a convenção do módulo para *a entrada*.
pub type TimeFans = BTreeMap<NodeId, Vec<TimeMap>>;

/// Push `map` (applied at `node`) onto a scope chain. FNV-1a over the node id
/// and the map's bits, so distinct chains key distinct memo lanes.
pub(super) fn push_scope(key: ScopeKey, node: NodeId, map: &TimeMap) -> ScopeKey {
    let mut hash = if key == SCOPE_ROOT {
        0xcbf2_9ce4_8422_2325
    } else {
        key
    };
    for b in node.0.to_le_bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    map.hash_into(&mut hash);
    // Never collide with the root: a scoped lane must not alias the unscoped one.
    if hash == SCOPE_ROOT { 1 } else { hash }
}
