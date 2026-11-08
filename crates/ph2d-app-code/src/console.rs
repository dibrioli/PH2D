//! ⭐ **E10 — o console de UM script**: o que a cena disse (`print`, erros de runtime, recusas de
//! recarga), guardado por `ScriptId` mesmo com a aba fechada, e a faixa de baixo da aba desse script:
//! aberta ou fechada, e em que separador (Console ou Problems).
//!
//! ⭐ **Onda A (doc 13 §4.1) — um problema, um lugar:** os erros do Play, as recusas da API e as de
//! recarga NÃO são linhas do console: agrupam-se por (espécie, linha, 1.ª linha da mensagem) em
//! [`PlayProblem`] — com os objectos e a contagem — e a lista de Problems mostra-os
//! ([`crate::play_problems`]). O console fica com o que a cena DISSE (`print`), com quem e quando.

use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr_with;
use ph2d_panel_code::{BandTab, ConsoleRow, ConsoleSource, Severity, Tone};
use ph2d_script::{CONSOLE_BYTES_MAX, CONSOLE_LINES_MAX, ConsoleKind, ConsoleLine, ScriptId};
use std::collections::{BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    text: String,
    tone: Tone,
    line: Option<u32>,
    /// «Hero · 0:12» (quem e quando), ou só a hora no topo do script.
    tag: Option<String>,
}

/// De onde vem um problema do Play.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PlayKind {
    Runtime,
    Refused,
    Warning,
}

/// ⭐ **Um problema do Play** — todas as ocorrências da MESMA (espécie, linha, mensagem), com os
/// objectos onde aconteceu (os primeiros [`NAMES_SHOWN`] pelo nome, o resto contado) e quantas vezes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlayProblem {
    pub kind: PlayKind,
    /// 1-based, como o diagnóstico.
    pub line: Option<u32>,
    /// A 1.ª linha da mensagem crua do Luau.
    pub message: String,
    pub names: Vec<String>,
    entities: BTreeSet<u64>,
    pub count: u64,
}

/// Os nomes que a linha mostra; o resto vira «+N more».
pub(crate) const NAMES_SHOWN: usize = 3;

impl PlayProblem {
    pub fn severity(&self) -> Severity {
        match self.kind {
            PlayKind::Warning => Severity::Warning,
            PlayKind::Runtime | PlayKind::Refused => Severity::Error,
        }
    }

    /// Quantos objectos ficaram fora dos nomes mostrados.
    pub fn more(&self) -> usize {
        self.entities.len().saturating_sub(self.names.len())
    }
}

/// O tempo de jogo como a linha o mostra (`m:ss`).
pub(crate) fn game_time(at_us: u64) -> String {
    let s = at_us / 1_000_000;
    let (min, sec) = (s / 60, format!("{:02}", s % 60));
    tr_with("code.console.time", &[("min", &min), ("sec", &sec)])
}

/// As linhas de um script. Os tectos são os do console do motor ([`CONSOLE_LINES_MAX`] linhas e
/// [`CONSOLE_BYTES_MAX`] bytes de texto, medidos; cada linha já chega cortada ao tecto por linha):
/// a mais antiga sai primeiro. Os bytes contam-se ao entrar e sair — o tecto não percorre as linhas.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Store {
    entries: VecDeque<Entry>,
    bytes: usize,
    pub open: bool,
    /// O separador da faixa (por script, como o `open`).
    pub tab: BandTab,
    /// Os problemas do Play deste script, pela ordem de chegada.
    play: Vec<PlayProblem>,
    /// Sobe a cada mudança de `play` (a chave da lista de problemas da aba).
    pub play_generation: u64,
    /// A corrida do Play a que `play` pertence ([`ph2d_script::ScriptHost::play_run`]).
    run: u64,
}

