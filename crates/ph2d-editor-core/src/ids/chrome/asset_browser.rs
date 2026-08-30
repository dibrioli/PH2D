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

// ── ⭐⭐ A COLUNA DE CATÁLOGOS (wave A3) ────────────────────────────────────────────────────────

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
/// A chave de ROLAGEM da coluna.
///
/// ⚠️ **Ela não é um painel, e o nome di-lo.** As tabelas `panel_scroll`/`panel_content_h`/
/// `panel_visible_h` aceitam qualquer `NodeId` como chave — é o que o popover do dropdown já faz.
/// ⛔ E é de propósito que ela **não** acaba em `_PANEL`: o gate
/// `scrollable_panels_intercept_the_wheel` só recolhe identificadores com esse sufixo, e a roda
/// sobre a coluna já é interceptada pelo painel que a contém.
pub const ASSET_CATALOG_COL: NodeId = hash_node_id("asset_browser.catalog.col");

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

/// Quantas linhas de catálogo o painel regista, no máximo. ⚠️ Mesmo teto e mesma razão do
/// [`MAX_ASSET_CELLS`]: cada linha é um `NodeId` no store e um rect no `HitIndex`.
pub const MAX_CATALOG_ROWS: usize = 256;

/// A escada inteira, assada **uma vez**.
///
/// ⛔⛔ **Sem ela a leitura inversa punha 256 `format!` + 256 FNV no caminho de TODO botão direito
/// do app** (achado da auditoria de 2026-08-30): o `pointer_down_menus` avalia-a
/// incondicionalmente dentro do ramo `Secondary`, portanto o canvas, a hierarquia e a timeline
/// pagavam o caso do MISS. ⚠️ **E o gate de zero-alocação era estruturalmente cego a isso** — o
/// `interaction_no_alloc` só despacha `Move` com o botão **primário**.
///
/// ⚠️ Ela também tira o `format!` do **laço que pinta** a coluna, que corria por quadro.
static CATALOG_ROWS: std::sync::LazyLock<Vec<NodeId>> = std::sync::LazyLock::new(|| {
    (0..MAX_CATALOG_ROWS)
        .map(|i| asset_fnv_node_id(&format!("asset_browser.catalog.row.{i}")))
        .collect()
});

/// O id da linha `index` da coluna — **posicional na lista visível**, como o cartão.
///
/// ⚠️ Fora da escada (`index >= MAX_CATALOG_ROWS`) ele continua a ser calculado, porque a resposta
/// tem de existir para o gate que afirma que ela **não** é uma linha.
#[must_use]
pub fn catalog_row_id(index: usize) -> NodeId {
    match CATALOG_ROWS.get(index) {
        Some(id) => *id,
        None => asset_fnv_node_id(&format!("asset_browser.catalog.row.{index}")),
    }
}

/// ⭐⭐ **A leitura INVERSA da escada** — `id` é uma linha de catálogo, e qual?
///
/// ⚠️ **Ela existe porque a mesma varredura estava escrita TRÊS vezes** (o `event.rs` do painel, o
/// `catalog_row_pick` do estado dele, e o despachante do botão direito que a ia escrever a quarta).
/// *Uma lei escrita em N sítios ainda não é uma lei — só uma PORTA é.*
///
/// ⚠️ **Ela responde sobre o ESPAÇO de ids, não sobre o que foi pintado**: quem precisa da linha
/// viva pergunta ao censo do quadro (`catalog_row_pick`). Para o hit-test isto basta e é exacto —
/// o `HitIndex` só contém rects registados neste quadro.
#[must_use]
pub fn catalog_row_index(id: NodeId) -> Option<usize> {
    CATALOG_ROWS.iter().position(|c| *c == id)
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
            crate::ids::CTX_MENU_CATALOG_RENAME,
            crate::ids::CTX_MENU_CATALOG_DELETE,
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
