//! ⭐ **I4 — a lista de completação** (`docs/CodeEditor/03_onda2.md`): o que o assistente propõe no
//! cursor, filtrado e ordenado no cliente enquanto se escreve.
//!
//! - **Abre sozinha** numa pausa de [`AUTO_OPEN_NS`] depois de uma letra de palavra, de `.`/`:` ou de
//!   uma aspa (o `code_complete_delay` do Godot, I0b §7); fora de strings e comentários. **Ctrl+Espaço**
//!   abre-a na hora, em qualquer sítio (só no texto do script: no resto do app é o modo foco).
//! - **Filtro e ordem**: a lei do `CodeEdit` do Godot, corrido sobre entradas nossas
//!   (`tests/it/fixtures/godot_completar.txt`): subsequência sem caixa, cinco faixas (prefixo exacto ·
//!   prefixo · trecho exacto · trecho · letras soltas), depois a posição, a caixa, o fim e o nome. Entre
//!   as duas, a relevância do servidor (`sortText`: o tipo esperado) e o USO no projeto.
//! - O `luau-lsp` não filtra (`ph2d.se` devolve os 17) e nunca diz `isIncomplete` (sonda de 16/10): um
//!   pedido por sessão, o resto é nosso. Depois de `self.` ele só conhece os campos ATRIBUÍDOS; as
//!   propriedades `ph2d.property` entram por nós.

use crate::doc::map_byte;
use ph2d_code_buffer::{Text, TextEdit};
use ph2d_code_intel::Ticket;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::ops::Range;

/// A pausa antes de a lista abrir sozinha: o `code_complete_delay` do Godot 4.7.2 (I0b §7, 0,3 s).
pub(crate) const AUTO_OPEN_NS: u128 = 300_000_000;

/// O `kind` LSP de um campo (as propriedades do `self` que juntamos).
const FIELD: u32 = 5;

/// Uma proposta, já com o uso dela no projeto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub label: String,
    /// O `kind` do LSP (3 função · 5 campo · 6 variável · 14 palavra reservada · 21 constante).
    pub kind: Option<u32>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    /// A relevância do servidor (`"1"` membro, `"2"` o tipo esperado, `"4"` global, `"8"` palavra).
    pub sort_text: String,
    pub filter: Option<String>,
    pub insert: String,
    /// `insert` é um snippet LSP (`set(${1:entity})$0`).
    pub snippet: bool,
    pub usage: u32,
}

/// Como uma proposta casa com o que se escreveu: a chave de ordem do Godot e as letras que casaram
/// (índices de CARÁCTER do rótulo, para o realce).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    /// 0 prefixo exacto · 1 prefixo · 2 trecho exacto · 3 trecho · 4 letras soltas.
    pub tier: u8,
    pub start: usize,
    /// Letras casadas com outra caixa.
    pub mismatches: usize,
    pub end: usize,
    pub chars: Vec<usize>,
}

/// Como `typed` casa com `label` (`None` = não casa: sai da lista).
#[must_use]
pub fn matches(typed: &str, label: &str) -> Option<Match> {
    let t: Vec<char> = typed.chars().collect();
    let l: Vec<char> = label.chars().collect();
    let lower = |c: &char| c.to_lowercase().next().unwrap_or(*c);
    let tl: Vec<char> = t.iter().map(lower).collect();
    let ll: Vec<char> = l.iter().map(lower).collect();
    let span = |start: usize, tier: u8| Match {
        tier,
        start,
        mismatches: (0..t.len()).filter(|&i| l[start + i] != t[i]).count(),
        end: (start + t.len()).saturating_sub(1),
        chars: (start..start + t.len()).collect(),
    };
    let find = |hay: &[char], needle: &[char]| {
        (needle.len() <= hay.len())
            .then(|| (0..=hay.len() - needle.len()).find(|&i| hay[i..].starts_with(needle)))
            .flatten()
    };
    if l.starts_with(&t) {
        return Some(span(0, 0));
    }
    if ll.starts_with(&tl) {
        return Some(span(0, 1));
    }
    if let Some(i) = find(&l, &t) {
        return Some(span(i, 2));
    }
    if let Some(i) = find(&ll, &tl) {
        return Some(span(i, 3));
    }
    let mut chars = Vec::with_capacity(t.len());
    let mut from = 0;
    for c in &tl {
        let i = from + ll[from..].iter().position(|x| x == c)?;
        chars.push(i);
        from = i + 1;
    }
    Some(Match {
        tier: 4,
        start: chars[0],
        mismatches: chars.iter().zip(&t).filter(|(i, c)| l[**i] != **c).count(),
        end: *chars.last().unwrap_or(&0),
        chars,
    })
}

