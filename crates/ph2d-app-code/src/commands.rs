//! ⭐ **E13 — os comandos da aba nos menus da barra e no `Ctrl+K`**: Save Script no *File*; Find,
//! Replace, Go to Line, Fold All e Unfold All no *Edit*. Só existem com a aba de código activa (o
//! hero pergunta à aba activa em todo quadro). Correr um é o MESMO caminho da tecla.

use crate::doc::CodeDoc;
use crate::docs::SaveKind;
use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::{DocCommand, DocMenu};
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

pub const CMD_SAVE: NodeId = hash_node_id("code_cmd_save");
pub const CMD_FIND: NodeId = hash_node_id("code_cmd_find");
pub const CMD_REPLACE: NodeId = hash_node_id("code_cmd_replace");
pub const CMD_GOTO: NodeId = hash_node_id("code_cmd_goto");
pub const CMD_FOLD_ALL: NodeId = hash_node_id("code_cmd_fold_all");
pub const CMD_UNFOLD_ALL: NodeId = hash_node_id("code_cmd_unfold_all");
/// O assistente de código parado: procura o programa outra vez e volta a ligá-lo (I2).
pub const CMD_RESTART_ASSIST: NodeId = hash_node_id("code_cmd_restart_assistant");

/// `(id, chave do rótulo, menu)` — a tabela única, lida pelos menus, pela paleta e pelo despacho.
const TABLE: [(NodeId, &str, DocMenu); 7] = [
    (CMD_SAVE, "code.cmd.save", DocMenu::File),
    (CMD_FIND, "code.cmd.find", DocMenu::Edit),
    (CMD_REPLACE, "code.cmd.replace", DocMenu::Edit),
    (CMD_GOTO, "code.cmd.goto", DocMenu::Edit),
    (CMD_FOLD_ALL, "code.cmd.fold_all", DocMenu::Edit),
    (CMD_UNFOLD_ALL, "code.cmd.unfold_all", DocMenu::Edit),
    (
        CMD_RESTART_ASSIST,
        "code.cmd.restart_assistant",
        DocMenu::Edit,
    ),
];

/// Os comandos da aba.
#[must_use]
pub fn commands() -> Vec<DocCommand> {
    TABLE
        .iter()
        .map(|&(id, key, menu)| DocCommand {
            id,
            label: tr(key).to_owned(),
            menu,
        })
        .collect()
}

impl CodeDoc {
    /// Corre o comando `cmd`. `true` = era um comando da aba.
    pub fn run(&mut self, cmd: NodeId) -> bool {
        match cmd {
            CMD_SAVE => self.save = Some(SaveKind::Save),
            CMD_FIND => self.find.open_find(&self.buffer, false),
            CMD_REPLACE => self.find.open_find(&self.buffer, true),
            CMD_GOTO => self.find.open_goto(),
            CMD_FOLD_ALL => self.fold_all(true),
            CMD_UNFOLD_ALL => self.fold_all(false),
            _ => return false,
        }
        true
    }
}
