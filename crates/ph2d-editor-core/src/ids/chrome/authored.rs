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

/// O id da opção `index` da row de chave `key` — a família da lista ABERTA.
///
/// ⚠️ **Só quem esconde as opções precisa dela** ([`WidgetKind::defers_a_popover`]): nas abas, no
/// rádio e na segmentada quem regista os segmentos é o pintor do catálogo, dentro do retângulo da
/// row. Um dropdown aberto desenha as opções numa superfície que só existe enquanto está aberto,
/// e cada uma precisa de um retângulo de hit PRÓPRIO — senão a lista pinta e o clique cai na row
/// por baixo dela.
///
/// ⚠️ **O índice, e não o rótulo.** Duas opções de mesmo nome são um documento que o artista pode
/// legitimamente ter (dois filhos homónimos), e derivar do rótulo faria as duas responderem ao
/// mesmo clique — o defeito que a chave da ROW aceita de propósito (ver a nota acima) e que aqui
/// **não** é preciso aceitar, porque a posição na lista é um fato que o documento já tem.
///
/// ⚠️ E o prefixo é `authored.opt.`, disjunto de `authored.row.` **por construção**: um rótulo
/// que começasse por `opt.` não pode colidir com uma opção, porque o índice é numérico e o
/// separador vem depois da chave inteira.
///
/// [`WidgetKind::defers_a_popover`]: crate::widget::WidgetKind::defers_a_popover
#[must_use]
pub fn authored_option_id(key: &str, index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("authored.opt.{key}.{index}"))
}
