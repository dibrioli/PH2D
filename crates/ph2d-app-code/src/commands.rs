//! ⭐ **Os comandos da aba** — a tabela única lida pelo menu PRÓPRIO da aba (`File · Edit · Find ·
//! Go To · Run`, 13 §4 item 11, decisão D3 do dono), pela paleta `Ctrl+K` e pelo despacho. Os menus
//! do app já não mudam com a aba ([`DocMenu::Tab`]). Correr um é o MESMO caminho da tecla.

use crate::doc::CodeDoc;
use crate::docs::SaveKind;
use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::{DocCommand, DocMenu};
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

pub const CMD_SAVE: NodeId = hash_node_id("code_cmd_save");
pub const CMD_SAVE_AS: NodeId = hash_node_id("code_cmd_save_as");
pub const CMD_NEW_SCRIPT: NodeId = hash_node_id("code_cmd_new_script");
pub const CMD_CLOSE_TAB: NodeId = hash_node_id("code_cmd_close_tab");
pub const CMD_UNDO: NodeId = hash_node_id("code_cmd_undo");
pub const CMD_REDO: NodeId = hash_node_id("code_cmd_redo");
pub const CMD_FIND: NodeId = hash_node_id("code_cmd_find");
pub const CMD_REPLACE: NodeId = hash_node_id("code_cmd_replace");
pub const CMD_GOTO: NodeId = hash_node_id("code_cmd_goto");
pub const CMD_GO_TO_DEFINITION: NodeId = hash_node_id("code_cmd_go_to_definition");
pub const CMD_BACK: NodeId = hash_node_id("code_cmd_back");
pub const CMD_FORWARD: NodeId = hash_node_id("code_cmd_forward");
pub const CMD_NEXT_PROBLEM: NodeId = hash_node_id("code_cmd_next_problem");
pub const CMD_FOLD_ALL: NodeId = hash_node_id("code_cmd_fold_all");
pub const CMD_UNFOLD_ALL: NodeId = hash_node_id("code_cmd_unfold_all");
/// Ctrl+/ comenta/descomenta; Alt+F dobra/desdobra o bloco do cursor (Onda A, item 3).
pub const CMD_TOGGLE_COMMENT: NodeId = hash_node_id("code_cmd_toggle_comment");
/// Alt+F no menu: UMA linha, a mesma troca da tecla (dobra o bloco do cursor ou abre o fechado).
pub const CMD_TOGGLE_FOLD: NodeId = hash_node_id("code_cmd_toggle_fold");
/// O assistente de código parado: procura o programa outra vez e volta a ligá-lo (I2).
pub const CMD_RESTART_ASSIST: NodeId = hash_node_id("code_cmd_restart_assistant");
/// Renomear o nome no cursor (I10, F2), e a lista dos sítios dele (Shift+F12).
pub const CMD_RENAME: NodeId = hash_node_id("code_cmd_rename");
pub const CMD_REFERENCES: NodeId = hash_node_id("code_cmd_references");
/// Abre/fecha o cartão «Script options» (item 12).
pub const CMD_OPTIONS: NodeId = hash_node_id("code_cmd_script_options");
/// Mostrar/esconder as dicas de tipo em todas as abas (I9). Caixa do cartão; na paleta pelo nome.
pub const CMD_TYPE_HINTS: NodeId = hash_node_id("code_cmd_type_hints");
/// Pôr/tirar o `--!strict` deste script (I14). Caixa do cartão; na paleta pelo nome.
pub const CMD_STRICT: NodeId = hash_node_id("code_cmd_strict");
/// Formatar ao salvar ligado/desligado em todas as abas (I13). Caixa do cartão; na paleta pelo nome.
pub const CMD_FORMAT_ON_SAVE: NodeId = hash_node_id("code_cmd_format_on_save");

/// Os cinco menus da aba (os do Godot, enxutos).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabMenu {
    File,
    Edit,
    Find,
    GoTo,
    Run,
}

impl TabMenu {
    pub const ALL: [Self; 5] = [Self::File, Self::Edit, Self::Find, Self::GoTo, Self::Run];

    #[must_use]
    pub fn title(self) -> &'static str {
        tr(match self {
            Self::File => "code.menu.file",
            Self::Edit => "code.menu.edit",
            Self::Find => "code.menu.find",
            Self::GoTo => "code.menu.go_to",
            Self::Run => "code.menu.run",
        })
    }
}

/// O estado que os comandos de ligar/desligar mostram no rótulo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommandState {
    pub type_hints: bool,
    pub strict: bool,
    pub format_on_save: bool,
}

