//! A página `debug` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/debug.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "info",
        kind: Function,
        sigs: &[
            "(level: number, options: string): (...any)",
            "(thread: thread, level: number, options: string): (...any)",
            "<A..., R1...>(func: (A...) -> (R1...), options: string): (...any)",
        ],
        params: &["level", "options", "thread", "func"],
        returns: true,
        example: r#"local function jump(height, ...) end
print(debug.info(jump, "na")) --> jump, 1, true
local function whoCalled() return debug.info(2, "n") end
local function attack() return whoCalled() end
print(attack()) --> attack"#,
    },
    Entry {
        name: "traceback",
        kind: Function,
        sigs: &[
            "(message: string?, level: number?): string",
            "(thread: thread, message: string?, level: number?): string",
        ],
        params: &["message", "level", "thread"],
        returns: true,
        example: r#"local trace = debug.traceback("lost the player")
print(string.split(trace, "\n")[1]) --> lost the player"#,
    },
];