/// O desempate pelo nome: sem caixa, com o `_` depois das letras e dos dígitos; depois com caixa
/// (as ordens do Godot: `getAll` antes de `get_all`, `Vec2` antes de `vec2`).
fn name_key(label: &str) -> (Vec<char>, &str) {
    let k = label
        .chars()
        .map(|c| {
            if c == '_' {
                char::MAX
            } else {
                c.to_lowercase().next().unwrap_or(c)
            }
        })
        .collect();
    (k, label)
}

/// A lista que se MOSTRA para `typed`: os índices de `items` que casam, pela ordem, com o que casou.
#[must_use]
pub fn order(typed: &str, items: &[Item]) -> Vec<(usize, Match)> {
    let mut out: Vec<(usize, Match)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, it)| {
            Some((
                i,
                matches(typed, it.filter.as_deref().unwrap_or(&it.label))?,
            ))
        })
        .collect();
    out.sort_by(|(a, ma), (b, mb)| {
        let (ia, ib) = (&items[*a], &items[*b]);
        (
            ma.tier,
            &ia.sort_text,
            Reverse(ia.usage),
            ma.start,
            ma.mismatches,
            ma.end,
        )
            .cmp(&(
                mb.tier,
                &ib.sort_text,
                Reverse(ib.usage),
                mb.start,
                mb.mismatches,
                mb.end,
            ))
            .then_with(|| name_key(&ia.label).cmp(&name_key(&ib.label)))
    });
    out
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Uma palavra que se completa: não começa por dígito (essa é um número; o Godot também não abre).
#[must_use]
pub fn completable(word: &str) -> bool {
    !word.starts_with(|c: char| c.is_ascii_digit())
}

/// O contexto do cursor: onde começa a palavra que se escreve e o que vem antes dela.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Word {
    pub start: usize,
    /// `ph2d` em `ph2d.se|`, `self` em `self.|` (`None` = não é membro de um nome simples).
    pub qualifier: Option<String>,
    /// Logo depois de uma aspa: um literal (as opções de `ph2d.set(id, "|")`).
    pub quote: bool,
}

/// A palavra no cursor `head` (`None` = uma palavra que começa por dígito: um número).
pub(crate) fn word_at(text: &Text, head: usize) -> Option<Word> {
    let line = text.byte_of_line(text.line_of_byte(head));
    let before = text.slice(line..head);
    let start = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map_or(before.len(), |(i, _)| i);
    if !completable(&before[start..]) {
        return None;
    }
    let lead = &before[..start];
    let mut q = None;
    if let Some(rest) = lead.strip_suffix(['.', ':'])
        && !rest.ends_with('.')
    {
        let s = rest
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_word(*c))
            .last()
            .map_or(rest.len(), |(i, _)| i);
        let name = &rest[s..];
        if !name.is_empty() && !name.starts_with(|c: char| c.is_ascii_digit()) {
            q = Some(name.to_owned());
        }
    }
    Some(Word {
        start: line + start,
        qualifier: q,
        quote: lead.ends_with(['"', '\'']),
    })
}

