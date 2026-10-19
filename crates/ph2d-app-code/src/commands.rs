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
/// Renomear o nome no cursor (I10, F2), e a lista dos sítios dele (Shift+F12).
pub const CMD_RENAME: NodeId = hash_node_id("code_cmd_rename");
pub const CMD_REFERENCES: NodeId = hash_node_id("code_cmd_references");
/// Mostrar/esconder as dicas de tipo em todas as abas (I9).
pub const CMD_TYPE_HINTS: NodeId = hash_node_id("code_cmd_type_hints");
/// Pôr/tirar o `--!strict` deste script (I14).
pub const CMD_STRICT: NodeId = hash_node_id("code_cmd_strict");
/// Formatar ao salvar ligado/desligado em todas as abas (I13).
pub const CMD_FORMAT_ON_SAVE: NodeId = hash_node_id("code_cmd_format_on_save");

/// O estado que os comandos de ligar/desligar mostram no rótulo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommandState {
    pub type_hints: bool,
    pub strict: bool,
    pub format_on_save: bool,
}

/// `(id, chave do rótulo, menu)` — a tabela única, lida pelos menus, pela paleta e pelo despacho. Um
/// comando de ligar/desligar tem aqui a chave do estado DESLIGADO (a do ligado, em [`commands`]).
const TABLE: [(NodeId, &str, DocMenu); 12] = [
    (CMD_SAVE, "code.cmd.save", DocMenu::File),
    (CMD_FIND, "code.cmd.find", DocMenu::Edit),
    (CMD_REPLACE, "code.cmd.replace", DocMenu::Edit),
    (CMD_GOTO, "code.cmd.goto", DocMenu::Edit),
    (CMD_RENAME, "code.cmd.rename", DocMenu::Edit),
    (CMD_REFERENCES, "code.cmd.references", DocMenu::Edit),
    (CMD_TYPE_HINTS, "code.cmd.type_hints", DocMenu::Edit),
    (CMD_STRICT, "code.cmd.strict", DocMenu::Edit),
    (CMD_FORMAT_ON_SAVE, "code.cmd.format_on_save", DocMenu::Edit),
    (CMD_FOLD_ALL, "code.cmd.fold_all", DocMenu::Edit),
    (CMD_UNFOLD_ALL, "code.cmd.unfold_all", DocMenu::Edit),
    (
        CMD_RESTART_ASSIST,
        "code.cmd.restart_assistant",
        DocMenu::Edit,
    ),
];

/// Os comandos da aba (`state` escolhe o rótulo dos de ligar/desligar).
#[must_use]
pub fn commands(state: CommandState) -> Vec<DocCommand> {
    TABLE
        .iter()
        .map(|&(id, key, menu)| {
            let on = match id {
                CMD_TYPE_HINTS if state.type_hints => Some("code.cmd.type_hints_off"),
                CMD_STRICT if state.strict => Some("code.cmd.strict_off"),
                CMD_FORMAT_ON_SAVE if state.format_on_save => Some("code.cmd.format_on_save_off"),
                _ => None,
            };
            DocCommand {
                id,
                label: tr(on.unwrap_or(key)).to_owned(),
                menu,
            }
        })
        .collect()
}

impl crate::docs::CodeDocs {
    /// Os comandos da aba `id`, com o estado dos de ligar/desligar.
    pub(crate) fn tab_commands(&self, id: u64) -> Vec<DocCommand> {
        let Some(d) = self.docs.iter().find(|d| d.script.0 == id) else {
            return Vec::new();
        };
        commands(CommandState {
            type_hints: self.intel.inlays_on,
            strict: crate::strict::is_strict(d.buffer.text()),
            format_on_save: self.format.on,
        })
    }

    /// Os comandos da FAMÍLIA (valem para todas as abas). `true` = era um deles.
    pub(crate) fn family_command(&mut self, cmd: NodeId) -> bool {
        match cmd {
            CMD_RESTART_ASSIST => self.intel.retry(),
            CMD_FORMAT_ON_SAVE => self.format.on = !self.format.on,
            CMD_TYPE_HINTS => {
                self.intel.inlays_on = !self.intel.inlays_on;
                for d in &mut self.docs {
                    d.inlay.clear();
                }
            }
            _ => return false,
        }
        true
    }
}

impl CodeDoc {
    /// Corre o comando `cmd`. `true` = era um comando da aba.
    pub fn run(&mut self, cmd: NodeId) -> bool {
        match cmd {
            CMD_SAVE => self.save = Some(SaveKind::Save),
            CMD_FIND => self.find.open_find(&self.buffer, false),
            CMD_REPLACE => self.find.open_find(&self.buffer, true),
            CMD_GOTO => self.find.open_goto(),
            CMD_RENAME => self.start_rename(),
            CMD_REFERENCES => self.open_refs(),
            CMD_STRICT => self.toggle_strict(),
            CMD_FOLD_ALL => self.fold_all(true),
            CMD_UNFOLD_ALL => self.fold_all(false),
            _ => return false,
        }
        true
    }
}
