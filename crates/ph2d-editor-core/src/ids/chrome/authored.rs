//! **Os ids do painel AUTORADO** (plano UI/UX W8b.2) — o painel que o artista desenhou, vivo.
//!
//! # O que este painel é, e por que os ids das rows são DERIVADOS
//!
//! A W8b.1 fez a árvore autorada descrever um painel e o app escrever o código dele. Este é o
//! outro lado: a tabela emitida é **compilada** e vira `populate`/`paint`/`apply_event` sobre os
//! widgets do catálogo. A lista de rows, portanto, **não existe em tempo de escrita destes ids** —
//! ela é o que o artista desenhou.
//!
//! ⇒ O id de cada row sai da **CHAVE** dela (`hash("authored.row.<chave>")`), como o
//! `wet_tuning_slider_id` sai da chave do knob e o `tokens_swatch_id` sai do índice. Uma const
//! por row seria impossível de escrever (ninguém sabe quantas), e um teto arbitrário deixaria as
//! rows além dele **pintadas e mortas sob o rato**.
//!
//! ⚠️ **E o gerador NÃO cunha ids**, o que é a razão de a chave existir: um `NodeId` literal num
//! arquivo gerado teria de entrar no `node_id_collisions`, e um gerador que cunha ids é um
//! gerador que pode cunhar o mesmo duas vezes. A unicidade da família contra o chrome estático é
//! gateada na crate do painel, que vê as chaves REAIS.
//!
//! # Duas rows de mesmo rótulo colidem, e isso está NOMEADO
//!
//! A chave é o slug do rótulo, então dois filhos chamados *"Opacity"* dão o mesmo id. Elas são o
//! mesmo controle autorado duas vezes, e desempatar nomes que o artista repetiu não é decisão do
//! gerador — é dele, na Hierarquia. O painel **avisa** (a linha de rótulos repetidos), em vez de
//! inventar um sufixo que o artista não escreveu e não consegue prever.

use ph2d_a11y::NodeId;

use super::painter::fnv_node_id_runtime;
use crate::ids::hash_node_id;

/// O retângulo externo do painel (z-order + barreira de hit + roteamento da roda).
pub const AUTHORED_PANEL: NodeId = hash_node_id("authored.panel");
/// O botão de fechar (X).
///
/// ⚠️ Ele escreve a MESMA visibilidade que o interruptor da seção Frame lê — um painel cujo X e
/// cujo abridor discordassem seria a falha de duas-portas na sua forma mais visível: o artista
/// fecha, o interruptor continua aceso, e clicar nele não faz nada.
pub const AUTHORED_CLOSE: NodeId = hash_node_id("authored.close");

/// Faixa de arraste do título (move o painel), parenteada a [`AUTHORED_PANEL`].
pub const AUTHORED_DRAG_HANDLE: NodeId = hash_node_id("authored.drag_handle");
/// Punho de redimensionar, canto inferior-direito.
pub const AUTHORED_RESIZE_HANDLE: NodeId = hash_node_id("authored.resize_handle");
/// Punho de redimensionar, canto inferior-esquerdo.
pub const AUTHORED_RESIZE_HANDLE_BL: NodeId = hash_node_id("authored.resize_handle_bl");

/// O id da row de chave `key`.
///
/// ⚠️ O twin de runtime do [`hash_node_id`] — o mesmo FNV-1a, gateado a concordar com a `const fn`
/// (`fnv_node_id_runtime_agrees_with_hash_node_id`). Duas funções de hash dariam um id no
/// `populate` e outro no `paint`, e o controle nasceria morto sob o rato.
#[must_use]
pub fn authored_row_id(key: &str) -> NodeId {
    fnv_node_id_runtime(&format!("authored.row.{key}"))
}
