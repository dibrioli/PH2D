//! ⭐ **Um problema, um lugar** (Onda A, `docs/CodeEditor/13_plano_bancada.md` §4.1): os problemas
//! do Play ([`PlayProblem`]) entram na MESMA lista que os do texto. Um do Play na linha de um do
//! texto funde-se nele — o texto diz a causa, o sufixo diz onde e quantas vezes o Play a viu, e o
//! tom é o mais grave dos dois. Os outros viram linhas próprias, explicadas pelo `ph2d-code-explain`
//! como as do texto. A barra de estado e o título dos Problems contam ESTA lista.

use crate::console::PlayProblem;
use ph2d_i18n::tr_with;
use ph2d_panel_code::Problem;

/// Os nomes que o «On» mostra; o resto conta-se.
const ON_SHOWN: usize = 2;

/// O «On»: «Circle», «Circle, Star» ou «Circle, Star +1»; só sem nome, «2 objects»; vazio sem
/// objectos (o topo do script).
fn who(p: &PlayProblem) -> String {
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

/// O texto que a lista mostra de um problema (o título da explicação, ou a mensagem crua).
fn shown(p: &Problem) -> &str {
    p.explanation
        .as_ref()
        .map_or(p.message.as_str(), |e| e.title.as_str())
        .lines()
        .next()
        .unwrap_or_default()
}

/// Põe o sufixo do Play no que a lista mostra de `p`.
fn suffix(p: &mut Problem, play: &PlayProblem, merged: bool) {
    let text = with_play(shown(p), play, merged);
    p.on = who(play);
    match &mut p.explanation {
        Some(e) => e.title = text,
        None => p.message = text,
    }
}

/// Os problemas do texto (`stat`, já pela ordem da calha) com os do Play fundidos.
pub(crate) fn merge(mut stat: Vec<Problem>, play: &[PlayProblem]) -> Vec<Problem> {
    if play.is_empty() {
        return stat;
    }
    let fixed = stat.len();
    let mut merged = std::collections::BTreeSet::new();
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
            explanation: crate::note::explanation(&g.message),
            on: String::new(),
        };
        suffix(&mut p, g, false);
        stat.push(p);
    }
    stat.sort_by_key(|p| (p.line, p.culprit.as_ref().map(|c| c.start)));
    stat
}

impl crate::doc::CodeDoc {
    /// Os problemas do Play cuja linha ainda tem o texto que correu, na linha onde as edições a
    /// levaram (desfazer o conserto devolve-os: são derivados, não apagados).
    pub(crate) fn live_play(&self) -> Vec<crate::console::PlayProblem> {
        let t = self.buffer.text();
        self.play
            .1
            .iter()
            .filter_map(|g| {
                let Some(n) = g.line else {
                    return Some(g.clone());
                };
                let a = self.play_at.iter().find(|a| a.is(g, n))?;
                let line = t.line_of_byte(a.byte.min(t.len()));
                (t.line_string(line) == a.text).then(|| {
                    let mut g = g.clone();
                    g.line = Some(line as u32 + 1);
                    g
                })
            })
            .collect()
    }

    /// Os problemas do Play guardados para este script (`generation` muda quando eles mudam).
    pub(crate) fn set_play(&mut self, store: Option<&crate::console::Store>) {
        let generation = store.map_or(0, |s| s.play_generation);
        if generation != self.play.0 {
            let list: Vec<_> = store.map(|s| s.play().to_vec()).unwrap_or_default();
            self.anchor_play(&list);
            self.play = (generation, list.into());
        }
    }

    /// Ancora os problemas do Play novos; os já ancorados guardam a âncora (já seguiu as edições).
    /// O texto da linha vem do `synced` (o que o Play corre). Um que chega com a aba suja ancora-se
    /// na MESMA linha do buffer — as edições de antes dele não têm mapa; se ela mudou, não aparece.
    fn anchor_play(&mut self, list: &[crate::console::PlayProblem]) {
        let old = std::mem::take(&mut self.play_at);
        let t = self.buffer.text();
        for g in list {
            let Some(n) = g.line else { continue };
            if let Some(a) = old.iter().find(|a| a.is(g, n)) {
                self.play_at.push(a.clone());
                continue;
            }
            let line = (n as usize).saturating_sub(1);
            let Some(text) = self.synced.split('\n').nth(line) else {
                continue;
            };
            self.play_at.push(PlayAt {
                // A âncora nasce no buffer de agora; o texto, no que correu.
                kind: g.kind,
                line: n,
                message: g.message.clone(),
                byte: t.byte_of_line(line.min(t.line_count().saturating_sub(1))),
                text: text.strip_suffix('\r').unwrap_or(text).to_owned(),
            });
        }
    }
}

/// A âncora de um problema do Play ([`crate::doc::CodeDoc::play_at`]): a identidade do grupo
/// (espécie, linha do texto que correu, mensagem), o byte do início da linha no buffer (segue as
/// edições em `CodeDoc::edited`) e o texto dela.
#[derive(Clone, Debug)]
pub(crate) struct PlayAt {
    kind: crate::console::PlayKind,
    line: u32,
    message: String,
    pub(crate) byte: usize,
    text: String,
}

impl PlayAt {
    fn is(&self, g: &crate::console::PlayProblem, line: u32) -> bool {
        self.kind == g.kind && self.line == line && self.message == g.message
    }
}

#[cfg(test)]
mod tests {
    use ph2d_script::{ConsoleKind, ConsoleLine, DiagnosticKind, ScriptDiagnostic, ScriptId};

    /// O «On» de objectos sem nome conta-os («2 objects»), nunca « +2» sozinho.
    #[test]
    fn o_on_sem_nomes_conta_os_objectos() {
        let mut s = crate::console::Store::default();
        for bits in [7, 8] {
            let d = ScriptDiagnostic {
                kind: DiagnosticKind::Runtime,
                line: Some(2),
                message: "boom".to_owned(),
                traceback: None,
            };
            let l = ConsoleLine {
                script: ScriptId(1),
                entity: Some(ph2d_ecs::Entity::from_bits(bits)),
                text: String::new(),
                kind: ConsoleKind::RuntimeError(d),
                at_us: 0,
            };
            s.push(&l, None);
        }
        assert_eq!(s.play().len(), 1, "controlo: um problema agrupado");
        assert_eq!(super::who(&s.play()[0]), "2 objects");
    }
}
