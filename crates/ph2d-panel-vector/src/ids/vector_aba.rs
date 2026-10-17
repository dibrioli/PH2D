//! **Os ids das ABAS do painel** (F17, dono 16/10) — «Drawing» / «Interface» no título, o
//! interruptor no fim da coluna ① e o grupo «Advanced» do corpo.
//!
//! ⚠️ **Bloco APPEND-ONLY.** Os três cliques (duas abas + interruptor) acabam na MESMA escrita
//! (`state::set_aba`).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// A fila das abas (`Role::TabList`) — o nome do painel no a11y.
pub const VECTOR_TABS: NodeId = hash_node_id("vector.tabs");

/// A aba **Drawing** — a geometria, a cor e os efeitos.
pub const VECTOR_TAB_DRAWING: NodeId = hash_node_id("vector.tab.drawing");

/// A aba **Interface** — componente, moldura, layout, âncoras, pele e estados.
pub const VECTOR_TAB_INTERFACE: NodeId = hash_node_id("vector.tab.interface");

/// ① **O interruptor da vista de interface** — no fim da coluna, separado das ferramentas: troca a
/// aba (Drawing ⇄ Interface) e o palco tinge. Não é ferramenta: o clique no canvas não muda.
pub const VECTOR_VIEW_INTERFACE: NodeId = hash_node_id("vector.view.interface");

/// ▸ **Advanced** — o grupo recolhido das secções de desenho raras (Expand, Blend, Morph, Envelope,
/// Pattern on Path, Contour, Boolean). Um cabeçalho dobrável; dobrado, nenhum membro pinta.
pub const VECTOR_SECTION_ADVANCED: NodeId = hash_node_id("vector.section.advanced");
