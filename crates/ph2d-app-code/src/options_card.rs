//! ⭐ **O cartão «Script options»** (13 §4 item 12): a cola — cada opção corre o MESMO comando que a
//! paleta `Ctrl+K` lista pelo nome dela; a pintura é do [`ph2d_panel_code::options_card`].

use crate::commands::{
    CMD_FORMAT_ON_SAVE, CMD_RESTART_ASSIST, CMD_STRICT, CMD_TYPE_HINTS, CommandState,
};
use ph2d_panel_code::options_card::{OptionRow, OptionsFrame};
pub use ph2d_panel_code::options_card::{OptionsCard, OptionsPainted};
use ph2d_tool_registry::hash_node_id;

/// As opções do cartão no estado `state`, e o que cada uma faz.
pub(crate) fn rows(state: CommandState) -> [OptionRow<'static>; 3] {
    let row = |cmd, title, says, on| OptionRow {
        cmd,
        title: ph2d_i18n::tr(title),
        says: ph2d_i18n::tr(says),
        on,
    };
    let (strict, says) = ("code.options.strict", "code.options.strict_says");
    let (fmt, fmt_says) = ("code.options.format_on_save", "code.options.format_says");
    let (hints, hints_says) = ("code.options.type_hints", "code.options.hints_says");
    [
        row(CMD_STRICT, strict, says, state.strict),
        row(CMD_FORMAT_ON_SAVE, fmt, fmt_says, state.format_on_save),
        row(CMD_TYPE_HINTS, hints, hints_says, state.type_hints),
    ]
}

pub(crate) fn frame<'a>(
    script: &'a str,
    rows: &'a [OptionRow<'a>],
    assist: Option<ph2d_panel_code::Assist>,
) -> OptionsFrame<'a> {
    OptionsFrame {
        id: hash_node_id("code_options_card"),
        script,
        rows,
        assist,
        restart: CMD_RESTART_ASSIST,
    }
}

impl crate::docs::CodeDocs {
    /// «Format on save» ligado (o estado que a caixa do cartão mostra).
    #[must_use]
    pub fn format_on(&self) -> bool {
        self.format.on
    }
}
