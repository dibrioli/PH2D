//! ⭐ **I8 — o balão dos PROBLEMAS** (`docs/CodeEditor/03_onda2.md`).
//!
//! - O rato parado [`HOVER_PAUSE_NS`] no **ponto da calha** mostra a explicação escrita à mão de cada
//!   problema da linha (`ph2d-code-explain`): o título marcado e o porquê. Não pergunta ao assistente.
//! - Parado sobre o **culpado sublinhado**, mostra a mensagem CRUA (a do Luau), por cima do que o
//!   assistente disser da palavra.
//! - **O registo local** (sem rede): uma linha `<segundos>\t<classe>` por explicação que um balão
//!   abriu, para reordenar o trabalho nas explicações depois.

use crate::doc::CodeDoc;
use crate::docs::CodeDocs;
use crate::hint::Note;
use ph2d_panel_code::{Explanation, HoverSection, Problem};
use std::io::Write as _;
use std::ops::Range;
use std::path::PathBuf;

#[cfg(doc)]
use crate::hint::HOVER_PAUSE_NS;

/// A explicação de uma mensagem crua, se a classe tem uma.
pub(crate) fn explanation(message: &str) -> Option<Explanation> {
    ph2d_code_explain::explain(message).map(|e| Explanation {
        kind: e.kind.id(),
        title: e.title(),
        why: e.why(),
    })
}

/// O ficheiro do registo por omissão: `PH2D_CODE_SEEN_FILE` · `$XDG_DATA_HOME/ph2d/…` ·
/// `$HOME/.local/share/ph2d/…` · `%APPDATA%\ph2d\…`.
fn default_file() -> Option<PathBuf> {
    const NAME: &str = "code-explanations-seen.tsv";
    let var = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    var("PH2D_CODE_SEEN_FILE")
        .or_else(|| var("XDG_DATA_HOME").map(|d| d.join("ph2d").join(NAME)))
        .or_else(|| var("HOME").map(|h| h.join(".local/share/ph2d").join(NAME)))
        .or_else(|| var("APPDATA").map(|d| d.join("ph2d").join(NAME)))
}

/// As explicações vistas nesta corrida, e onde se gravam.
#[derive(Debug)]
pub(crate) struct Seen {
    /// `None` = só na memória (os testes).
    file: Option<PathBuf>,
    pub kinds: Vec<&'static str>,
    /// A primeira escrita que falhou já foi dita no terminal.
    told: bool,
}

impl Default for Seen {
    fn default() -> Self {
        Self {
            file: default_file(),
            kinds: Vec::new(),
            told: false,
        }
    }
}

impl Seen {
    fn record(&mut self, kinds: &[&'static str]) {
        self.kinds.extend_from_slice(kinds);
        let Some(file) = self.file.as_ref() else {
            return;
        };
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let lines: String = kinds.iter().map(|k| format!("{secs}\t{k}\n")).collect();
        let wrote = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file)?
                    .write_all(lines.as_bytes())
            });
        if let Err(e) = wrote
            && !self.told
        {
            self.told = true;
            eprintln!("code explanations: could not write {}: {e}", file.display());
        }
    }
}

impl CodeDoc {
    /// O rato na calha da linha `line`: o balão dela, se a linha tem problemas.
    pub(crate) fn gutter_note(&mut self, line: usize) -> Option<(Range<usize>, Note)> {
        self.problems().iter().any(|p| p.line == line).then(|| {
            let at = self.buffer.text().byte_of_line(line);
            (at..at, Note::Gutter(line))
        })
    }

    /// O culpado sublinhado sob o rato em `(x, y)` (byte `b`): o desenho dele, nunca o vazio à volta.
    pub(crate) fn culprit_at(&mut self, b: usize, x: f32, y: f32) -> Option<Range<usize>> {
        self.problems()
            .iter()
            .filter_map(|p| p.culprit.clone())
            .find(|c| c.start <= b && b <= c.end && c.start < c.end && self.drawn_over(c, x, y))
    }

    /// `(x, y)` cai no desenho dos bytes `r` (o mesmo critério da palavra do balão).
    pub(crate) fn drawn_over(&self, r: &Range<usize>, x: f32, y: f32) -> bool {
        let (a, z) = (
            self.view.caret_rect(&self.buffer, r.start),
            self.view.caret_rect(&self.buffer, r.end),
        );
        a.zip(z)
            .is_some_and(|(a, z)| a.x <= x && x <= z.x && a.y <= y && y < a.y + a.h)
    }

