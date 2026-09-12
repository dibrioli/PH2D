//! **As SETAS do Morph** — os `NodeId` da seção *Morph States* (plano 32 W4/W7/W8).
//!
//! ⛔⛔ **SEÇÃO PRÓPRIA, e a lição é de produto** (Enio, 2026-08-25: *"os states de morph deveriam
//! ter sessão exclusiva"*). A W4 pendurou estas linhas dentro da seção **States** — a das poses de
//! UI e do Smart Animate — com o argumento de que *"o Inspector mostra o que o objecto TEM"* e de
//! que *"um objecto raramente é as duas coisas"*. O argumento é verdadeiro e **não era a pergunta**:
//! partilhar a seção fez o cabeçalho de uma feature **já entregue** passar a aparecer por causa de
//! outra, e o dono leu isso como contaminação — que é exactamente o que era.
//!
//! ⚠️ *A lei do ADR-0166 diz o que MOSTRAR, nunca ONDE.* Duas features com donos diferentes,
//! histórias diferentes e gates diferentes partilhando um cabeçalho é uma porta a mais na seção de
//! quem chegou primeiro.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_morph.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// **O cabeçalho da seção MORPH STATES** — a máquina de estados do Morph selecionado.
///
/// Vizinha da [`super::vector::VECTOR_SECTION_MORPH`] e **abaixo dela**, porque a ordem é o
/// assunto: a seção Morph diz o que o objecto **é**, esta diz **como ele decide** em que forma
/// está.
pub const VECTOR_SECTION_MORPH_STATES: NodeId = hash_node_id("vector.section.morph_states");

/// **O botão que faz o conjunto** — escolhe-se N formas no canvas e ele cria o objecto que as
/// governa, com **todas** as transições possíveis já ligadas (plano 32 W8).
pub const VECTOR_MORPH_STATES_MAKE: NodeId = hash_node_id("vector.morph.states.make");

/// **Quantas acções o menu da condição oferece.** É o pool de ids do popover.
///
/// ⚠️ Um mapa com mais acções continua a funcionar; o menu mostra as primeiras. O número acompanha
/// o que um projecto real tem (o mapa de fábrica traz **seis**).
pub const MAX_MORPH_ACTIONS: usize = 24;

/// ⭐⭐ **DESFAZER TUDO** — dissolve o conjunto: o objecto some, as formas voltam **soltas e
/// visíveis**, onde estavam.
///
/// ⚠️ **Ele é o inverso EXACTO do [`VECTOR_MORPH_STATES_MAKE`]**, e desde a W11 não precisa de
/// código próprio de desmontagem: a lista são os filhos, então dissolver é reparentar todos para
/// fora. *Um botão que desfaz tem de chegar ao mesmo mundo de onde se partiu, e não a um parecido.*
pub const VECTOR_MORPH_DISSOLVE: NodeId = hash_node_id("vector.morph.dissolve");

/// **PLAY na forma `row`** — vai até ela, animado, para o artista a VER.
///
/// ⚠️ **Era o `Show` das poses de UI, e o Enio renomeou-o** (2026-08-26): ali um estado é uma pose
/// que se **aplica**; aqui é uma forma para onde se **viaja**, e a viagem é o produto.
///
/// ⚠️ **Ele LIGA a pré-visualização se estiver desligada** — a máquina só anda dentro do modo (é
/// ele que tem o relógio), e um Play que não tocasse nada seria um botão morto com nome de verbo.
#[must_use]
pub fn morph_shape_play_id(row: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.morph.shape.play.{row}"))
}

/// **DESCONECTAR a forma `row`** — ela sai do conjunto e volta a ser uma forma solta e visível.
///
/// ⚠️ **Era o `Clear`, e o Enio renomeou-o** (2026-08-26) — com razão: `Clear` sugere *apagar*, e
/// aqui não se apaga nada. A forma continua no documento, com o desenho dela; ela só deixa de ser
/// um estado. ⭐ **E a tecla dela FICA guardada**: voltar a arrastá-la para dentro devolve-a.
///
/// ⚠️ Desde a W11 isto é **exactamente** o gesto de arrastar para fora na Hierarquia — o botão é o
/// atalho, nunca uma segunda lei.
#[must_use]
pub fn morph_shape_disconnect_id(row: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.morph.shape.disconnect.{row}"))
}

/// **O botão que ABRE o modal dos eventos** para a forma `row`.
///
/// ⚠️ **Era um dropdown, e o Enio trocou-o** (2026-08-26: *"no lugar do dropdown melhor um botão
/// que abre um modal com os eventos"*). O menu vivia dentro do scroll da seção e precisava de um
/// passe diferido só para não ser cortado na borda; um modal não tem esse problema, e mostra a
/// lista inteira com espaço para o nome de cada acção.
#[must_use]
pub fn morph_shape_key_button_id(row: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.morph.shape.keybtn.{row}"))
}

/// A opção `action` no menu da condição da seta `row`.
///
/// ⚠️ **O índice `0` é o «—»** (sem condição): ele existe porque *tirar* a condição tem de ser um
/// gesto — e, desde a W8, é **a única maneira de desligar uma transição**. O grafo é completo por
/// construção, então uma seta sem acção é uma passagem que existe e **nunca acontece**.
#[must_use]
pub fn morph_shape_key_option_id(row: usize, action: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.morph.arrow.when.{row}.{action}"))
}
