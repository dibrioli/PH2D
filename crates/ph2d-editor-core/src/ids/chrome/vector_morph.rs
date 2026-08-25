//! **As SETAS do Morph** — os `NodeId` da lista de arestas na seção *States* (plano 32 W4).
//!
//! ⚠️ Irmão do [`super::vector_states`]: ali *que poses esta forma tem*, aqui *que setas esta
//! máquina tem*. As duas vivem na mesma seção porque o Inspector mostra **o que o objecto TEM**
//! ([ADR-0166](../../../../docs/architecture/decisions/)) — e um objecto raramente é as duas
//! coisas.

use super::{NodeId, fnv_node_id_runtime, hash_node_id};

/// **Quantas setas a seção mostra**, e o teto é da UI — não do documento.
///
/// ⚠️ **É o tamanho do POOL de ids que o `populate` regista de antemão**, exactamente como o
/// [`super::vector_states::MAX_SIGNAL_BINDINGS`]: um grafo com mais arestas **funciona** (a máquina
/// percorre todas), a seção é que não as mostra todas. ⛔ Não é um limite de recurso e não se
/// mede — é a fronteira entre o que está registado e o que estaria **morto sob o ponteiro**.
///
/// **12** e não 6: uma máquina útil tem ida e volta entre os estados, então o número de setas
/// cresce com o QUADRADO das formas — três formas totalmente ligadas já são seis.
pub const MAX_MORPH_ARROWS: usize = 12;

/// **Quantas acções o menu da condição oferece.** Mesmo argumento: é o pool de ids do popover.
///
/// ⚠️ Um mapa com mais acções continua a funcionar; o menu mostra as primeiras. O número acompanha
/// o que um projecto real tem (o mapa de fábrica traz **seis**).
pub const MAX_MORPH_ACTIONS: usize = 24;

/// O chip da CONDIÇÃO da seta `row` — abre o menu das acções do Input Map.
#[must_use]
pub fn morph_arrow_when_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.morph.arrow.when.{row}"))
}

/// A opção `action` no menu da condição da seta `row`.
///
/// ⚠️ **O índice `0` é o «—»** (sem condição): ele existe porque *tirar* a condição tem de ser um
/// gesto, e sem ele o artista só poderia apagar a seta inteira para se arrepender.
#[must_use]
pub fn morph_arrow_when_option_id(row: usize, action: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.morph.arrow.when.{row}.{action}"))
}

/// Apagar a seta `row`.
#[must_use]
pub fn morph_arrow_delete_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.morph.arrow.delete.{row}"))
}

/// O cabeçalho da sub-lista das setas, dentro da seção *States*.
pub const VECTOR_MORPH_ARROWS_LABEL: NodeId = hash_node_id("vector.morph.arrows.label");
