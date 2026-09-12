//! **Os ids do NAVEGADOR DE ASSETS** (plano `docs/Components/07`, waves A4/A5/A7).
//!
//! ⚠️ A porta de entrada **não** é um id novo: é o pill `TOPBAR_RIGHT_ASSETS`, que existe e é
//! pintado desde sempre — e que até 2026-08-30 **não tinha despacho nenhum**. Ele era um dos três
//! chips mortos daquele grupo (Layers · Assets · Script), a espécie que o §5.0 do `CLAUDE.md`
//! descreve: *pintado, registado, hit-indexado — e nenhum leitor decide nada com ele.*

use super::{NodeId, hash_node_id};

/// O rectângulo exterior do painel flutuante.
pub const ASSET_PANEL: NodeId = hash_node_id("asset_browser.panel");

/// O id do cartão em `index` da lista **filtrada e ordenada**.
///
/// ⚠️ Posicional na lista filtrada, e não no índice inteiro: é o que faz o cartão debaixo do dedo
/// ser sempre o que o artista vê, com qualquer busca activa. A ordem da lista é **total** nos três
/// modos (o índice garante-o), então este `index` não muda entre dois quadros com o mesmo filtro.
#[must_use]
pub fn asset_cell_id(index: usize) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("asset_browser.cell.{index}"))
}

// ── ⭐⭐ A COLUNA DE CATÁLOGOS (wave A3) ────────────────────────────────────────────────────────

/// A chave de ROLAGEM da coluna.
///
/// ⚠️ **Ela não é um painel, e o nome di-lo.** As tabelas `panel_scroll`/`panel_content_h`/
/// `panel_visible_h` aceitam qualquer `NodeId` como chave — é o que o popover do dropdown já faz.
/// ⛔ E é de propósito que ela **não** acaba em `_PANEL`: o gate
/// `scrollable_panels_intercept_the_wheel` só recolhe identificadores com esse sufixo, e a roda
/// sobre a coluna já é interceptada pelo painel que a contém.
pub const ASSET_CATALOG_COL: NodeId = hash_node_id("asset_browser.catalog.col");

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
static CATALOG_ROWS: std::sync::LazyLock<Vec<NodeId>> =
    std::sync::LazyLock::new(|| (0..MAX_CATALOG_ROWS).map(catalog_row_hash).collect());

/// O id da linha `index` da coluna — **posicional na lista visível**, como o cartão.
///
/// ⚠️ Fora da escada (`index >= MAX_CATALOG_ROWS`) ele continua a ser calculado, porque a resposta
/// tem de existir para o gate que afirma que ela **não** é uma linha.
#[must_use]
pub fn catalog_row_id(index: usize) -> NodeId {
    match CATALOG_ROWS.get(index) {
        Some(id) => *id,
        None => catalog_row_hash(index),
    }
}

/// **A lei da linha, escrita UMA vez** — a escada assada e o fora-da-escada leem-na daqui.
///
/// ⚠️ O molde `asset_browser.catalog.row.{…}` estava escrito DUAS vezes neste ficheiro (a escada e
/// o `None` acima), e o censo derivado de colisões (`node_id_collisions`) acusou-o: mudar um sem o
/// outro faria a linha 300 viver noutro espaço de ids que a 299, em silêncio.
fn catalog_row_hash(index: usize) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("asset_browser.catalog.row.{index}"))
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
