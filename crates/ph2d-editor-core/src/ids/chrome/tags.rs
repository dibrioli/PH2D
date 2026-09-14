//! ⭐⭐⭐ **Os ids do painel TAGS** (TOP-20 #9, W4) — a taxonomia do projecto, docada.
//!
//! ⚠️ **Só os dois que o CHROME lê moram aqui.** O rect exterior é lido pelo passeio de z
//! (`screens/hero/paint.rs`) e pela intercepção da roda (`shells/desktop/src/forwarding.rs`), e o
//! abridor pelas duas tabelas da barra — nenhum dos dois é alcançável de dentro da crate do painel.
//! Os controlos (os verbos, o campo de renomear, a barra de rolagem) vivem em
//! `ph2d_panel_tags::ids`, que é a crate mais baixa que todo leitor deles vê (auditoria A5b).
//!
//! ⚠️ **Bloco APPEND-ONLY**: um id é o hash de uma STRING — reordenar não quebra nada, renomear
//! quebra tudo o que o referencia por nome.

use super::super::hash_node_id;
use ph2d_a11y::NodeId;

/// ⭐⭐⭐ **O PAINEL das tags** — o rect exterior dele.
///
/// ⚠️ **Sem esta entrada em três listas o painel nasce meio-morto**, e cada uma falha de maneira
/// diferente: fora do passeio de z ele é registado, visível e **nunca pintado**; fora do
/// `cursor_over_hero_panel` a roda sobre ele **dá zoom na câmera** por baixo; fora do
/// `canonical_panel_id` o interruptor do menu vaza um `Box::leak` por sessão.
pub const TAGS_PANEL: NodeId = hash_node_id("tags.panel");

/// ⭐⭐⭐ **O abridor do painel de TAGS** (*Window → Tags*).
///
/// ⚠️ **Ele existe porque o painel nasce FECHADO**, e é a única porta dele num projecto que ainda
/// não tem tag nenhuma — que é exactamente onde o artista quer carregar em *+ New* para fazer a
/// primeira. *Uma feature cuja única porta é já ter o que ela produz não tem porta* (a lei que o
/// `TOPBAR_SKELETON` pagou, escrita ao lado dele).
///
/// ⚠️ **É a MESMA visibilidade que o painel usa** (a chave `"tags"`), nunca um segundo bool.
pub const TOPBAR_TAGS: NodeId = hash_node_id("topbar_tags");
