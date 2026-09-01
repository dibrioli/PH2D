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

// ⛔⛔ **`model3d_mode_button` e `model3d_frame_button` MORRERAM em 2026-09-01.** Elas eram o
// seletor de verbo e o de eixos DENTRO do painel — e o trilho já tinha os chips `MOVE`/`ROT`/
// `SCALE` e o `SPACE` a fazer a mesma pergunta, pintados e sem consumidor nenhum (Enio, com foto:
// *«esses botões já existiam. só não estavam ligados a cada modo»*).
//
// ⚠️ **Ligar os que existem e apagar estes é UMA obra, não duas.** Ficar com as duas famílias
// deixaria o app com dois sítios para o mesmo verbo, e o que apodrece é o que ninguém relê. Quem
// conduz o gizmo hoje é `ph2d_panel_model3d::area_bar::rail_verb_slot`, pela CHAVE do verbo.

/// ⭐⭐ **O botão do MODO DO LAÇO** (somar / subtrair à seleção), pela posição no seletor.
///
/// ⚠️ Família própria pela mesma razão do referencial: os seletores coexistem no painel, e
/// partilhar a família faria um clique em «Subtract» disparar o verbo da mesma posição.
#[must_use]
pub fn model3d_select_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.select.{slot}"))
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

/// ⭐ **O botão de um MODIFICADOR** (casca, afastamento), pela posição.
///
/// ⚠️ Família própria, como as outras: um interruptor de modificador e um botão de operação vivem
/// no mesmo painel, e partilhar a família faria «Casca» disparar «Unir».
#[must_use]
pub fn model3d_mod_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.mod.{slot}"))
}

/// ⭐⭐⭐ **O botão do VERBO DESTA FORMA** — com que operação ela dobra sobre as anteriores.
///
/// ⚠️ **Família própria, e não a do `model3d_op_button`.** As duas fileiras dizem as mesmas
/// palavras e têm sujeitos diferentes: aquela é a operação **do grupo** (o padrão de quem não se
/// pronunciou); esta é a escolha **de uma forma**. Partilhar a família faria um clique em «Subtract»
/// na fileira de baixo trocar a operação do grupo inteiro.
#[must_use]
pub fn model3d_verb_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.verb.{slot}"))
}

/// ⭐⭐⭐ **O botão do CARÁCTER da mistura** — a FORMA da transição (arco · corte reto · derretido).
///
/// ⚠️ **Família própria, e não a do verbo:** as duas fileiras aparecem juntas na mesma forma e
/// respondem a perguntas diferentes (*como se junta* · *que forma tem a junta*). Partilhar a família
/// faria um clique em «Chamfer» trocar o verbo.
#[must_use]
pub fn model3d_character_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.character.{slot}"))
}

/// ⭐ **O botão de EXPORTAR** numa resolução, pela posição.
#[must_use]
pub fn model3d_export_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.export.{slot}"))
}

/// ⭐ **O botão de uma AÇÃO sobre o objeto escolhido** (duplicar, apagar), pela posição.
#[must_use]
pub fn model3d_act_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.act.{slot}"))
}

/// ⭐ **O botão de uma VISTA NOMEADA** (frente, topo, …), pela posição no seletor.
#[must_use]
pub fn model3d_view_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.view.{slot}"))
}

/// ⭐ **O botão de um gesto de CÂMERA** que não é uma vista — a lente, o enquadrar.
#[must_use]
pub fn model3d_camera_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.camera.{slot}"))
}

/// ⭐ **A track de motion de uma VIAGEM entre vistas** (ADR-0161 W51), pela geração.
///
/// ⚠️ Um id **por viagem**: a mola lembra-se por id, e reusar um faria a segunda viagem continuar de
/// onde a primeira parou. Ids transientes são podados pelo `UiMotion` (`PRUNE_AFTER_S`), que é
/// exactamente o ciclo de vida para que ele foi feito.
#[must_use]
pub fn model3d_view_travel(generation: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.view.travel.{generation}"))
}

/// ⭐⭐⭐ **UM BOTÃO DE ESCOLHA** de uma linha de painel — a fileira `X / Y / Z` do eixo de um
/// modificador (Enio, 2026-08-31).
///
/// ⚠️ **`cell` é a posição do botão DENTRO da linha**, e `row` é a linha. Os dois num id só porque
/// um `NodeId` é um hash de nome: cunhar `row` e `cell` no nome é o que os mantém distintos sem uma
/// convenção implícita entre duas crates.
#[must_use]
pub fn model3d_choice_button(row: u32, cell: u32) -> NodeId {
    fnv_node_id_runtime(&format!("model3d.choice.{row}.{cell}"))
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
