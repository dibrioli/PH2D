//! Os ids da seção **Component** do Inspector (ADR-0164 / F5).

use super::hash_node_id;
use ph2d_a11y::NodeId;

/// O cabeçalho da seção — o que o `begin_section` regista para a nota e o recolher.
pub const INSP_INSTANCE_SECTION: NodeId = hash_node_id("insp_instance_section");

/// ⭐ **Limpar as excepções SEM ALVO** (F5.3).
///
/// ⚠️ **Um botão, e não uma limpeza automática:** a lei do *«unused overrides»* do Unity é que
/// elas **nunca** se apagam sozinhas — sair por causa de um `Delete` no mestre é perder trabalho
/// do artista em silêncio. ⇒ o gesto existe, e é explícito.
pub const INSP_INSTANCE_CLEAR_ORPHANS: NodeId = hash_node_id("insp_instance_clear_orphans");