/// `(id, chave do rótulo, menu da aba)` — `None` = só na paleta (as três caixas do cartão). Um
/// comando de ligar/desligar tem aqui a chave do estado DESLIGADO (a do ligado, em [`commands`]).
const TABLE: [(NodeId, &str, Option<TabMenu>); 24] = [
    (CMD_NEW_SCRIPT, "code.cmd.new_script", Some(TabMenu::File)),
    (CMD_SAVE, "code.cmd.save", Some(TabMenu::File)),
    (CMD_SAVE_AS, "code.cmd.save_as", Some(TabMenu::File)),
    (CMD_CLOSE_TAB, "code.cmd.close_tab", Some(TabMenu::File)),
    (CMD_UNDO, "code.cmd.undo", Some(TabMenu::Edit)),
    (CMD_REDO, "code.cmd.redo", Some(TabMenu::Edit)),
    (
        CMD_TOGGLE_COMMENT,
        "code.cmd.toggle_comment",
        Some(TabMenu::Edit),
    ),
    (CMD_TOGGLE_FOLD, "code.cmd.toggle_fold", Some(TabMenu::Edit)),
    (CMD_FOLD_ALL, "code.cmd.fold_all", Some(TabMenu::Edit)),
    (CMD_UNFOLD_ALL, "code.cmd.unfold_all", Some(TabMenu::Edit)),
    (CMD_RENAME, "code.cmd.rename", Some(TabMenu::Edit)),
    (CMD_OPTIONS, "code.cmd.script_options", Some(TabMenu::Edit)),
    (CMD_FIND, "code.cmd.find", Some(TabMenu::Find)),
    (CMD_REPLACE, "code.cmd.replace", Some(TabMenu::Find)),
    (CMD_REFERENCES, "code.cmd.references", Some(TabMenu::Find)),
    (CMD_GOTO, "code.cmd.goto", Some(TabMenu::GoTo)),
    (
        CMD_GO_TO_DEFINITION,
        "code.cmd.go_to_definition",
        Some(TabMenu::GoTo),
    ),
    (CMD_BACK, "code.cmd.back", Some(TabMenu::GoTo)),
    (CMD_FORWARD, "code.cmd.forward", Some(TabMenu::GoTo)),
    (
        CMD_NEXT_PROBLEM,
        "code.cmd.next_problem",
        Some(TabMenu::GoTo),
    ),
    (
        CMD_RESTART_ASSIST,
        "code.cmd.restart_assistant",
        Some(TabMenu::Run),
    ),
    (CMD_STRICT, "code.cmd.strict", None),
    (CMD_FORMAT_ON_SAVE, "code.cmd.format_on_save", None),
    (CMD_TYPE_HINTS, "code.cmd.type_hints", None),
];

/// As linhas do menu `menu` da aba, pela ordem: `(id, rótulo, atalho)` (o atalho sai do rótulo,
/// depois do « · », e pinta-se à direita).
#[must_use]
pub fn menu_rows(menu: TabMenu) -> Vec<(NodeId, String, String)> {
    TABLE
        .iter()
        .filter(|r| r.2 == Some(menu))
        .map(|&(id, key, _)| {
            let full = tr(key);
            match full.split_once(" \u{00b7} ") {
                Some((name, keys)) => (id, name.to_owned(), keys.to_owned()),
                None => (id, full.to_owned(), String::new()),
            }
        })
        .collect()
}

/// Os comandos da aba (`state` escolhe o rótulo dos de ligar/desligar).
#[must_use]
pub fn commands(state: CommandState) -> Vec<DocCommand> {
    TABLE
        .iter()
        .map(|&(id, key, _)| {
            let on = match id {
                CMD_TYPE_HINTS if state.type_hints => Some("code.cmd.type_hints_off"),
                CMD_STRICT if state.strict => Some("code.cmd.strict_off"),
                CMD_FORMAT_ON_SAVE if state.format_on_save => Some("code.cmd.format_on_save_off"),
                _ => None,
            };
            DocCommand {
                id,
                label: tr(on.unwrap_or(key)).to_owned(),
                menu: DocMenu::Tab,
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
        commands(self.command_state(d))
    }

    pub(crate) fn command_state(&self, d: &CodeDoc) -> CommandState {
        CommandState {
            type_hints: self.intel.inlays_on,
            strict: crate::strict::is_strict(d.buffer.text()),
            format_on_save: self.format.on,
        }
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
            CMD_NEW_SCRIPT => self.list.asks.push(crate::list::Ask::New),
            CMD_OPTIONS => self.options.toggle(),
            _ => return false,
        }
        true
    }

    /// Os comandos que precisam da aba E da família (fechar, o histórico, o undo). `true` = era um.
    pub(crate) fn tab_family_command(&mut self, id: u64, cmd: NodeId) -> bool {
        use ph2d_editor_core::doc_surface::DocumentSurface;
        match cmd {
            CMD_CLOSE_TAB => self.close_active(id),
            CMD_BACK | CMD_FORWARD => {
                let _ = self.nav_step(id, cmd == CMD_BACK);
            }
            CMD_UNDO | CMD_REDO => {
                let _ = self.undo(id, cmd == CMD_REDO);
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
            CMD_SAVE_AS => self.save = Some(SaveKind::SaveAs),
            CMD_FIND => self.find.open_find(&self.buffer, false),
            CMD_REPLACE => self.find.open_find(&self.buffer, true),
            CMD_GOTO => self.find.open_goto(),
            CMD_GO_TO_DEFINITION => self.go_to_definition(self.buffer.selection().primary().head),
            CMD_NEXT_PROBLEM => self.problem_step(true),
            CMD_RENAME => self.start_rename(),
            CMD_REFERENCES => self.open_refs(),
            CMD_STRICT => self.toggle_strict(),
            CMD_TOGGLE_COMMENT => self.toggle_comment(),
            CMD_TOGGLE_FOLD => self.toggle_fold_at_cursor(),
            CMD_FOLD_ALL => self.fold_all(true),
            CMD_UNFOLD_ALL => self.fold_all(false),
            _ => return false,
        }
        true
    }
}
