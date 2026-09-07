//! **Os ids da BARRA DO MODO de edição de receita** (o *Edit Prefab*, 2026-09-07).
//!
//! ⚠️ **Ela é chrome do CANVAS, não de um painel** — vive por cima da área de desenho enquanto uma
//! receita está aberta, e desaparece com ela. É por isso que o id não mora com os do painel vetorial
//! (`vector_components.rs`): aqueles são de uma secção que existe sempre; este é de uma superfície
//! que só existe dentro de um modo.

use ph2d_a11y::NodeId;

use super::hash_node_id;

/// **O botão que FECHA o modo.**
///
/// ⚠️ Ele é a razão de a barra existir: até 2026-09-07 a única saída do modo era **adivinhar** que
/// clicar no vazio desfaz a selecção. *Um modo que se entra por um verbo e se sai por acidente é um
/// modo sem saída.*
pub const PREFAB_EDIT_DONE: NodeId = hash_node_id("prefab_edit_done");

/// **O botão que CANCELA as modificações** e sai (Enio, 2026-09-07).
///
/// ⚠️ **Ele é o irmão do [`PREFAB_EDIT_DONE`], não uma variante dele:** *sair* e *sair desfazendo*
/// são dois fins diferentes, e um só botão com modificador esconderia o segundo. A tecla é o `Esc`,
/// como em toda a casa.
pub const PREFAB_EDIT_CANCEL: NodeId = hash_node_id("prefab_edit_cancel");