/// O que um snippet LSP insere sem paragens (os buracos chegam no I12): o texto antes da 1.ª
/// paragem numerada e depois da última, e o cursor na 1.ª (`set(${1:a}, ${2:b})$0` → `set(|)`).
#[must_use]
pub fn expand(snippet: &str) -> (String, usize) {
    // (texto, paragens: (número, byte no texto))
    let mut text = String::new();
    let mut stops: Vec<(u32, usize, usize)> = Vec::new();
    let mut it = snippet.chars().peekable();
    let mut depth: Vec<(u32, usize)> = Vec::new();
    while let Some(c) = it.next() {
        match c {
            '\\' => {
                if let Some(n) = it.next() {
                    text.push(n);
                }
            }
            '$' if it.peek().is_some_and(char::is_ascii_digit) => {
                let mut n = 0;
                while let Some(d) = it.peek().and_then(|d| d.to_digit(10)) {
                    n = n * 10 + d;
                    it.next();
                }
                stops.push((n, text.len(), text.len()));
            }
            '$' if it.peek() == Some(&'{') => {
                it.next();
                let mut n = 0;
                while let Some(d) = it.peek().and_then(|d| d.to_digit(10)) {
                    n = n * 10 + d;
                    it.next();
                }
                if it.peek() == Some(&':') {
                    it.next();
                }
                depth.push((n, text.len()));
            }
            '}' if !depth.is_empty() => {
                let (n, at) = depth.pop().unwrap_or((0, 0));
                stops.push((n, at, text.len()));
            }
            _ => text.push(c),
        }
    }
    let numbered: Vec<&(u32, usize, usize)> = stops.iter().filter(|s| s.0 > 0).collect();
    match (numbered.first(), numbered.last()) {
        (Some(first), Some(last)) => {
            let a = numbered.iter().map(|s| s.1).min().unwrap_or(first.1);
            let b = numbered.iter().map(|s| s.2).max().unwrap_or(last.2);
            (format!("{}{}", &text[..a], &text[b..]), a)
        }
        _ => {
            let at = stops.iter().find(|s| s.0 == 0).map_or(text.len(), |s| s.1);
            (text, at)
        }
    }
}

/// Quantas vezes cada nome aparece nos scripts do projeto (fora de strings e comentários), e cada
/// `a.b`/`a:b` também (a chave de um membro de `ph2d`).
#[must_use]
pub fn usage<'a>(texts: impl IntoIterator<Item = &'a str>) -> BTreeMap<String, u32> {
    let mut out = BTreeMap::new();
    for s in texts {
        count_words(s, &mut out);
    }
    out
}

/// A chave de uso de uma proposta.
fn usage_key(qualifier: Option<&str>, label: &str) -> String {
    qualifier.map_or_else(|| label.to_owned(), |q| format!("{q}.{label}"))
}

/// O nível de um `[[`/`[==[` longo que começa em `b[i]` (`None` = não é um).
fn long_bracket(b: &[u8], i: usize) -> Option<usize> {
    if b.get(i) != Some(&b'[') {
        return None;
    }
    let eq = b[i + 1..].iter().take_while(|&&c| c == b'=').count();
    (b.get(i + 1 + eq) == Some(&b'[')).then_some(eq)
}

/// O fim (exclusivo) do longo de nível `eq` cujo conteúdo começa em `from`.
fn long_end(b: &[u8], from: usize, eq: usize) -> usize {
    let mut close = vec![b']'];
    close.extend(std::iter::repeat_n(b'=', eq));
    close.push(b']');
    b[from..]
        .windows(close.len())
        .position(|w| w == close.as_slice())
        .map_or(b.len(), |p| from + p + close.len())
}

fn count_words(s: &str, out: &mut BTreeMap<String, u32>) {
    let b = s.as_bytes();
    let mut i = 0;
    // O último nome e se veio um `.`/`:` depois dele.
    let mut prev: Option<(usize, usize)> = None;
    let mut member = false;
    while i < b.len() {
        let c = b[i];
        if c == b'-' && b.get(i + 1) == Some(&b'-') {
            i = match long_bracket(b, i + 2) {
                Some(eq) => long_end(b, i + 4 + eq, eq),
                None => b[i..]
                    .iter()
                    .position(|&c| c == b'\n')
                    .map_or(b.len(), |p| i + p),
            };
            prev = None;
            continue;
        }
        if let Some(eq) = long_bracket(b, i) {
            i = long_end(b, i + 2 + eq, eq);
            prev = None;
            continue;
        }
        if matches!(c, b'"' | b'\'' | b'`') {
            i += 1;
            while i < b.len() && b[i] != c && b[i] != b'\n' {
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            i += 1;
            prev = None;
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let st = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let w = &s[st..i];
            *out.entry(w.to_owned()).or_insert(0) += 1;
            if member && let Some((ps, pe)) = prev {
                *out.entry(format!("{}.{w}", &s[ps..pe])).or_insert(0) += 1;
            }
            prev = Some((st, i));
            member = false;
            continue;
        }
        if c.is_ascii_digit() {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || b[i] == b'_') {
                i += 1;
            }
            prev = None;
            continue;
        }
        if (c == b'.' || c == b':') && prev.is_some() && !member && b.get(i + 1) != Some(&b'.') {
            member = true;
        } else if !c.is_ascii_whitespace() {
            prev = None;
            member = false;
        }
        i += 1;
    }
}

