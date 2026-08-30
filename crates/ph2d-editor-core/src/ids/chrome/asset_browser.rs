//! **Os ids do NAVEGADOR DE ASSETS** (plano `docs/Components/07`, waves A4/A5/A7).
//!
//! ⚠️ A porta de entrada **não** é um id novo: é o pill `TOPBAR_RIGHT_ASSETS`, que existe e é
//! pintado desde sempre — e que até 2026-08-30 **não tinha despacho nenhum**. Ele era um dos três
//! chips mortos daquele grupo (Layers · Assets · Script), a espécie que o §5.0 do `CLAUDE.md`
//! descreve: *pintado, registado, hit-indexado — e nenhum leitor decide nada com ele.*

use super::{NodeId, hash_node_id};

/// O rectângulo exterior do painel flutuante.
pub const ASSET_PANEL: NodeId = hash_node_id("asset_browser.panel");
/// A faixa de arrasto do título.
pub const ASSET_DRAG_HANDLE: NodeId = hash_node_id("asset_browser.drag_handle");
/// A alça de redimensionar (canto inferior esquerdo, como os irmãos).
pub const ASSET_RESIZE_HANDLE_BL: NodeId = hash_node_id("asset_browser.resize_bl");
/// O `X` do cabeçalho.
pub const ASSET_CLOSE: NodeId = hash_node_id("asset_browser.close");

/// **A busca da GRADE.**
///
/// ⚠️ **A segunda busca — a dos CATÁLOGOS — não existe ainda, e a ausência é declarada:** ela só
/// tem sujeito quando a árvore de catálogos existir (wave A3). O plano 07 D1 chama-lhes duas
/// porque o dock do Godot, que é o único com largura tão estreita como a nossa, é o único que as
/// separa. *Registar aqui um id que nada pinta seria um id órfão* — a terceira espécie do §5.0,
/// cuja cura é oposta à do knob morto.
pub const ASSET_SEARCH: NodeId = hash_node_id("asset_browser.search");

/// O slider do tamanho do cartão (plano 07 D3 — **um slider, não presets**, que é o
/// `thumbnail_size_slider` do Godot).
pub const ASSET_SIZE: NodeId = hash_node_id("asset_browser.size");

/// Quantos modos de ordenação a fileira endereça — **é o comprimento de `SortBy::ALL`**, e há gate
/// a ligar os dois. Um chip a mais aqui é um chip que nada pinta; um a menos é um modo inalcançável.
pub const ASSET_SORT_MODES: usize = 3;

/// Os chips de ordenação da grade (plano 07 D6).
pub const ASSET_SORT: [NodeId; ASSET_SORT_MODES] = [
    hash_node_id("asset_browser.sort.0"),
    hash_node_id("asset_browser.sort.1"),
    hash_node_id("asset_browser.sort.2"),
];

/// Quantos filtros de família a fileira endereça: **todas, mais uma por família**.
pub const ASSET_KIND_FILTERS: usize = 3;

/// Os chips de família (`All` · `Component` · `Image`).
pub const ASSET_KIND: [NodeId; ASSET_KIND_FILTERS] = [
    hash_node_id("asset_browser.kind.0"),
    hash_node_id("asset_browser.kind.1"),
    hash_node_id("asset_browser.kind.2"),
];

/// **Quantos cartões a grade endereça de uma vez.**
///
/// ⚠️ **Teto de TABELA DE IDS, e ele diz de que recurso é:** cada célula é um `NodeId` registado
/// no `WidgetStore` e um rectângulo no `HitIndex`, e as duas estruturas são varridas por gesto.
/// ⛔ **Não é teto do índice** — um projecto pode ter os assets que quiser; o que passa daqui
/// continua a existir, continua a ser encontrável pela busca, e o painel **diz quantos ficaram de
/// fora** em vez de os truncar em silêncio.
pub const MAX_ASSET_CELLS: usize = 512;

/// O id do cartão em `index` da lista **filtrada e ordenada**.
///
/// ⚠️ Posicional na lista filtrada, e não no índice inteiro: é o que faz o cartão debaixo do dedo
/// ser sempre o que o artista vê, com qualquer busca activa. A ordem da lista é **total** nos três
/// modos (o índice garante-o), então este `index` não muda entre dois quadros com o mesmo filtro.
#[must_use]
pub fn asset_cell_id(index: usize) -> NodeId {
    asset_fnv_node_id(&format!("asset_browser.cell.{index}"))
}

/// O gémeo de runtime do `hash_node_id`, **a PORTA e não uma cópia**
/// ([`ph2d_tool_registry::hash_node_id_runtime`]).
///
/// ⚠️ **A cópia à mão que aqui esteve tinha o PRIMO errado** (`0x1000_0000_01b3` em vez de
/// `0x0000_0100_0000_01b3`) — os ids das células caíam noutro espaço e o hit-test nunca os
/// resolveria. O gate abaixo apanhou-o; a cura foi promover a lei a porta única.
fn asset_fnv_node_id(slug: &str) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O gémeo de runtime tem de concordar com o hasher `const` — senão os ids das células vivem
    /// noutro espaço e o hit-test nunca os resolve.
    #[test]
    fn the_runtime_hasher_agrees_with_the_const_one() {
        assert_eq!(asset_fnv_node_id("asset_browser.panel"), ASSET_PANEL);
        assert_eq!(asset_fnv_node_id("asset_browser.search"), ASSET_SEARCH);
    }

    /// Duas células diferentes são dois ids diferentes — e a célula 0 não colide com nenhum dos
    /// ids fixos do painel.
    #[test]
    fn cells_are_distinct_from_each_other_and_from_the_fixed_ids() {
        assert_ne!(asset_cell_id(0), asset_cell_id(1));
        for fixed in [ASSET_PANEL, ASSET_SEARCH, ASSET_SIZE, ASSET_CLOSE] {
            for i in 0..8 {
                assert_ne!(asset_cell_id(i), fixed);
            }
        }
    }
}
