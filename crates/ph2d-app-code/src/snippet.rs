//! ⭐ **I12 — o modelo com buracos** (`docs/CodeEditor/03_onda2.md`; medição
//! `ferramentas/medicoes_onda2/I9_I12_I14.md` §I12, o VS Code 1.142.0 corrido).
//!
//! - **Leitura:** a gramática de snippets do LSP 3.17 (`$1`, `${1:texto}`, `${1|a,b|}`, `$0`, as
//!   variáveis e os escapes `\$ \} \\`); a cópia de um número (`$1` depois de `${1:x}`) leva o texto dele.
//! - **Sessão** ([`Tabstops`]): o 1.º buraco seleccionado (as cópias = mais cursores), os outros
//!   marcados; escrever nas pontas do buraco activo estica-o; Tab/Shift+Tab andam pelos números; depois
//!   do último, o `$0` (ou o fim do modelo) e a sessão acaba. Esc, sair do modelo, desfazer ou refazer
//!   acabam-na (a aba decide, [`crate::keys`]).

use ph2d_code_buffer::{Range as Cursor, Selection, TextEdit};
use std::collections::BTreeMap;
use std::ops::Range;

/// Um modelo lido: o texto que entra e onde ficam os buracos (bytes desde o início dele).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expanded {
    pub text: String,
    /// Os buracos numerados (> 0), pela ordem do número; cada um com as faixas das suas cópias.
    pub stops: Vec<Vec<Range<usize>>>,
    /// O `$0` (onde a sessão acaba); `None` = o fim do texto.
    pub end: Option<usize>,
}

enum Node {
    Text(String),
    /// Um buraco: o número e o texto de dentro (vazio = `$1`, uma cópia ou um buraco sem texto).
    Stop(u32, Vec<Node>),
}

struct Parser<'a> {
    s: &'a [char],
    i: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }

    fn int(&mut self) -> Option<u32> {
        let start = self.i;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.i += 1;
        }
        (self.i > start)
            .then(|| {
                self.s[start..self.i]
                    .iter()
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .flatten()
    }

    fn var(&mut self) -> Option<String> {
        let start = self.i;
        if !self
            .peek()
            .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        {
            return None;
        }
        while self
            .peek()
            .is_some_and(|c| c == '_' || c.is_ascii_alphanumeric())
        {
            self.i += 1;
        }
        Some(self.s[start..self.i].iter().collect())
    }

    /// Até ao `}` que fecha (ou ao fim), com `inner` = estamos dentro de `${…}`.
    fn nodes(&mut self, inner: bool) -> Vec<Node> {
        let mut out = Vec::new();
        let mut text = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\\' => {
                    self.i += 1;
                    match self.peek() {
                        Some(n @ ('$' | '}' | '\\')) => {
                            text.push(n);
                            self.i += 1;
                        }
                        _ => text.push('\\'),
                    }
                }
                '}' if inner => break,
                '$' => {
                    let save = self.i;
                    self.i += 1;
                    match self.dollar() {
                        Some(n) => {
                            if !text.is_empty() {
                                out.push(Node::Text(std::mem::take(&mut text)));
                            }
                            out.push(n);
                        }
                        None => {
                            self.i = save + 1;
                            text.push('$');
                        }
                    }
                }
                c => {
                    text.push(c);
                    self.i += 1;
                }
            }
        }
        if !text.is_empty() {
            out.push(Node::Text(text));
        }
        out
    }

    /// Depois de um `$`: um buraco, uma escolha, uma variável — ou `None` (o `$` é texto).
    fn dollar(&mut self) -> Option<Node> {
        if let Some(n) = self.int() {
            return Some(Node::Stop(n, Vec::new()));
        }
        if let Some(v) = self.var() {
            return Some(Node::Text(v));
        }
        if self.peek() != Some('{') {
            return None;
        }
        self.i += 1;
        if let Some(n) = self.int() {
            return match self.peek() {
                Some('}') => {
                    self.i += 1;
                    Some(Node::Stop(n, Vec::new()))
                }
                Some(':') => {
                    self.i += 1;
                    let inner = self.nodes(true);
                    self.i += 1;
                    Some(Node::Stop(n, inner))
                }
                // A escolha leva a 1.ª opção (a lista dela não se mostra).
                Some('|') => {
                    self.i += 1;
                    let mut first = String::new();
                    let mut done = false;
                    while let Some(c) = self.peek() {
                        self.i += 1;
                        match c {
                            '\\' => {
                                if let Some(n) = self.peek() {
                                    if !done {
                                        first.push(n);
                                    }
                                    self.i += 1;
                                }
                            }
                            ',' => done = true,
                            '|' if self.peek() == Some('}') => {
                                self.i += 1;
                                break;
                            }
                            c if !done => first.push(c),
                            _ => {}
                        }
                    }
                    Some(Node::Stop(n, vec![Node::Text(first)]))
                }
                // Uma transformação (`${1/…/…/}`) não se aplica: fica a cópia.
                _ => {
                    self.skip_to_close();
                    Some(Node::Stop(n, Vec::new()))
                }
            };
        }
        let v = self.var()?;
        match self.peek() {
            Some('}') => {
                self.i += 1;
                Some(Node::Text(v))
            }
            // Variável com omissão: nenhuma variável é conhecida aqui, fica a omissão.
            Some(':') => {
                self.i += 1;
                let inner = self.nodes(true);
                self.i += 1;
                let mut s = String::new();
                flat(&inner, &mut s);
                Some(Node::Text(s))
            }
            _ => {
                self.skip_to_close();
                Some(Node::Text(v))
            }
        }
    }

    fn skip_to_close(&mut self) {
        while let Some(c) = self.peek() {
            self.i += 1;
            if c == '\\' {
                self.i += 1;
            } else if c == '}' {
                break;
            }
        }
    }
}