/// Uma sessão de completação: a palavra que se escreve (o início segue as edições) e a resposta.
#[derive(Debug)]
pub(crate) struct Session {
    pub anchor: usize,
    pub qualifier: Option<String>,
    /// Quando a lista pode abrir (a pausa); o pedido sai nesse quadro.
    pub due_ns: u128,
    pub ticket: Option<Ticket>,
    pub items: Option<Vec<Item>>,
    pub selected: usize,
    pub top: usize,
    /// O que estava escrito da última vez que a seleção foi posta (escrever volta-a ao topo).
    typed: String,
}

/// A lista de uma aba.
#[derive(Debug, Default)]
pub(crate) struct Completer {
    pub session: Option<Session>,
}

impl Completer {
    pub fn close(&mut self) {
        self.session = None;
    }

    /// O início da palavra segue as edições; o que se escreve NELE fica dentro da palavra.
    pub fn edited(&mut self, edits: &[TextEdit]) {
        if let Some(s) = &mut self.session {
            for e in edits {
                if s.anchor > e.start_byte {
                    s.anchor = map_byte(s.anchor, e);
                }
            }
        }
    }

    /// Depois de escrever `ch` com o cursor em `head`: a sessão continua se a palavra é a mesma; uma
    /// letra de palavra, `.`/`:` ou uma aspa abrem uma nova (`code` = o cursor não está numa string
    /// nem num comentário).
    pub fn typed(&mut self, text: &Text, head: usize, ch: char, code: bool, now_ns: u128) {
        let w = word_at(text, head);
        if let (Some(s), Some(w)) = (&self.session, &w)
            && s.anchor == w.start
            && is_word(ch)
        {
            return;
        }
        self.session = None;
        let Some(w) = w else { return };
        let opens = match ch {
            '.' | ':' => code && w.qualifier.is_some(),
            '"' | '\'' => w.quote,
            c => code && is_word(c) && w.start < head,
        };
        if opens {
            self.session = Some(Session {
                anchor: w.start,
                qualifier: w.qualifier,
                due_ns: now_ns + AUTO_OPEN_NS,
                ticket: None,
                items: None,
                selected: 0,
                top: 0,
                typed: String::new(),
            });
        }
    }

    /// Pedida à mão (Ctrl+Espaço): sem pausa, e também numa string ou num comentário. Uma sessão da
    /// mesma palavra fica (e perde a pausa que faltava).
    pub fn open_now(&mut self, text: &Text, head: usize, now_ns: u128) {
        let Some(w) = word_at(text, head) else {
            self.session = None;
            return;
        };
        if let Some(s) = self.session.as_mut().filter(|s| s.anchor == w.start) {
            s.due_ns = s.due_ns.min(now_ns);
            return;
        }
        self.session = Some(Session {
            anchor: w.start,
            qualifier: w.qualifier,
            due_ns: now_ns,
            ticket: None,
            items: None,
            selected: 0,
            top: 0,
            typed: String::new(),
        });
    }

    /// Depois de apagar ou de mover o cursor: a sessão morre se o cursor saiu da palavra.
    pub fn moved(&mut self, text: &Text, head: usize) {
        let keep = self.session.as_ref().is_some_and(|s| {
            head >= s.anchor && word_at(text, head).is_some_and(|w| w.start == s.anchor)
        });
        if !keep {
            self.session = None;
        }
    }

    /// O que está escrito da palavra (o filtro).
    pub fn typed_text(&self, text: &Text, head: usize) -> Option<String> {
        let s = self.session.as_ref()?;
        (head >= s.anchor && head <= text.len()).then(|| text.slice(s.anchor..head))
    }

    /// A lista que se mostra agora (`None` = fechada): a resposta chegou, a pausa passou e algo casa.
    pub fn shown(&mut self, text: &Text, head: usize) -> Option<Vec<(usize, Match)>> {
        let typed = self.typed_text(text, head)?;
        let s = self.session.as_mut()?;
        let items = s.items.as_ref()?;
        let list = order(&typed, items);
        if list.is_empty() {
            return None;
        }
        if s.typed != typed {
            s.typed = typed;
            s.selected = 0;
            s.top = 0;
        }
        s.selected = s.selected.min(list.len() - 1);
        Some(list)
    }

