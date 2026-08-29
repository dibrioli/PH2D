//! Os ids da seção **Component** do Inspector (ADR-0164 / F5).

use super::hash_node_id;
use ph2d_a11y::NodeId;

// ⛔ **Havia um `INSP_INSTANCE_SECTION` e ele SAIU** (Enio, 2026-08-27): a superfície deixou de ser
// uma seção e passou a ser um CARTÃO no topo, que não tem cabeçalho, nem recolher, nem âncora de
// nota — logo não tem o que registar. *Um id que nada regista é um id que nada resolve.*

/// ⭐ **Limpar as excepções SEM ALVO** (F5.3).
///
/// ⚠️ **Um botão, e não uma limpeza automática:** a lei do *«unused overrides»* do Unity é que
/// elas **nunca** se apagam sozinhas — sair por causa de um `Delete` no mestre é perder trabalho
/// do artista em silêncio. ⇒ o gesto existe, e é explícito.
pub const INSP_INSTANCE_CLEAR_ORPHANS: NodeId = hash_node_id("insp_instance_clear_orphans");