impl Store {
    /// Guarda uma linha da cena; `who` = o nome do objecto que falou (o da Hierarchy). `true` = um
    /// erro de runtime (a faixa abre-se sozinha, nos Problems).
    pub fn push(&mut self, l: &ConsoleLine, who: Option<&str>) -> bool {
        let (kind, d) = match &l.kind {
            ConsoleKind::Print => {
                let time = game_time(l.at_us);
                let tag = match who {
                    Some(w) => tr_with("code.console.tag", &[("who", &w), ("time", &time)]),
                    None => time,
                };
                self.put(l.text.clone(), Tone::Plain, None, Some(tag));
                return false;
            }
            ConsoleKind::RuntimeError(d) => (PlayKind::Runtime, d),
            ConsoleKind::ReloadRefused(d) => (PlayKind::Refused, d),
            ConsoleKind::Warning(d) => (PlayKind::Warning, d),
        };
        let message = d.message.lines().next().unwrap_or_default();
        let entity = l.entity.map(ph2d_ecs::Entity::to_bits);
        let i = match self
            .play
            .iter()
            .position(|g| g.kind == kind && g.line == d.line && g.message == message)
        {
            Some(i) => i,
            None => {
                if self.play.len() >= CONSOLE_LINES_MAX {
                    self.play.remove(0);
                }
                self.play.push(PlayProblem {
                    kind,
                    line: d.line,
                    message: message.to_owned(),
                    names: Vec::new(),
                    entities: BTreeSet::new(),
                    count: 0,
                });
                self.play.len() - 1
            }
        };
        let g = &mut self.play[i];
        g.count += 1;
        if let Some(e) = entity
            && g.entities.insert(e)
            && g.names.len() < NAMES_SHOWN
            && let Some(w) = who
        {
            g.names.push(w.to_owned());
        }
        self.play_generation += 1;
        let runtime = kind == PlayKind::Runtime;
        if runtime {
            self.open = true;
            self.tab = BandTab::Problems;
        }
        runtime
    }

    /// Os problemas do Play guardados.
    pub fn play(&self) -> &[PlayProblem] {
        &self.play
    }

    /// Esquece os problemas do Play (o script recarregou bem, ou a corrida acabou).
    pub fn clear_play(&mut self) {
        if !self.play.is_empty() {
            self.play.clear();
            self.play_generation += 1;
        }
    }

    /// A corrida `run` do Play: outra que a guardada ⇒ os problemas da anterior saem (o Stop/o
    /// rebobinar acabou com ela).
    pub fn follow_run(&mut self, run: u64) {
        if run != self.run {
            self.run = run;
            self.clear_play();
        }
    }

    /// Uma linha do PRÓPRIO editor (não da cena): o formatador que recusou (I13).
    pub fn note(&mut self, text: String, tone: Tone) {
        self.put(text, tone, None, None);
    }

    fn put(&mut self, text: String, tone: Tone, line: Option<u32>, tag: Option<String>) {
        while !self.entries.is_empty()
            && (self.entries.len() >= CONSOLE_LINES_MAX
                || self.bytes + text.len() > CONSOLE_BYTES_MAX)
        {
            self.pop_front();
        }
        self.bytes += text.len() + tag.as_ref().map_or(0, String::len);
        self.entries.push_back(Entry {
            text,
            tone,
            line,
            tag,
        });
    }

    fn pop_front(&mut self) {
        if let Some(e) = self.entries.pop_front() {
            self.bytes -= e.text.len() + e.tag.as_ref().map_or(0, String::len);
        }
    }

    /// O contador da barra e o Ctrl+Shift+M: abre a faixa nos Problems; aberta nos Problems, fecha-a.
    pub fn toggle_problems(&mut self) {
        if self.open && self.tab == BandTab::Problems {
            self.open = false;
        } else {
            self.open = true;
            self.tab = BandTab::Problems;
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn line_of(&self, i: usize) -> Option<u32> {
        self.entries.get(i).and_then(|e| e.line)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }

    /// O texto de cada linha (os testes leem o que a aba mostra).
    pub fn texts(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.text.clone()).collect()
    }

    /// O «quem · quando» de cada linha (os testes leem o que a aba mostra à direita).
    pub fn tags(&self) -> Vec<Option<String>> {
        self.entries.iter().map(|e| e.tag.clone()).collect()
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
            tag: e.tag.as_deref(),
        }
    }
}

impl crate::docs::CodeDocs {
    /// As linhas do console do script `id`, como a aba as mostra.
    #[must_use]
    pub fn console_texts(&self, id: ScriptId) -> Vec<String> {
        self.consoles.get(&id).map(Store::texts).unwrap_or_default()
    }

    /// O «quem · quando» de cada linha do console do script `id`.
    #[must_use]
    pub fn console_tags(&self, id: ScriptId) -> Vec<Option<String>> {
        self.consoles.get(&id).map(Store::tags).unwrap_or_default()
    }

    #[must_use]
    pub fn console_open(&self, id: ScriptId) -> bool {
        self.consoles.get(&id).is_some_and(|s| s.open)
    }

    /// Onde o console pintou a linha `i` no último quadro.
    #[must_use]
    pub fn console_row_rect(&self, id: ScriptId, i: usize) -> Option<Rect> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.console_view.row_rect(i))
    }
}