    /// Move a seleção `by` linhas (dá a volta nas pontas, como no Godot e no VS Code); a janela de
    /// `rows` linhas segue-a.
    pub fn step(&mut self, by: isize, len: usize, rows: usize) {
        let Some(s) = self.session.as_mut() else {
            return;
        };
        if len == 0 {
            return;
        }
        let n = len as isize;
        s.selected = if by.abs() > 1 {
            (s.selected as isize + by).clamp(0, n - 1) as usize
        } else {
            (s.selected as isize + by).rem_euclid(n) as usize
        };
        s.follow(rows);
    }

    /// A roda do rato sobre a lista: rola a janela, a seleção fica.
    pub fn scroll(&mut self, by: isize, len: usize, rows: usize) {
        if let Some(s) = self.session.as_mut() {
            let max = len.saturating_sub(rows) as isize;
            s.top = (s.top as isize + by).clamp(0, max) as usize;
        }
    }

    /// A resposta do servidor ao pedido `ticket` (as propriedades do `self` juntam-se aqui).
    pub fn answer(
        &mut self,
        ticket: Ticket,
        items: Vec<ph2d_code_intel::Completion>,
        properties: &[String],
        usage: &BTreeMap<String, u32>,
    ) -> bool {
        let Some(s) = self.session.as_mut().filter(|s| s.ticket == Some(ticket)) else {
            return false;
        };
        let q = s.qualifier.as_deref();
        let mut list: Vec<Item> = items
            .into_iter()
            .map(|c| Item {
                usage: usage.get(&usage_key(q, &c.label)).copied().unwrap_or(0),
                label: c.label,
                kind: c.kind,
                detail: c.detail,
                documentation: c.documentation,
                sort_text: c.sort_text.unwrap_or_default(),
                filter: c.filter,
                insert: c.insert,
                snippet: c.snippet,
            })
            .collect();
        if q == Some("self") {
            for p in properties {
                if list.iter().all(|i| &i.label != p) {
                    list.push(Item {
                        label: p.clone(),
                        kind: Some(FIELD),
                        detail: Some(ph2d_i18n::tr("code.complete.property").to_owned()),
                        documentation: None,
                        sort_text: "1".to_owned(),
                        filter: None,
                        insert: p.clone(),
                        snippet: false,
                        usage: usage.get(&usage_key(q, p)).copied().unwrap_or(0),
                    });
                }
            }
        }
        s.items = Some(list);
        true
    }
}

impl Session {
    fn follow(&mut self, rows: usize) {
        if self.selected < self.top {
            self.top = self.selected;
        } else if self.selected >= self.top + rows {
            self.top = self.selected + 1 - rows;
        }
    }
}

/// O texto a inserir para `item` sobre a palavra, e onde fica o cursor (bytes desde o início).
#[must_use]
pub(crate) fn insertion(item: &Item) -> (String, usize) {
    if item.snippet {
        expand(&item.insert)
    } else {
        (item.insert.clone(), item.insert.len())
    }
}