fn flat(nodes: &[Node], out: &mut String) {
    for n in nodes {
        match n {
            Node::Text(t) => out.push_str(t),
            Node::Stop(_, inner) => flat(inner, out),
        }
    }
}

/// O texto de cada número: o da 1.ª vez que ele aparece com texto.
fn defaults(nodes: &[Node], out: &mut BTreeMap<u32, String>) {
    for n in nodes {
        if let Node::Stop(k, inner) = n {
            if !inner.is_empty() && !out.contains_key(k) {
                let mut s = String::new();
                flat(inner, &mut s);
                out.insert(*k, s);
            }
            defaults(inner, out);
        }
    }
}

fn emit(
    nodes: &[Node],
    dflt: &BTreeMap<u32, String>,
    text: &mut String,
    stops: &mut BTreeMap<u32, Vec<Range<usize>>>,
) {
    for n in nodes {
        match n {
            Node::Text(t) => text.push_str(t),
            Node::Stop(k, inner) => {
                let a = text.len();
                if inner.is_empty() {
                    if let Some(d) = dflt.get(k) {
                        text.push_str(d);
                    }
                } else {
                    emit(inner, dflt, text, stops);
                }
                stops.entry(*k).or_default().push(a..text.len());
            }
        }
    }
}

/// Lê um snippet LSP.
#[must_use]
pub fn parse(snippet: &str) -> Expanded {
    let chars: Vec<char> = snippet.chars().collect();
    let nodes = Parser { s: &chars, i: 0 }.nodes(false);
    let mut dflt = BTreeMap::new();
    defaults(&nodes, &mut dflt);
    let mut text = String::new();
    let mut stops = BTreeMap::new();
    emit(&nodes, &dflt, &mut text, &mut stops);
    let end = stops.remove(&0).and_then(|v| v.first().map(|r| r.start));
    Expanded {
        text,
        stops: stops.into_values().collect(),
        end,
    }
}

impl Expanded {
    /// Como entra numa linha indentada com `line_indent`: as linhas seguintes do modelo levam essa
    /// indentação, e cada `\t` a unidade do editor (`unit`), como no VS Code.
    #[must_use]
    pub fn adapt(&self, line_indent: &str, unit: &str) -> Expanded {
        let mut text = String::with_capacity(self.text.len());
        let mut at = vec![0usize; self.text.len() + 1];
        for (i, c) in self.text.char_indices() {
            for k in 0..c.len_utf8() {
                at[i + k] = text.len();
            }
            match c {
                '\t' => text.push_str(unit),
                '\n' => {
                    text.push('\n');
                    text.push_str(line_indent);
                }
                c => text.push(c),
            }
        }
        at[self.text.len()] = text.len();
        Expanded {
            stops: self
                .stops
                .iter()
                .map(|v| v.iter().map(|r| at[r.start]..at[r.end]).collect())
                .collect(),
            end: self.end.map(|e| at[e]),
            text,
        }
    }
}

