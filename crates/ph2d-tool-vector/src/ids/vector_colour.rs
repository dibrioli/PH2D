//! **As três portas da COR** da coluna ① (D10, o padrão Illustrator `X` / `Shift+X` / `D`).
//!
//! ⚠️ Cada tecla é um CLIQUE nestes ids (a mesma porta que o botão da coluna e a paleta usam), e
//! cada botão é a porta visível da sua tecla — *todo gesto precisa de uma porta visível ao lado*
//! (estudo §3).
use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// `X`: o foco da cor passa de Preenchimento a Traço (e volta), e o seletor abre no que ficou.
pub const VECTOR_COLOUR_FOCUS: NodeId = hash_node_id("vector.colour.focus");
/// `Shift+X`: troca as cores do Preenchimento e do Traço (na ferramenta e na seleção).
pub const VECTOR_COLOUR_SWAP: NodeId = hash_node_id("vector.colour.swap");
/// `D`: repõe as cores de fábrica da ferramenta (na ferramenta e na seleção).
pub const VECTOR_COLOUR_DEFAULT: NodeId = hash_node_id("vector.colour.default");