/// A faixa de bytes que a aceitação troca: do início da palavra ao cursor.
pub(crate) fn replaced(s: &Session, head: usize) -> Range<usize> {
    s.anchor..head
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn um_snippet_insere_o_nome_e_os_parenteses_com_o_cursor_dentro() {
        assert_eq!(
            expand("set(${1:entity}, ${2:field}, ${3:value})$0"),
            ("set()".into(), 4)
        );
        assert_eq!(expand("spawn()$0"), ("spawn()".into(), 7));
        assert_eq!(expand("print($1)$0"), ("print()".into(), 6));
        assert_eq!(expand("a\\$b"), ("a$b".into(), 3));
        assert_eq!(expand("plain"), ("plain".into(), 5));
    }

    #[test]
    fn a_palavra_no_cursor_sabe_o_que_vem_antes() {
        let t = Text::new("  ph2d.se\nself.\nx = 1.5\na..b\nf(\"\")\n");
        let at = |s: &str| t.slice(0..t.len()).find(s).unwrap() + s.len();
        let w = word_at(&t, at("ph2d.se")).unwrap();
        assert_eq!(
            (w.start, w.qualifier.as_deref()),
            (at("ph2d."), Some("ph2d"))
        );
        assert_eq!(
            word_at(&t, at("self.")).unwrap().qualifier.as_deref(),
            Some("self")
        );
        assert_eq!(word_at(&t, at("1.5")), None, "um número não se completa");
        assert_eq!(
            word_at(&t, at("a..b")).unwrap().qualifier,
            None,
            "`..` concatena"
        );
        assert!(word_at(&t, at("f(\"")).unwrap().quote);
    }

    #[test]
    fn o_uso_conta_nomes_e_membros_fora_de_strings_e_comentarios() {
        let u = usage([
            "ph2d.set(self.id, \"x\", 1)\nph2d.set(a)\n-- ph2d.set no comentário\nlocal s = \"ph2d.get\"\n--[[ ph2d.get ]]\nself:emit()\n",
        ]);
        assert_eq!(u.get("ph2d.set"), Some(&2));
        assert_eq!(u.get("ph2d.get"), None);
        assert_eq!(u.get("self.id"), Some(&1));
        assert_eq!(u.get("self.emit"), Some(&1));
        assert_eq!(u.get("set"), Some(&2));
    }

    #[test]
    fn com_o_mesmo_casar_manda_a_relevancia_e_depois_o_uso() {
        let it = |l: &str, s: &str, u: u32| Item {
            label: l.into(),
            kind: None,
            detail: None,
            documentation: None,
            sort_text: s.into(),
            filter: None,
            insert: l.into(),
            snippet: false,
            usage: u,
        };
        let items = [
            it("get", "1", 1),
            it("set", "1", 5),
            it("emit", "1", 0),
            it("if", "8", 99),
        ];
        let names = |typed: &str| -> Vec<&str> {
            order(typed, &items)
                .iter()
                .map(|(i, _)| items[*i].label.as_str())
                .collect()
        };
        assert_eq!(names(""), ["set", "get", "emit", "if"]);
        // Escrito, o casar vem primeiro: `e` é prefixo de `emit`, trecho dos outros.
        assert_eq!(names("e"), ["emit", "set", "get"]);
    }
}

/// A tinta do rótulo pelo `kind` do LSP (as cores do realce do próprio código).
fn ink_of_kind(kind: Option<u32>) -> ph2d_panel_code::Ink {
    use ph2d_panel_code::Ink;
    match kind {
        Some(2 | 3 | 4) => Ink::Function,
        Some(5 | 10) => Ink::Property,
        Some(6) => Ink::Variable,
        Some(7 | 8 | 9 | 22 | 25) => Ink::Type,
        Some(14) => Ink::Keyword,
        Some(12 | 20 | 21) => Ink::String,
        _ => Ink::Plain,
    }
}

/// As faixas de bytes do rótulo que casaram (os índices de carácter agrupados em trechos).
fn matched_bytes(label: &str, chars: &[usize]) -> Vec<Range<usize>> {
    let offs: Vec<(usize, usize)> = label
        .char_indices()
        .map(|(b, c)| (b, b + c.len_utf8()))
        .collect();
    let mut out: Vec<Range<usize>> = Vec::new();
    for &i in chars {
        let Some(&(a, b)) = offs.get(i) else { continue };
        match out.last_mut() {
            Some(r) if r.end == a => r.end = b,
            _ => out.push(a..b),
        }
    }
    out
}

/// O que a aba pinta da lista neste quadro.
pub(crate) struct Shown {
    pub anchor: usize,
    pub rows: Vec<ph2d_panel_code::CompletionRow>,
    pub total: usize,
    pub top: usize,
    pub selected: usize,
    pub width_chars: usize,
    pub doc: Option<String>,
}

