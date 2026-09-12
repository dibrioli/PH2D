//! **§14 Platform Player** — os ids da seção de COMPORTAMENTO (W5).
//!
//! ⚠️ **Seção própria, e sem SELETOR** (D9 do plano). Um seletor de um item é um
//! controle morto: ele pede uma escolha onde não há escolha, e o custo dele é
//! permanente (uma row para sempre) enquanto o benefício é hipotético. O ponto
//! de extensão de *"que comportamentos este corpo tem?"* é o **componente** —
//! um comportamento novo é uma seção nova, exatamente como um joint novo é um
//! `JointKind` novo e não um dropdown dentro do Pin.
//!
//! ⚠️ **E ela é irmã da §11, não parte dela.** A §11 responde *"que corpo é
//! este?"* (massa, forma, material) e a §14 responde *"que comportamento este
//! corpo tem?"*. Colapsá-las daria a um estado de colapso dois assuntos, e o
//! artista que fecha "Physics Body" para ver a lista de objetos perderia os
//! controles do personagem junto.

use super::hash_node_id;
use ph2d_a11y::NodeId;

/// **O cabeçalho da §14 — Platform Player** (dono do estado de colapso) e o
/// círculo de cor dele.
pub const INSP_LIVE_PLAYER_SECTION: NodeId = hash_node_id("insp_live_player_section");
pub const INSP_LIVE_PLAYER_COLOR: NodeId = hash_node_id("insp_live_player_color");
