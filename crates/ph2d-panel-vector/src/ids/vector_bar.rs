//! **Os ids da BARRA DE CONTROLE** (F16, dono 15/10) — a fila ② à maneira da *Control bar* do
//! Illustrator: os três chips que abrem uma lista ou um painel POR BAIXO deles.
//!
//! ⚠️ **Bloco APPEND-ONLY.** As LINHAS das listas são os ids que o painel já tinha (os perfis de
//! largura, as tintas do traço): o chip da fila é só outra porta para o mesmo verbo.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **Stroke ▾** — abre o painel do traço (pontas, juntas, alinhamento, tracejado, setas, perfil)
/// por baixo do chip. Um `Dropdown` do store: o light-dismiss genérico fecha-o num clique fora.
pub const VECTOR_BAR_STROKE: NodeId = hash_node_id("vector.bar.stroke");

/// **Perfil ▾** — a lista dos perfis de largura (Uniform, Taper…) da forma selecionada.
pub const VECTOR_BAR_PROFILE: NodeId = hash_node_id("vector.bar.profile");

/// **Tinta ▾** — com que tinta o traço desenha (Solid, Pattern, Brush).
pub const VECTOR_BAR_KIND: NodeId = hash_node_id("vector.bar.kind");