impl crate::doc::CodeDoc {
    /// A lista aberta deste quadro (só a janela das linhas à vista).
    pub(crate) fn completion_shown(&mut self) -> Option<Shown> {
        let head = self.buffer.selection().primary().head;
        let list = self.complete.shown(self.buffer.text(), head)?;
        let s = self.complete.session.as_ref()?;
        let items = s.items.as_ref()?;
        let rows = ph2d_panel_code::COMPLETION_ROWS;
        let width_chars = ph2d_panel_code::CompletionView::width_of(
            list.iter()
                .map(|(i, _)| (items[*i].label.as_str(), items[*i].detail.as_deref())),
        );
        let window = list
            .iter()
            .skip(s.top)
            .take(rows)
            .map(|(i, m)| {
                let it = &items[*i];
                ph2d_panel_code::CompletionRow {
                    matched: matched_bytes(&it.label, &m.chars),
                    ink: ink_of_kind(it.kind),
                    detail: it.detail.clone(),
                    label: it.label.clone(),
                }
            })
            .collect();
        let sel = &items[list[s.selected].0];
        Some(Shown {
            anchor: s.anchor,
            rows: window,
            total: list.len(),
            top: s.top,
            selected: s.selected,
            width_chars,
            doc: sel.documentation.clone().or_else(|| sel.detail.clone()),
        })
    }

    /// Uma tecla com a lista aberta: `true` = era dela (setas, páginas, Tab/Enter aceitam, Esc fecha).
    pub(crate) fn completion_key(
        &mut self,
        key: ph2d_editor_core::doc_surface::Key,
        mods: ph2d_host::Modifiers,
    ) -> bool {
        use ph2d_editor_core::doc_surface::Key;
        if mods.ctrl || mods.meta || mods.alt {
            return false;
        }
        let head = self.buffer.selection().primary().head;
        let Some(len) = self
            .complete
            .shown(self.buffer.text(), head)
            .map(|l| l.len())
        else {
            return false;
        };
        let rows = ph2d_panel_code::COMPLETION_ROWS;
        match key {
            Key::Up if !mods.shift => self.complete.step(-1, len, rows),
            Key::Down if !mods.shift => self.complete.step(1, len, rows),
            Key::PageUp if !mods.shift => self.complete.step(-(rows as isize), len, rows),
            Key::PageDown if !mods.shift => self.complete.step(rows as isize, len, rows),
            Key::Tab | Key::Enter if !mods.shift => {
                let i = self.complete.session.as_ref().map_or(0, |s| s.selected);
                self.accept_completion(i);
            }
            Key::Escape => self.complete.close(),
            _ => return false,
        }
        true
    }

    /// Aceita a linha `index` da lista mostrada: troca a palavra escrita em CADA cursor que a tem
    /// pela proposta, num passo de undo, com o cursor onde o snippet o põe.
    pub(crate) fn accept_completion(&mut self, index: usize) {
        use ph2d_code_buffer::{Change, ChangeSet, EditKind, Range as Cursor, Selection};
        let head = self.buffer.selection().primary().head;
        let Some(list) = self.complete.shown(self.buffer.text(), head) else {
            return;
        };
        let (Some((i, _)), Some(s)) = (list.get(index), self.complete.session.as_ref()) else {
            return;
        };
        let Some(item) = s.items.as_ref().and_then(|v| v.get(*i)) else {
            return;
        };
        let (insert, caret) = insertion(item);
        let t = self.buffer.text();
        let typed = t.slice(replaced(s, head));
        let n = typed.len();
        let sel = self.buffer.selection();
        let primary = sel.primary();
        let mut spots: Vec<(usize, usize, bool)> = sel
            .ranges()
            .iter()
            .filter(|r| r.is_empty() && r.head >= n && t.slice(r.head - n..r.head) == typed)
            .filter(|r| word_at(t, r.head).is_some_and(|w| w.start == r.head - n))
            .map(|r| (r.head - n, r.head, *r == primary))
            .collect();
        spots.sort_by_key(|s| s.0);
        let changes = spots
            .iter()
            .map(|&(a, b, _)| Change {
                range: a..b,
                insert: insert.clone(),
            })
            .collect();
        let Some(cs) = ChangeSet::new(changes) else {
            return;
        };
        let mut delta = 0isize;
        let mut cursors = Vec::with_capacity(spots.len());
        let mut main = 0;
        for &(a, b, is_primary) in &spots {
            let at = (a as isize + delta) as usize + caret;
            if is_primary {
                main = cursors.len();
            }
            cursors.push(Cursor::cursor(at));
            delta += insert.len() as isize - (b - a) as isize;
        }
        if cursors.is_empty() {
            return;
        }
        let edits = self
            .buffer
            .apply(cs, Some(Selection::new(cursors, main)), EditKind::Other);
        self.edited(&edits, true);
        self.complete.close();
    }
}