/// Onde fica `pos` depois da edição `e`; `after` = uma inserção em `pos` deixa-o depois dela.
fn map(pos: usize, e: &TextEdit, after: bool) -> usize {
    let (s, o, n) = (e.start_byte, e.old_end_byte, e.new_end_byte);
    if pos < s {
        pos
    } else if pos > o {
        pos - o + n
    } else if pos == s && (s != o || !after) {
        // No início de uma troca fica antes dela (uma inserção pura, só sem `after`).
        s
    } else if pos == o || after {
        n
    } else {
        s
    }
}

/// Uma faixa da sessão e o sítio (a inserção) de onde veio.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Spot {
    at: usize,
    bytes: Range<usize>,
}

/// A sessão de um modelo inserido (num ou em vários cursores).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tabstops {
    stops: Vec<Vec<Spot>>,
    /// O fim de cada inserção (o `$0`, ou o fim do texto dela).
    ends: Vec<usize>,
    /// A faixa inteira de cada inserção (sair de TODAS acaba a sessão).
    spans: Vec<Range<usize>>,
    active: usize,
    primary: usize,
}

/// O que um Tab fez.
pub(crate) enum Jump {
    /// Outro buraco: seleccione isto.
    Stop(Selection),
    /// Acabou: o cursor no `$0`.
    End(Selection),
}

impl Tabstops {
    /// A sessão para `spots` = (onde o texto de cada inserção começa, já no documento, e o modelo);
    /// `primary` = a inserção do cursor primário. `None` = o modelo não tem buracos.
    pub fn new(spots: &[(usize, &Expanded)], primary: usize) -> Option<Self> {
        let n = spots.iter().map(|(_, e)| e.stops.len()).max()?;
        if n == 0 {
            return None;
        }
        let stops = (0..n)
            .map(|k| {
                spots
                    .iter()
                    .enumerate()
                    .flat_map(|(at, (base, e))| {
                        e.stops.get(k).into_iter().flatten().map(move |r| Spot {
                            at,
                            bytes: base + r.start..base + r.end,
                        })
                    })
                    .collect()
            })
            .collect();
        Some(Self {
            stops,
            ends: spots
                .iter()
                .map(|(b, e)| b + e.end.unwrap_or(e.text.len()))
                .collect(),
            spans: spots.iter().map(|(b, e)| *b..b + e.text.len()).collect(),
            active: 0,
            primary,
        })
    }

    /// A selecção do buraco activo (as cópias são mais cursores; o primário é o da inserção primária).
    pub fn selection(&self) -> Selection {
        let spots = &self.stops[self.active];
        let main = spots.iter().position(|s| s.at == self.primary).unwrap_or(0);
        Selection::new(
            spots
                .iter()
                .map(|s| Cursor::new(s.bytes.start, s.bytes.end))
                .collect(),
            main,
        )
    }

    fn ends_selection(&self) -> Selection {
        let main = self.primary.min(self.ends.len().saturating_sub(1));
        Selection::new(self.ends.iter().map(|&e| Cursor::cursor(e)).collect(), main)
    }

    /// Tab: o buraco seguinte, ou o fim.
    pub fn next(&mut self) -> Jump {
        if self.active + 1 < self.stops.len() {
            self.active += 1;
            Jump::Stop(self.selection())
        } else {
            Jump::End(self.ends_selection())
        }
    }

    /// Shift+Tab: o buraco anterior (no 1.º, ele outra vez, inteiro).
    pub fn prev(&mut self) -> Selection {
        self.active = self.active.saturating_sub(1);
        self.selection()
    }

    /// As faixas seguem `edits` (as do `Buffer`: da última troca para a primeira, no texto de partida).
    pub fn edited(&mut self, edits: &[TextEdit]) {
        for e in edits {
            for (k, spots) in self.stops.iter_mut().enumerate() {
                let active = k == self.active;
                for s in spots {
                    let r = &mut s.bytes;
                    let (a, b) = if active {
                        (map(r.start, e, false), map(r.end, e, true))
                    } else if r.start == r.end {
                        let p = map(r.start, e, true);
                        (p, p)
                    } else {
                        (map(r.start, e, true), map(r.end, e, false))
                    };
                    *r = a.min(b)..b;
                }
            }
            for x in &mut self.ends {
                *x = map(*x, e, true);
            }
            for r in &mut self.spans {
                *r = map(r.start, e, false)..map(r.end, e, true);
            }
        }
    }