    /// As secções do balão dos problemas, já com a pausa cumprida: `(título, texto)`.
    pub(crate) fn note_sections(&self, problems: &[Problem]) -> Vec<(Option<String>, String)> {
        let Some(h) = self.hints.hover.as_ref().filter(|h| h.ripe) else {
            return Vec::new();
        };
        match &h.note {
            Some(Note::Gutter(line)) => problems
                .iter()
                .filter(|p| p.line == *line)
                .map(|p| match &p.explanation {
                    Some(e) => (Some(e.title.clone()), e.why.clone()),
                    None => (Some(p.message.clone()), String::new()),
                })
                .collect(),
            Some(Note::Culprit) => problems
                .iter()
                .filter(|p| p.culprit.as_ref() == Some(&h.word))
                .map(|p| (None, p.message.clone()))
                .collect(),
            None => Vec::new(),
        }
    }
}

/// A pista do botão `i` do balão da aba `serial` no relógio de animação (um espaço de ids só dela).
fn fix_track(serial: u64, i: usize) -> ph2d_a11y::NodeId {
    const FIX_TRACKS: u64 = 0xC0DE_F1C5 << 32;
    ph2d_a11y::NodeId(FIX_TRACKS | (serial << 8) | i as u64)
}

/// As secções do balão: as dos problemas primeiro, depois o que o assistente diz da palavra.
pub(crate) fn sections<'a>(
    note: &'a [(Option<String>, String)],
    assistant: Option<&'a str>,
) -> Vec<HoverSection<'a>> {
    note.iter()
        .map(|(title, text)| HoverSection {
            title: title.as_deref(),
            text,
        })
        .chain(assistant.map(|text| HoverSection { title: None, text }))
        .collect()
}

impl CodeDocs {
    /// Um quadro: o balão dos problemas cuja pausa passou abre, e as explicações dele contam como vistas.
    pub(crate) fn notes(&mut self) {
        let now = self.clock.0;
        for i in 0..self.docs.len() {
            let d = &mut self.docs[i];
            d.hints.settle(now);
            let Some(h) = d.hints.hover.as_mut() else {
                continue;
            };
            if h.ripe || h.note.is_none() || h.due_ns > now {
                continue;
            }
            h.ripe = true;
            let note = h.note;
            let word = h.word.clone();
            let heads: Vec<usize> = match note {
                Some(Note::Gutter(line)) => d
                    .problems()
                    .iter()
                    .filter(|p| p.line == line)
                    .filter_map(|p| p.culprit.as_ref().map(|c| c.start))
                    .collect(),
                _ => vec![word.start],
            };
            let fixes = d.actions_at(&heads);
            if let Some(h) = d.hints.hover.as_mut() {
                h.fixes = Some(fixes);
            }
            let Some(Note::Gutter(line)) = note else {
                continue;
            };
            let kinds: Vec<&'static str> = d
                .problems()
                .iter()
                .filter(|p| p.line == line)
                .filter_map(|p| p.explanation.as_ref().map(|e| e.kind))
                .collect();
            self.seen.record(&kinds);
        }
    }

    /// Um quadro: o realce de cada botão de correcção anda no relógio de animação do app (`motion`,
    /// o mesmo dos botões dos painéis e com as mesmas preferências), aceso sob o rato.
    pub fn fix_hover(&mut self, motion: &mut ph2d_editor_core::motion::UiMotion) {
        for d in &mut self.docs {
            let Some(h) = d.hints.hover.as_mut() else {
                continue;
            };
            let n = h
                .fixes
                .as_ref()
                .filter(|f| f.ready())
                .map_or(0, |f| f.list().count());
            h.live = (0..n)
                .map(|i| {
                    let on = if h.hot == Some(i) { 1.0 } else { 0.0 };
                    motion.animate(
                        fix_track(d.serial, i),
                        on,
                        ph2d_editor_core::motion::Role::Fade,
                    )
                })
                .collect();
        }
    }

    /// Onde se grava o registo das explicações vistas (`None` = só na memória).
    pub fn set_seen_file(&mut self, file: Option<PathBuf>) {
        self.seen.file = file;
    }

    /// ⚠️ SONDA: as classes das explicações que os balões abriram nesta corrida.
    #[must_use]
    pub fn seen_kinds(&self) -> &[&'static str] {
        &self.seen.kinds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_registo_acrescenta_uma_linha_por_explicacao_e_cria_a_pasta() {
        let dir = std::env::temp_dir().join(format!("ph2d-seen-{}", std::process::id()));
        let file = dir.join("sub").join("seen.tsv");
        let mut s = Seen {
            file: Some(file.clone()),
            kinds: Vec::new(),
            told: false,
        };
        s.record(&["unknown_global"]);
        s.record(&["unclosed", "same_line"]);
        let text = std::fs::read_to_string(&file).expect("gravado");
        let kinds: Vec<&str> = text
            .lines()
            .map(|l| l.split('\t').nth(1).expect("<segundos>\t<classe>"))
            .collect();
        assert_eq!(kinds, ["unknown_global", "unclosed", "same_line"]);
        assert_eq!(s.kinds, kinds);
        let _ = std::fs::remove_dir_all(dir);
    }
}
