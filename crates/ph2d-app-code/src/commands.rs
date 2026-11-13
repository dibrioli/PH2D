//! ⭐ **Os comandos da aba** — a tabela única do menu PRÓPRIO da aba (13 §4 item 11, D3), da paleta
//! `Ctrl+K` e do despacho ([`DocMenu::Tab`]). Correr um é o MESMO caminho da tecla.

use crate::doc::CodeDoc;
use crate::docs::SaveKind;
use TabMenu::{Edit, File, Find, GoTo, Run, View};
use ph2d_a11y::NodeId;
use ph2d_code_debugcard::CardStyle;
use ph2d_editor_core::doc_surface::{DocCommand, DocMenu};
use ph2d_i18n::tr;
use ph2d_panel_code::code_text::{self, ROWS as TEXT};
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
/// Ctrl+/ comenta/descomenta; Alt+F (UMA linha no menu) dobra/desdobra o bloco do cursor (Onda A, 3).
pub const CMD_TOGGLE_COMMENT: NodeId = hash_node_id("code_cmd_toggle_comment");
pub const CMD_TOGGLE_FOLD: NodeId = hash_node_id("code_cmd_toggle_fold");
/// O assistente de código parado: procura o programa outra vez e volta a ligá-lo (I2).
pub const CMD_RESTART_ASSIST: NodeId = hash_node_id("code_cmd_restart_assistant");
/// Renomear o nome no cursor (I10, F2), e a lista dos sítios dele (Shift+F12).
pub const CMD_RENAME: NodeId = hash_node_id("code_cmd_rename");
pub const CMD_REFERENCES: NodeId = hash_node_id("code_cmd_references");
/// Abre/fecha o cartão «Script options» (item 12).
pub const CMD_OPTIONS: NodeId = hash_node_id("code_cmd_script_options");
/// Caixas do cartão (na paleta pelo nome): as dicas de tipo de todas as abas (I9), o `--!strict` deste
/// script (I14) e formatar ao salvar em todas as abas (I13).
pub const CMD_TYPE_HINTS: NodeId = hash_node_id("code_cmd_type_hints");
pub const CMD_STRICT: NodeId = hash_node_id("code_cmd_strict");
pub const CMD_FORMAT_ON_SAVE: NodeId = hash_node_id("code_cmd_format_on_save");
/// O depurador (Onda D): F9, Continue, F10, Pause on Error, tirar todos os pontos.
pub const CMD_TOGGLE_BREAKPOINT: NodeId = hash_node_id("code_cmd_toggle_breakpoint");
pub const CMD_CONTINUE: NodeId = hash_node_id("code_cmd_continue");
pub const CMD_NEXT_LINE: NodeId = hash_node_id("code_cmd_next_line");
pub const CMD_PAUSE_ON_ERROR: NodeId = hash_node_id("code_cmd_pause_on_error");
pub const CMD_REMOVE_BREAKPOINTS: NodeId = hash_node_id("code_cmd_remove_breakpoints");
/// A Bancada (Onda B; dono 09/11): espiar a cena e o mapa, ligados por omissão.
pub const CMD_PEEK: NodeId = hash_node_id("code_cmd_peek_at_scene");
pub const CMD_MAP: NodeId = hash_node_id("code_cmd_scene_map");
/// A aparência dos quadros acessórios (decisão 13 do dono, plano 14): uma linha por aparência.
pub const CMD_CARD_SHEET: NodeId = hash_node_id("code_cmd_card_style_sheet");
pub const CMD_CARD_CARDS: NodeId = hash_node_id("code_cmd_card_style_cards");
pub const CMD_CARD_STRIP: NodeId = hash_node_id("code_cmd_card_style_strip");

/// Os seis menus da aba (os do Godot, enxutos).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabMenu {
    File,
    Edit,
    Find,
    GoTo,
    View,
    Run,
}

impl TabMenu {
    pub const ALL: [Self; 6] = [File, Edit, Find, GoTo, View, Run];

