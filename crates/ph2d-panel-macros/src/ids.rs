//! **Os ids do painel MACROS** — os que ESTA crate pinta e despacha. O rect do painel e o abridor do
//! menu vivem no `ph2d_editor_core::ids` (o chrome lê-os); a barra de rolagem é o
//! `ph2d_editor_core::widget::MACROS_SCROLLBAR_ID` (onde os ids de barra se contam).
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// O `×` do cabeçalho.
pub const MACROS_CLOSE: NodeId = hash_node_id("macros.close");
/// **Gravar** — arma a família que o painel diz.
pub const MACROS_RECORD: NodeId = hash_node_id("macros.record");
/// **Parar** — o objecto fica com a `Macro` e o clipe.
pub const MACROS_STOP: NodeId = hash_node_id("macros.stop");
/// **Renomear** a etapa escolhida (o mesmo que o duplo clique na linha).
pub const MACROS_RENAME: NodeId = hash_node_id("macros.rename");
/// **Apagar** a etapa escolhida: o tempo dela fecha-se.
pub const MACROS_DELETE: NodeId = hash_node_id("macros.delete");
/// **O texto do balão** do tutorial «fazer» na etapa escolhida (abre o campo da linha).
pub const MACROS_HINT: NodeId = hash_node_id("macros.hint");
/// O campo de renomear, sobreposto à linha.
pub const MACROS_RENAME_INPUT: NodeId = hash_node_id("macros.rename_input");
/// A espera antes da etapa, em segundos.
pub const MACROS_WAIT: NodeId = hash_node_id("macros.wait");
/// O tamanho do pincel da pincelada, em px.
pub const MACROS_SIZE: NodeId = hash_node_id("macros.size");
/// O valor final de uma sessão de param.
pub const MACROS_VALUE: NodeId = hash_node_id("macros.value");
/// A amostra da cor da pincelada: abre o selector partilhado.
pub const MACROS_COLOR: NodeId = hash_node_id("macros.color");

/// **O id da LINHA de uma etapa** — derivado do endereço dela (trilha, etapa): a população é a
/// gravação, sem tecto.
#[must_use]
pub fn row_id(trilha: usize, k: usize) -> NodeId {
    hash_node_id_runtime(&format!("macros.row.{trilha}.{k}"))
}

// ─── A biblioteca do utilizador (W18, `docs/Macros/16_W18_plano.md` §2.6, §4) ─────────────────────

/// **Guardar na biblioteca** a gravação do objecto mostrado.
pub const MACROS_LIB_SAVE: NodeId = hash_node_id("macros.lib.save");
/// **Guardar como asset** (receita do projecto) o objecto mostrado.
pub const MACROS_LIB_SAVE_ASSET: NodeId = hash_node_id("macros.lib.save_asset");
/// **Importar…** um `.ph2dmacro` para a biblioteca.
pub const MACROS_LIB_IMPORT: NodeId = hash_node_id("macros.lib.import");
/// **Criar novo** com a macro escolhida.
pub const MACROS_LIB_APPLY_NEW: NodeId = hash_node_id("macros.lib.apply_new");
/// **Aplicar por cima** do objecto escolhido.
pub const MACROS_LIB_APPLY_OVER: NodeId = hash_node_id("macros.lib.apply_over");
/// **Exportar…** a macro escolhida.
pub const MACROS_LIB_EXPORT: NodeId = hash_node_id("macros.lib.export");
/// **Apagar** a macro escolhida da biblioteca.
pub const MACROS_LIB_DELETE: NodeId = hash_node_id("macros.lib.delete");
/// O campo de renomear uma macro da biblioteca, sobreposto à linha dela.
pub const MACROS_LIB_RENAME_INPUT: NodeId = hash_node_id("macros.lib.rename_input");

/// **O id da LINHA de uma macro da biblioteca** — derivado do NOME (o nome é o ficheiro: único).
#[must_use]
pub fn lib_row_id(nome: &str) -> NodeId {
    hash_node_id_runtime(&format!("macros.lib.row.{nome}"))
}
