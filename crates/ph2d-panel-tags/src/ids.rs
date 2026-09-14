//! **Os ids do painel TAGS** — os que ESTA crate pinta e despacha.
//!
//! ⚠️ O rect do painel (`TAGS_PANEL`) e o abridor do menu (`TOPBAR_TAGS`) **não** moram aqui: eles
//! são lidos pelo chrome (o passeio de z, a roda, as duas tabelas da barra), que não vê esta crate
//! ⇒ vivem no `ph2d_editor_core::ids` (auditoria A5b: *o dono de um id é a crate mais BAIXA que
//! todo leitor dele vê*).
//!
//! ⚠️ **Bloco APPEND-ONLY**: um id é o hash de uma STRING — reordenar não quebra nada, renomear
//! quebra tudo o que o referencia por nome.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// O `×` do cabeçalho do painel.
pub const TAGS_CLOSE: NodeId = hash_node_id("tags.close");

/// **`+ New`** — cria uma tag de RAIZ e põe a linha em modo de renomear.
pub const TAGS_NEW: NodeId = hash_node_id("tags.new");

/// **`+ Child`** — cria uma tag debaixo da que está em mãos.
pub const TAGS_CHILD: NodeId = hash_node_id("tags.child");

/// **`Rename`** — o mesmo que o duplo clique na linha, para quem procura um botão.
pub const TAGS_RENAME: NodeId = hash_node_id("tags.rename");

/// **`Delete`** — apaga a tag em mãos, a subárvore dela e a pertença, num passo de undo.
///
/// ⚠️ O RÓTULO dele diz o que o gesto leva (*«Delete (3 tags, 5 objects)»*) — é aí que vive o
/// *«remove from N objects»* do plano. *Um verbo destrutivo que não diz o tamanho do estrago pede
/// uma caixa de confirmação; um que diz, não precisa.*
pub const TAGS_DELETE: NodeId = hash_node_id("tags.delete");

/// **`Select`** — escolhe na cena todos os objectos que pertencem à tag em mãos (com a subárvore).
pub const TAGS_SELECT: NodeId = hash_node_id("tags.select");

/// **`Move to root`** — tira a tag em mãos de debaixo do pai dela.
///
/// ⚠️ Ele existe porque o arrasto **não sabe apontar para o nada**: largar uma linha «fora de todas
/// as outras» é uma área que muda de tamanho com a lista, e num painel cheio ela não existe.
pub const TAGS_UNPARENT: NodeId = hash_node_id("tags.unparent");

/// O campo de renomear, sobreposto à linha.
pub const TAGS_RENAME_INPUT: NodeId = hash_node_id("tags.rename_input");

// ⛔⛔ **A barra de rolagem NÃO tem id aqui, e a ausência é a decisão.** Ela é
// `ph2d_editor_core::widget::TAGS_SCROLLBAR_ID`, porque o dono de um id de barra é o DESPACHO: o
// `scrollbar_panel_for_id` tem de o mapear ao painel, senão o polegar pinta e não se agarra. A 1.ª
// redacção declarou-o aqui e o `hit_indexed_ids_are_registered` apanhou-o.

/// **O id da LINHA de uma tag** — derivado da identidade dela.
///
/// ⚠️ **Derivado e não uma tabela de constantes**, pela mesma razão das linhas da Hierarquia: a
/// população é o documento, e um array fixo seria um tecto sobre a árvore — que **não tem tecto**
/// (medido: `9 344` tags abrem em `9,2 ms`).
///
/// ⚠️ **Pela porta [`hash_node_id_runtime`]**, nunca por uma FNV escrita à mão: a terceira cópia
/// dela neste repo teve o primo errado e os ids caíam noutro espaço, onde o hit-test nunca os
/// resolve.
#[must_use]
pub fn row_id(tag: u64) -> NodeId {
    hash_node_id_runtime(&format!("tags.row.{tag}"))
}