    #[must_use]
    pub fn title(self) -> &'static str {
        tr(match self {
            Self::File => "code.menu.file",
            Self::Edit => "code.menu.edit",
            Self::Find => "code.menu.find",
            Self::GoTo => "code.menu.go_to",
            Self::View => "code.menu.view",
            Self::Run => "code.menu.run",
        })
    }

    /// A linha do transbordo `…` que abre o submenu deste título quando ele não cabe na barra.
    #[must_use]
    pub fn more_row(self) -> ph2d_a11y::NodeId {
        ph2d_panel_code::tab_bar::menu::more_row(self as usize)
    }
}

/// O estado que os comandos de ligar/desligar mostram no rótulo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommandState {
    pub peek: bool,
    pub map: bool,
    pub type_hints: bool,
    pub strict: bool,
    pub format_on_save: bool,
    pub card_style: CardStyle,
}

/// `(id, chave do rótulo, menu da aba)` — `None` = só na paleta (as caixas do cartão; o estilo do quadro e os modos da letra, que no View são submenus). Um
/// comando de ligar/desligar tem aqui a chave do estado DESLIGADO (a do ligado, em [`commands`]).
const TABLE: [(NodeId, &str, Option<TabMenu>); 38] = [
    (CMD_NEW_SCRIPT, "code.cmd.new_script", Some(File)),
    (CMD_SAVE, "code.cmd.save", Some(File)),
    (CMD_SAVE_AS, "code.cmd.save_as", Some(File)),
    (CMD_CLOSE_TAB, "code.cmd.close_tab", Some(File)),
    (CMD_UNDO, "code.cmd.undo", Some(Edit)),
    (CMD_REDO, "code.cmd.redo", Some(Edit)),
    (CMD_TOGGLE_COMMENT, "code.cmd.toggle_comment", Some(Edit)),
    (CMD_TOGGLE_FOLD, "code.cmd.toggle_fold", Some(Edit)),
    (CMD_FOLD_ALL, "code.cmd.fold_all", Some(Edit)),
    (CMD_UNFOLD_ALL, "code.cmd.unfold_all", Some(Edit)),
    (CMD_RENAME, "code.cmd.rename", Some(Edit)),
    (CMD_OPTIONS, "code.cmd.script_options", Some(Edit)),
    (CMD_FIND, "code.cmd.find", Some(Find)),
    (CMD_REPLACE, "code.cmd.replace", Some(Find)),
    (CMD_REFERENCES, "code.cmd.references", Some(Find)),
    (CMD_GOTO, "code.cmd.goto", Some(GoTo)),
    (
        CMD_GO_TO_DEFINITION,
        "code.cmd.go_to_definition",
        Some(GoTo),
    ),
    (CMD_BACK, "code.cmd.back", Some(GoTo)),
    (CMD_FORWARD, "code.cmd.forward", Some(GoTo)),
    (CMD_NEXT_PROBLEM, "code.cmd.next_problem", Some(GoTo)),
    (CMD_RESTART_ASSIST, "code.cmd.restart_assistant", Some(Run)),
    (
        CMD_TOGGLE_BREAKPOINT,
        "code.cmd.toggle_breakpoint",
        Some(Run),
    ),
    (CMD_CONTINUE, "code.cmd.continue", Some(Run)),
    (CMD_NEXT_LINE, "code.cmd.next_line", Some(Run)),
    (CMD_PAUSE_ON_ERROR, "code.cmd.pause_on_error", Some(Run)),
    (
        CMD_REMOVE_BREAKPOINTS,
        "code.cmd.remove_breakpoints",
        Some(Run),
    ),
    (CMD_PEEK, "code.cmd.peek", Some(View)),
    (CMD_MAP, "code.cmd.map", Some(View)),
    (CMD_CARD_SHEET, "code.cmd.card_sheet", None),
    (CMD_CARD_CARDS, "code.cmd.card_cards", None),
    (CMD_CARD_STRIP, "code.cmd.card_strip", None),
    (TEXT[0].0, TEXT[0].1, None),
    (TEXT[1].0, TEXT[1].1, None),
    (TEXT[2].0, TEXT[2].1, None),
    (TEXT[3].0, TEXT[3].1, None),
    (CMD_STRICT, "code.cmd.strict", None),
    (CMD_FORMAT_ON_SAVE, "code.cmd.format_on_save", None),
    (CMD_TYPE_HINTS, "code.cmd.type_hints", None),
];

