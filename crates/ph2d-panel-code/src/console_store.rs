//! ⭐ **E10 — o console de UM script**: o que a cena disse (`print`, erros de runtime, recusas de
//! recarga), guardado por `ScriptId` mesmo com a aba fechada, e a faixa de baixo da aba desse script:
//! aberta ou fechada, e em que separador (Console ou Problems).
//!
//! ⭐ **Onda A (doc 13 §4.1) — um problema, um lugar:** os erros do Play, as recusas da API e as de
//! recarga NÃO são linhas do console: agrupam-se por (espécie, linha, 1.ª linha da mensagem) em
//! [`PlayProblem`] — com os objectos e a contagem — e a lista de Problems mostra-os
//! (`play_problems`, na família). O console fica com o que a cena DISSE (`print`), com quem e quando.

use crate::console::{BandTab, ConsoleRow, ConsoleSource};
use crate::status::Tone;
use ph2d_i18n::tr_with;
use ph2d_script::{CONSOLE_BYTES_MAX, CONSOLE_LINES_MAX, ConsoleKind, ConsoleLine};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    text: String,
    tone: Tone,
    line: Option<u32>,
    /// «Hero · 0:12» (quem e quando), ou só a hora no topo do script; e de quem (os bits).
    tag: Option<String>,
    who: Option<u64>,
}

pub use crate::doc_rules::game_time;
pub use ph2d_code_assist::play_problems::{PlayKind, PlayProblem};

/// As linhas de um script. Os tectos são os do console do motor ([`CONSOLE_LINES_MAX`] linhas e
/// [`CONSOLE_BYTES_MAX`] bytes de texto, medidos; cada linha já chega cortada ao tecto por linha):
/// a mais antiga sai primeiro. Os bytes contam-se ao entrar e sair — o tecto não percorre as linhas.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Store {
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
                let bits = l.entity.map(ph2d_ecs::Entity::to_bits);
                self.put(l.text.clone(), Tone::Plain, None, Some((tag, bits)));
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
                self.play.push(PlayProblem::new(kind, d.line, message));
                self.play.len() - 1
            }
        };
        self.play[i].seen(entity, who);
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

    fn put(
        &mut self,
        text: String,
        tone: Tone,
        line: Option<u32>,
        tag: Option<(String, Option<u64>)>,
    ) {
        let (tag, who) = tag.map_or((None, None), |(t, w)| (Some(t), w));
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
            who,
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

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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

impl Store {
    /// De quem é a linha `i` do console (os bits do objecto que falou).
    pub fn who_of(&self, i: usize) -> Option<u64> {
        self.entries.get(i)?.who
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
