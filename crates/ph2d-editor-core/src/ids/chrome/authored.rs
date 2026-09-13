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

use crate::ids::hash_node_id;

/// O retângulo externo do painel (z-order + barreira de hit + roteamento da roda).
pub const AUTHORED_PANEL: NodeId = hash_node_id("authored.panel");
