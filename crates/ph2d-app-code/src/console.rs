//! ⭐ **E10 — o console de UM script**: o que a cena disse (`print`, erros de runtime, recusas de
//! recarga), guardado por `ScriptId` mesmo com a aba fechada, e se ele está aberto.

use ph2d_i18n::tr_with;
use ph2d_panel_code::{ConsoleRow, ConsoleSource, Tone};
use ph2d_script::{CONSOLE_BYTES_MAX, CONSOLE_LINES_MAX, ConsoleKind, ConsoleLine};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    text: String,
    tone: Tone,
    line: Option<u32>,
}

/// As linhas de um script. Os tectos são os do console do motor ([`CONSOLE_LINES_MAX`] linhas e
/// [`CONSOLE_BYTES_MAX`] bytes de texto, medidos; cada linha já chega cortada ao tecto por linha):
/// a mais antiga sai primeiro. Os erros e os bytes contam-se ao entrar e sair — o título e o tecto
/// não percorrem as linhas.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Store {
    entries: VecDeque<Entry>,
    bytes: usize,
    errors: usize,
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
        self.put(text, tone, line);
        if runtime {
            self.open = true;
        }
        runtime
    }

    /// Uma linha do PRÓPRIO editor (não da cena): o formatador que recusou (I13).
    pub fn note(&mut self, text: String, tone: Tone) {
        self.put(text, tone, None);
    }

    fn put(&mut self, text: String, tone: Tone, line: Option<u32>) {
        while !self.entries.is_empty()
            && (self.entries.len() >= CONSOLE_LINES_MAX
                || self.bytes + text.len() > CONSOLE_BYTES_MAX)
        {
            self.pop_front();
        }
        self.bytes += text.len();
        self.errors += usize::from(tone == Tone::Error);
        self.entries.push_back(Entry { text, tone, line });
    }

    fn pop_front(&mut self) {
        if let Some(e) = self.entries.pop_front() {
            self.bytes -= e.text.len();
            self.errors -= usize::from(e.tone == Tone::Error);
        }
    }

    pub fn errors(&self) -> usize {
        self.errors
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn line_of(&self, i: usize) -> Option<u32> {
        self.entries.get(i).and_then(|e| e.line)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        (self.bytes, self.errors) = (0, 0);
    }

    /// O texto de cada linha (os testes leem o que a aba mostra).
    pub fn texts(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.text.clone()).collect()
    }
}

impl ConsoleSource for Store {
    fn len(&self) -> usize {
        self.entries.len()
    }

    fn row(&self, i: usize) -> ConsoleRow<'_> {
        let e = &self.entries[i];
        ConsoleRow {
            text: &e.text,
            tone: e.tone,
            line: e.line,
        }
    }
}
