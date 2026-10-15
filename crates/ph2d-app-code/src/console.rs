//! ⭐ **E10 — o console de UM script**: o que a cena disse (`print`, erros de runtime, recusas de
//! recarga), guardado por `ScriptId` mesmo com a aba fechada, e se ele está aberto.

use ph2d_i18n::tr_with;
use ph2d_panel_code::{ConsoleRow, Tone};
use ph2d_script::{CONSOLE_LINES_MAX, ConsoleKind, ConsoleLine};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    text: String,
    tone: Tone,
    line: Option<u32>,
}

/// As linhas de um script. O tecto é o do console do motor ([`CONSOLE_LINES_MAX`], medido): a mais
/// antiga sai primeiro.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Store {
    entries: VecDeque<Entry>,
    pub open: bool,
}

impl Store {
    /// Guarda uma linha da cena. `true` = um erro de runtime (o console abre-se sozinho).
    pub fn push(&mut self, l: &ConsoleLine) -> bool {
        let (text, tone, line, runtime) = match &l.kind {
            ConsoleKind::Print => (l.text.clone(), Tone::Plain, None, false),
            ConsoleKind::RuntimeError(d) => {
                let msg = d.message.lines().next().unwrap_or_default();
                let text = match d.line {
                    Some(n) => {
                        tr_with("code.console.runtime_error", &[("line", &n), ("msg", &msg)])
                    }
                    None => tr_with("code.console.runtime_error_noline", &[("msg", &msg)]),
                };
                (text, Tone::Error, d.line, true)
            }
            ConsoleKind::ReloadRefused(d) => {
                let msg = d.message.lines().next().unwrap_or_default();
                let text = match d.line {
                    Some(n) => tr_with("code.console.refused", &[("line", &n), ("msg", &msg)]),
                    None => tr_with("code.console.refused_noline", &[("msg", &msg)]),
                };
                (text, Tone::Error, d.line, false)
            }
        };
        if self.entries.len() >= CONSOLE_LINES_MAX {
            self.entries.pop_front();
        }
        self.entries.push_back(Entry { text, tone, line });
        if runtime {
            self.open = true;
        }
        runtime
    }

    pub fn rows(&self) -> Vec<ConsoleRow> {
        self.entries
            .iter()
            .map(|e| ConsoleRow {
                text: e.text.clone(),
                tone: e.tone,
                line: e.line,
            })
            .collect()
    }

    pub fn errors(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.tone == Tone::Error)
            .count()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn line_of(&self, i: usize) -> Option<u32> {
        self.entries.get(i).and_then(|e| e.line)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// O texto de cada linha (os testes leem o que a aba mostra).
    pub fn texts(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.text.clone()).collect()
    }
}
