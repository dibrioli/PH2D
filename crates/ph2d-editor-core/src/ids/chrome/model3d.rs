//! **Os ids do painel de MODELAGEM 3D** (ADR-0161) — o seletor do verbo do gizmo (W6) e o raio de
//! cada operação, editável ao vivo (W4).
//!
//! ⚠️ **Não confundir com `sculpt3d`**, que é o painel do módulo de **escultura**. São dois módulos
//! 3D, duas linhas, e dois prefixos de id que nunca se cruzam.
//!
//! # As linhas são DERIVADAS, e por quê
//!
//! Uma linha do painel é um **nó do documento** com raio editável — e quantos nós um documento tem
//! é o que o artista modelou, não algo que se saiba ao escrever estes ids. Então o id sai do
//! **índice do nó na arena** (`hash("model3d.radius.<n>")`), como o `tokens_swatch_id` sai do
//! índice da linha.
//!
//! ⚠️ O índice é **estável enquanto a arena não muda de forma**, que é a mesma garantia que a
//! própria arena dá (todo filho antes do pai, e os índices são a identidade dos nós). Se um dia
//! houver inserção no meio, os ids das linhas seguintes andam — e o sintoma seria o foco do teclado
//! saltar de linha, não uma forma errada.

use ph2d_a11y::NodeId;

use crate::ids::hash_node_id;
use ph2d_tool_registry::hash_node_id_runtime;

/// O retângulo externo do painel — z-order, barreira de hit e roteamento da roda.
pub const MODEL3D_PANEL: NodeId = hash_node_id("model3d.panel");

// ⛔⛔ **`model3d_mode_button` e `model3d_frame_button` MORRERAM em 2026-09-01.** Elas eram o
// seletor de verbo e o de eixos DENTRO do painel — e o trilho já tinha os chips `MOVE`/`ROT`/
// `SCALE` e o `SPACE` a fazer a mesma pergunta, pintados e sem consumidor nenhum (Enio, com foto:
// *«esses botões já existiam. só não estavam ligados a cada modo»*).
//
// ⚠️ **Ligar os que existem e apagar estes é UMA obra, não duas.** Ficar com as duas famílias
// deixaria o app com dois sítios para o mesmo verbo, e o que apodrece é o que ninguém relê. Quem
// conduz o gizmo hoje é `ph2d_panel_model3d::area_bar::rail_verb_slot`, pela CHAVE do verbo.

/// ⭐ **A track de motion de uma VIAGEM entre vistas** (ADR-0161 W51), pela geração.
///
/// ⚠️ Um id **por viagem**: a mola lembra-se por id, e reusar um faria a segunda viagem continuar de
/// onde a primeira parou. Ids transientes são podados pelo `UiMotion` (`PRUNE_AFTER_S`), que é
/// exactamente o ciclo de vida para que ele foi feito.
#[must_use]
pub fn model3d_view_travel(generation: u32) -> NodeId {
    hash_node_id_runtime(&format!("model3d.view.travel.{generation}"))
}
