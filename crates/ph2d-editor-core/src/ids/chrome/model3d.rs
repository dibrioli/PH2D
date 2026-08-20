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

use super::painter::fnv_node_id_runtime;
use crate::ids::hash_node_id;

/// O retângulo externo do painel — z-order, barreira de hit e roteamento da roda.
pub const MODEL3D_PANEL: NodeId = hash_node_id("model3d.panel");
/// O botão de fechar (X).
pub const MODEL3D_CLOSE: NodeId = hash_node_id("model3d.close");

/// ⭐ **O botão de um VERBO do gizmo** (mover / rodar / escalar), pela posição no seletor.
///
/// ⚠️ Pela POSIÇÃO, e não pelo nome do verbo: o `populate` corre antes de o gizmo existir e cunha a
/// família às cegas, exatamente como faz com as linhas de raio. Quem escolhe o verbo é o retrato
/// publicado, que diz o que cada posição significa naquele quadro.
#[must_use]
pub fn model3d_mode_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.mode.{slot}"))
}

/// ⭐ **O botão de um REFERENCIAL de eixos** (global / local), pela posição no seletor.
///
/// ⚠️ Família própria, e não a dos verbos: os dois seletores coexistem no painel, e partilhar a
/// família faria um clique em «Local» disparar o verbo da mesma posição.
#[must_use]
pub fn model3d_frame_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.frame.{slot}"))
}

/// ⭐ **O botão de ACRESCENTAR uma forma** (caixa, esfera, cilindro, toro), pela posição.
#[must_use]
pub fn model3d_add_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.add.{slot}"))
}

/// ⭐ **O botão de uma OPERAÇÃO booleana** (unir, subtrair, intersectar), pela posição.
#[must_use]
pub fn model3d_op_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.op.{slot}"))
}

/// O **slider do raio** do nó `node` da arena.
#[must_use]
pub fn model3d_radius_slider(node: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.radius.slider.{node}"))
}

/// O **campo numérico** do raio do nó `node` — o gêmeo do slider, para digitar o valor exato.
///
/// ⚠️ Existe porque um slider sozinho **não consegue** exprimir um raio de CAD: o artista que quer
/// 2,5 mm quer 2,5 mm, não "onde o pixel calhou". Os dois estão ligados no store, então mexer num
/// move o outro.
#[must_use]
pub fn model3d_radius_chip(node: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.radius.chip.{node}"))
}