    /// `inner` (já no documento, depois de [`Self::edited`]) foi inserido DENTRO do buraco activo: as
    /// faixas de todas as inserções dele cabem numa cópia desse buraco.
    pub fn nests(&self, inner: &Self) -> bool {
        inner.spans.iter().all(|r| {
            self.stops[self.active]
                .iter()
                .any(|s| s.bytes.start <= r.start && r.end <= s.bytes.end)
        })
    }

    /// Funde `inner` (inserido dentro do buraco activo, [`Self::nests`]) nesta sessão, como o VS Code
    /// 1.142: o buraco activo deixa de ser paragem; no lugar dele, os buracos de dentro e o `$0` de
    /// dentro (uma paragem como as outras); depois seguem os de fora, e a sessão acaba no `$0` de
    /// fora. Oráculo: `docs/CodeEditor/ferramentas/oraculo_snippet_aninhado/saida/amostras.txt`.
    pub fn merge(&mut self, inner: Self) {
        let outer = &self.stops[self.active];
        // Cada inserção de dentro pertence à cópia de fora que a contém (o cursor primário é o dela).
        let owner = |r: &Range<usize>| {
            outer
                .iter()
                .find(|s| s.bytes.start <= r.start && r.end <= s.bytes.end)
                .map_or(self.primary, |s| s.at)
        };
        let at_of: Vec<usize> = inner.spans.iter().map(owner).collect();
        let mut merged: Vec<Vec<Spot>> = inner
            .stops
            .into_iter()
            .map(|v| {
                v.into_iter()
                    .map(|s| Spot {
                        at: at_of[s.at],
                        ..s
                    })
                    .collect()
            })
            .collect();
        merged.push(
            inner
                .ends
                .iter()
                .enumerate()
                .map(|(j, &e)| Spot {
                    at: at_of[j],
                    bytes: e..e,
                })
                .collect(),
        );
        self.stops.splice(self.active..=self.active, merged);
        self.spans.extend(inner.spans);
    }

    /// `head` ainda está dentro de alguma inserção (sair de todas acaba a sessão).
    pub fn holds(&self, head: usize) -> bool {
        self.spans.iter().any(|r| r.start <= head && head <= r.end)
    }

    /// Os buracos que NÃO estão activos (a aba marca-os).
    pub fn marks(&self) -> Vec<Range<usize>> {
        let mut out: Vec<Range<usize>> = self
            .stops
            .iter()
            .enumerate()
            .filter(|(k, _)| *k != self.active)
            .flat_map(|(_, v)| v.iter().map(|s| s.bytes.clone()))
            .collect();
        out.sort_by_key(|r| r.start);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gramatica_do_lsp() {
        let e = parse("set(${1:entity}, ${2:field}, ${3:value})$0");
        assert_eq!(e.text, "set(entity, field, value)");
        assert_eq!(e.stops, vec![vec![4..10], vec![12..17], vec![19..24]]);
        assert_eq!(e.end, Some(25));
        let e = parse("const ${1:x} = ${2:0}; log($1);$0");
        assert_eq!(e.text, "const x = 0; log(x);");
        assert_eq!(
            e.stops[0],
            vec![6..7, 17..18],
            "a cópia leva o texto do número"
        );
        let e = parse("a\\$b ${1|um,dois|} ${2:f(${3:x})} $TM_X ${V:omissão} \\}");
        assert_eq!(e.text, "a$b um f(x) TM_X omissão }");
        assert_eq!(e.stops, vec![vec![4..6], vec![7..11], vec![9..10]]);
        assert_eq!(e.end, None);
        assert_eq!(parse("print($1)$0").stops, vec![vec![6..6]]);
        assert_eq!(parse("custa $ 5").text, "custa $ 5");
    }

    #[test]
    fn as_linhas_seguintes_levam_a_indentacao_da_linha() {
        let e = parse("function f(${1:a})\n\t$0\nend").adapt("  ", "  ");
        assert_eq!(e.text, "function f(a)\n    \n  end");
        assert_eq!((e.stops.clone(), e.end), (vec![vec![11..12]], Some(18)));
    }
}
