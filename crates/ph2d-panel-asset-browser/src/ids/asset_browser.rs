//! **Os ids do NAVEGADOR DE ASSETS** (plano `docs/Components/07`, waves A4/A5/A7).
//!
//! ⚠️ A porta de entrada **não** é um id novo: é o pill `TOPBAR_RIGHT_ASSETS`, que existe e é
//! pintado desde sempre — e que até 2026-08-30 **não tinha despacho nenhum**. Ele era um dos três
//! chips mortos daquele grupo (Layers · Assets · Script), a espécie que o §5.0 do `CLAUDE.md`
//! descreve: *pintado, registado, hit-indexado — e nenhum leitor decide nada com ele.*
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/asset_browser.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

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

/// ⭐⭐ **Largar o filtro de relação** — o `✕` da faixa que diz *o que usa X* / *o que X usa*.
///
/// ⚠️ **Um modo sem saída visível é uma armadilha**, e este tem-na porque a faixa é o único sítio
/// onde o filtro se vê: ao contrário do chip de família e da linha de catálogo, ele não tem
/// controlo permanente no cabeçalho — ele nasce de um menu e desaparece quando se larga. *Um
/// filtro que só um menu liga tem de trazer o próprio interruptor de desligar.*
pub const ASSET_RELATED_CLEAR: NodeId = hash_node_id("asset_browser.related.clear");

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

/// O interruptor da coluna — o botão *só-grade* que a decisão D2 do plano 07 pedia, e que sobrou
/// dela depois de a §10 reverter o resto.
pub const ASSET_CATALOG_TOGGLE: NodeId = hash_node_id("asset_browser.catalog.toggle");

/// **+ New catalog** — cria um catálogo dentro do escolhido (ou na raiz, se for *All*).
pub const ASSET_CATALOG_NEW: NodeId = hash_node_id("asset_browser.catalog.new");

/// A linha **All** — sem filtro.
pub const ASSET_CATALOG_ALL: NodeId = hash_node_id("asset_browser.catalog.all");

/// A linha **Unassigned** — os que não estão em catálogo nenhum.
///
/// ⚠️ Ela é uma LINHA e não um estado escondido: sem ela, um asset por arrumar fica inalcançável
/// no dia em que existir um catálogo (ver `CatalogScope`).
pub const ASSET_CATALOG_UNASSIGNED: NodeId = hash_node_id("asset_browser.catalog.unassigned");

/// O campo de renomeação in-place de uma linha de catálogo.
///
/// ⚠️ **Um id FIXO, e não um por linha:** só uma renomeação existe de cada vez, e o texto vive no
/// `WidgetStore` como o de qualquer campo — é isso que faz a rota global de foco alimentá-lo e o
/// `text_entry_focused` do shell suprimir os atalhos enquanto se escreve, sem gate extra.
///
/// ⛔ Ele **não pode ter tabelas laterais que o `register` deixe para trás** (cor, z, rolagem,
/// tooltip): a abertura usa o `register` que SUBSTITUI, e isso só é seguro porque o id não as tem
/// — a mesma nota que o rename da Hierarquia carrega.
///
/// ⚠️ **O `cancel_on_escape` NÃO é uma dessas, e a distinção é o que salva a nota** (auditada em
/// 2026-08-30): ele é um conjunto *insert-only* que ninguém esvazia, então marcá-lo uma vez basta
/// e um `register` a seguir não o perde. *Uma tabela lateral só é perigosa se o `register` a
/// dessincronizar.*
pub const ASSET_CATALOG_RENAME: NodeId = hash_node_id("asset_browser.catalog.rename");

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::ids::{
        ASSET_PANEL, MAX_CATALOG_ROWS, asset_cell_id, catalog_row_id, catalog_row_index,
    };

    /// O gémeo de runtime tem de concordar com o hasher `const` — senão os ids das células vivem
    /// noutro espaço e o hit-test nunca os resolve.
    #[test]
    fn the_runtime_hasher_agrees_with_the_const_one() {
        assert_eq!(
            ph2d_tool_registry::hash_node_id_runtime("asset_browser.panel"),
            ASSET_PANEL
        );
        assert_eq!(
            ph2d_tool_registry::hash_node_id_runtime("asset_browser.search"),
            ASSET_SEARCH
        );
    }

    /// ⭐⭐ **A escada lê-se nos dois sentidos, e os dois sentidos concordam.**
    ///
    /// ⚠️ Esta varredura estava escrita **três** vezes fora daqui, e a 4.ª cópia ia nascer no
    /// despachante do botão direito. Um gate sobre a PORTA é o que torna as cópias desnecessárias
    /// — e é o que apanha um `catalog_row_id` que mude de esquema sem o inverso o acompanhar.
    ///
    /// **Mutação que deve sangrar:** trocar o `find` por `Some(0)`.
    #[test]
    fn the_catalog_row_ladder_round_trips() {
        for i in [0usize, 1, 2, 7, MAX_CATALOG_ROWS - 1] {
            assert_eq!(catalog_row_index(catalog_row_id(i)), Some(i));
        }
        // ⛔ E um id que não é da escada não é uma linha — nem o do painel, nem o da linha logo
        // acima do tecto.
        assert_eq!(catalog_row_index(ASSET_PANEL), None);
        assert_eq!(catalog_row_index(catalog_row_id(MAX_CATALOG_ROWS)), None);
        // ⛔⛔ **E os ids do MENU não são linhas.** A arm da escada vem ANTES da arm do menu no
        // `event_catalog`, então uma colisão de hash engoliria o `Rename…`/`Delete` **em
        // silêncio** — o clique escolheria uma linha em vez de renomear. (Auditoria de 30/08.)
        for menu in [
            ph2d_editor_core::ids::CTX_MENU_CATALOG_RENAME,
            ph2d_editor_core::ids::CTX_MENU_CATALOG_DELETE,
        ] {
            assert_eq!(catalog_row_index(menu), None);
        }
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
