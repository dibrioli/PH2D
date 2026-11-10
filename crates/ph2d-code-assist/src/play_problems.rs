//! ⭐ **Um problema, um lugar** (Onda A, `docs/CodeEditor/13_plano_bancada.md` §4.1): os problemas
//! do Play ([`PlayProblem`]) entram na MESMA lista que os do texto. Um do Play na linha de um do
//! texto funde-se nele — o texto diz a causa, o sufixo diz onde e quantas vezes o Play a viu, e o
//! tom é o mais grave dos dois. Os outros viram linhas próprias, explicadas pelo `ph2d-code-explain`
//! como as do texto. A barra de estado e o título dos Problems contam ESTA lista.
//!
//! A ordem da lista é a do VS Code 1.142: gravidade (erro, aviso, informação), depois a linha,
//! depois o início do culpado ([`order`]).

use crate::problems::explanation;
use ph2d_code_viewmodel::{Problem, Severity};
use ph2d_i18n::tr_with;
use std::cmp::Reverse;
use std::collections::BTreeSet;

/// De onde vem um problema do Play.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayKind {
    Runtime,
    Refused,
    Warning,
}

/// ⭐ **Um problema do Play** — todas as ocorrências da MESMA (espécie, linha, mensagem), com os
/// objectos onde aconteceu (os primeiros [`NAMES_SHOWN`] pelo nome, o resto contado) e quantas vezes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayProblem {
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
pub const NAMES_SHOWN: usize = 3;

/// Os nomes que o «On» mostra; o resto conta-se.
const ON_SHOWN: usize = 2;

impl PlayProblem {
    /// Um grupo novo, ainda sem ocorrências.
    #[must_use]
    pub fn new(kind: PlayKind, line: Option<u32>, message: &str) -> Self {
        Self {
            kind,
            line,
            message: message.to_owned(),
            names: Vec::new(),
            entities: BTreeSet::new(),
            count: 0,
        }
    }

    /// Mais uma ocorrência, no objecto `entity` (os bits) chamado `who`.
    pub fn seen(&mut self, entity: Option<u64>, who: Option<&str>) {
        self.count += 1;
        if let Some(e) = entity
            && self.entities.insert(e)
            && self.names.len() < NAMES_SHOWN
            && let Some(w) = who
        {
            self.names.push(w.to_owned());
        }
    }

    #[must_use]
    pub fn severity(&self) -> Severity {
        match self.kind {
            PlayKind::Warning => Severity::Warning,
            PlayKind::Runtime | PlayKind::Refused => Severity::Error,
        }
    }

    /// Quantos objectos ficaram fora dos nomes mostrados.
    #[must_use]
    pub fn more(&self) -> usize {
        self.entities.len().saturating_sub(self.names.len())
    }
}

/// O «On»: «Circle», «Circle, Star» ou «Circle, Star +1»; só sem nome, «2 objects»; vazio sem
/// objectos (o topo do script).
#[must_use]
pub fn who(p: &PlayProblem) -> String {
    let rest = p.names.len().saturating_sub(ON_SHOWN) + p.more();
    let names = p.names[..p.names.len().min(ON_SHOWN)].join(", ");
    match rest {
        0 => names,
        n if names.is_empty() => tr_with("code.problems.objects", &[("count", &n)]),
        n => tr_with("code.problems.more", &[("names", &names), ("count", &n)]),
    }
}

/// `text` com o sufixo do Play: o «×N» (só com N > 1); `merged` = funde-se num problema do texto
/// (o sufixo diz «in Play»). Os objectos vão ao `Problem.on`.
fn with_play(text: &str, p: &PlayProblem, merged: bool) -> String {
    let n = p.count;
    let key = match (n > 1, merged) {
        (true, false) => "code.problems.times",
        (false, true) => "code.problems.in_play",
        (true, true) => "code.problems.times_in_play",
        (false, false) => return text.to_owned(),
    };
    tr_with(key, &[("text", &text), ("count", &n)])
}

/// O texto de uma linha da lista: o título da explicação (I8) quando a há, senão a mensagem crua
/// (a 1.ª linha dela).
#[must_use]
pub fn text(p: &Problem) -> &str {
    p.explanation
        .as_ref()
        .map_or(p.message.as_str(), |e| e.title.as_str())
        .lines()
        .next()
        .unwrap_or_default()
}

/// Põe o sufixo do Play no que a lista mostra de `p`.
fn suffix(p: &mut Problem, play: &PlayProblem, merged: bool) {
    let text = with_play(text(p), play, merged);
    p.on = who(play);
    match &mut p.explanation {
        Some(e) => e.title = text,
        None => p.message = text,
    }
}

/// Os problemas do texto (`stat`, já pela ordem da calha) com os do Play fundidos.
#[must_use]
pub fn merge(mut stat: Vec<Problem>, play: &[PlayProblem]) -> Vec<Problem> {
    if play.is_empty() {
        return stat;
    }
    let fixed = stat.len();
    let mut merged = BTreeSet::new();
    for g in play {
        let line = g.line.map_or(0, |n| n.saturating_sub(1) as usize);
        if g.line.is_some()
            && merged.insert(line)
            && let Some(p) = stat[..fixed].iter_mut().find(|p| p.line == line)
        {
            p.severity = p.severity.max(g.severity());
            suffix(p, g, true);
            continue;
        }
        let mut p = Problem {
            line,
            severity: g.severity(),
            culprit: None,
            message: g.message.clone(),
            explanation: explanation(&g.message),
            on: String::new(),
        };
        suffix(&mut p, g, false);
        stat.push(p);
    }
    stat.sort_by_key(|p| (p.line, p.culprit.as_ref().map(|c| c.start)));
    stat
}

/// Um problema na ordem da lista (o F8 segue-o entre quadros, mesmo que a lista mude).
pub type Key = (Reverse<Severity>, usize, Option<usize>, String);

/// A chave de ordem de `p`, emprestada.
#[must_use]
pub fn sort_key(p: &Problem) -> (Reverse<Severity>, usize, Option<usize>, &str) {
    (
        Reverse(p.severity),
        p.line,
        p.culprit.as_ref().map(|c| c.start),
        p.message.as_str(),
    )
}

/// A [`Key`] de `p`, para guardar entre quadros.
#[must_use]
pub fn key(p: &Problem) -> Key {
    let (s, l, c, m) = sort_key(p);
    (s, l, c, m.to_owned())
}

/// `k` na forma de [`sort_key`], para comparar.
#[must_use]
pub fn key_ref(k: &Key) -> (Reverse<Severity>, usize, Option<usize>, &str) {
    (k.0, k.1, k.2, k.3.as_str())
}

/// Os índices de `ps` pela ordem da lista.
#[must_use]
pub fn order(ps: &[Problem]) -> Vec<usize> {
    let mut v: Vec<usize> = (0..ps.len()).collect();
    v.sort_by(|&a, &b| sort_key(&ps[a]).cmp(&sort_key(&ps[b])));
    v
}