/// As linhas do menu `menu` da aba, pela ordem: `(id, rótulo, atalho)` (o atalho sai do rótulo,
/// depois do « · », e pinta-se à direita).
#[must_use]
pub fn menu_rows(menu: TabMenu) -> Vec<(NodeId, String, String)> {
    menu_rows_in(menu, CommandState::default())
}

/// [`menu_rows`] com o estado dos interruptores (o rótulo diz se está ligado).
#[must_use]
pub fn menu_rows_in(menu: TabMenu, state: CommandState) -> Vec<(NodeId, String, String)> {
    TABLE
        .iter()
        .filter(|r| r.2 == Some(menu))
        .map(|&(id, key, _)| {
            let full = tr(shown(id, key, state));
            match full.split_once(" \u{00b7} ") {
                Some((name, keys)) => (id, name.to_owned(), keys.to_owned()),
                None => (id, full.to_owned(), String::new()),
            }
        })
        .collect()
}

/// A linha `id` do menu com o jogo em `stop`: `(vale, mostra o atalho)`. Continue e Next Line só
/// com o jogo parado (Next Line nunca num erro); o F8 é de UM só: Continue parado, Next Problem não.
#[must_use]
pub fn row_state(id: NodeId, stop: crate::debug::GameStop) -> (bool, bool) {
    use crate::debug::GameStop::{Line, Running};
    match id {
        CMD_CONTINUE => (stop != Running, stop != Running),
        CMD_NEXT_LINE => (stop == Line, stop == Line),
        CMD_NEXT_PROBLEM => (true, stop == Running),
        _ => (true, true),
    }
}

/// O rótulo de `id` com o estado dos interruptores: o que diz «On» quando ligado.
fn shown(id: NodeId, key: &'static str, state: CommandState) -> &'static str {
    match id {
        CMD_TYPE_HINTS if state.type_hints => "code.cmd.type_hints_off",
        CMD_STRICT if state.strict => "code.cmd.strict_off",
        CMD_FORMAT_ON_SAVE if state.format_on_save => "code.cmd.format_on_save_off",
        CMD_PEEK if state.peek => "code.cmd.peek_off",
        CMD_MAP if state.map => "code.cmd.map_off",
        CMD_CARD_SHEET if state.card_style == CardStyle::Sheet => "code.cmd.card_sheet_on",
        CMD_CARD_CARDS if state.card_style == CardStyle::Cards => "code.cmd.card_cards_on",
        CMD_CARD_STRIP if state.card_style == CardStyle::Strip => "code.cmd.card_strip_on",
        _ => code_text::label(id).unwrap_or_else(|| stateful(id, key)),
    }
}

/// O rótulo de `id` com o estado que só o host sabe (Pause on Error: o espelho do quadro).
fn stateful(id: NodeId, key: &'static str) -> &'static str {
    let on = crate::debug::PAUSE_ON_ERROR.load(std::sync::atomic::Ordering::Relaxed);
    if id == CMD_PAUSE_ON_ERROR && on {
        "code.cmd.pause_on_error_off"
    } else {
        key
    }
}

/// Os comandos da aba (`state` escolhe o rótulo dos de ligar/desligar).
#[must_use]
pub fn commands(state: CommandState) -> Vec<DocCommand> {
    TABLE
        .iter()
        .map(|&(id, key, _)| DocCommand {
            id,
            label: tr(shown(id, key, state)).to_owned(),
            menu: DocMenu::Tab,
            icon: ph2d_editor_core::icons::IconId::Script,
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
            peek: self.bench.peek_on,
            map: self.bench.map_on,
            type_hints: self.intel.inlays_on,
            strict: crate::strict::is_strict(d.buffer.text()),
            format_on_save: self.format.on,
            card_style: self.card_style,
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
            CMD_PEEK => self.bench.peek_on = !self.bench.peek_on,
            CMD_MAP => self.bench.map_on = !self.bench.map_on,
            CMD_CARD_SHEET => self.card_style = CardStyle::Sheet,
            CMD_CARD_CARDS => self.card_style = CardStyle::Cards,
            CMD_CARD_STRIP => self.card_style = CardStyle::Strip,
            c if code_text::choose(c) => {}
            _ => return self.debug_command(cmd),
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
            CMD_TOGGLE_BREAKPOINT => self.toggle_breakpoint_at_caret(),
            _ => return false,
        }
        true
    }
}
