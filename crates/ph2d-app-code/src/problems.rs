//! ⭐ **E7 + I3 — os problemas da aba**: os erros vivos do tree-sitter, o erro que o Luau deu ao texto
//! salvo (com a LINHA) e os diagnósticos do assistente (`luau-lsp`) enquanto se escreve, sem duplicar.
//! Um erro de sintaxe sai UMA vez: o lugar exacto é o da árvore (conferido contra o `luau-lsp` em
//! `tests/it/oraculo_lsp.rs`), a mensagem é a do Luau (`docs/CodeEditor/03_onda2.md` I3).

use ph2d_code_buffer::{Text, TextEdit};
use ph2d_code_intel::Diag;
use ph2d_code_syntax::{HighlightKind, SyntaxError};
use ph2d_i18n::{tr, tr_with};
use ph2d_panel_code::{Ink, Problem, Severity};
use ph2d_script::ScriptDiagnostic;
use ph2d_script::api_table::HOOKS;
use std::ops::Range;

/// O `code` do Luau para um erro de SINTAXE (I0c; a mensagem começa por `SyntaxError:`).
const SYNTAX: &str = "1014";
/// O lint `FunctionUnused`.
const FUNCTION_UNUSED: &str = "8";

/// Os diagnósticos do assistente que a aba SEGURA: os da última resposta, deslocados pelas edições
/// feitas depois dela. Cada tecla avança a versão do servidor; pintar só os da versão corrente
/// fá-los-ia piscar a cada tecla.
#[derive(Default)]
pub(crate) struct Held {
    /// A versão do servidor a que a lista responde (`None` = nenhuma resposta).
    pub version: Option<u64>,
    /// Nenhuma edição desde a resposta: o texto é o que o servidor analisou.
    pub exact: bool,
    pub list: Vec<Diag>,
}

impl Held {
    /// A resposta para `version`, dada sobre o texto de antes de `since` (as edições ainda por enviar).
    pub fn adopt(&mut self, version: u64, list: &[Diag], since: &[TextEdit]) {
        self.version = Some(version);
        list.clone_into(&mut self.list);
        self.exact = true;
        self.edited(since);
    }

    pub fn edited(&mut self, edits: &[TextEdit]) {
        if edits.is_empty() {
            return;
        }
        self.exact = false;
        for e in edits {
            self.list.retain_mut(|d| match shift(&d.bytes, e) {
                Some(r) => {
                    d.bytes = r;
                    true
                }
                None => false,
            });
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Uma faixa do texto de antes de `e`, no texto de depois; `None` = a edição apagou-a toda. Escrever
/// logo a seguir ao fim não a estica; escrever dentro, sim.
fn shift(r: &Range<usize>, e: &TextEdit) -> Option<Range<usize>> {
    let moved = |b: usize| b - e.old_end_byte + e.new_end_byte;
    if e.old_end_byte <= r.start {
        Some(moved(r.start)..moved(r.end))
    } else if e.start_byte >= r.end {
        Some(r.clone())
    } else if e.start_byte <= r.start && r.end <= e.old_end_byte {
        None
    } else {
        let start = if r.start < e.start_byte {
            r.start
        } else {
            e.new_end_byte
        };
        let end = if r.end > e.old_end_byte {
            moved(r.end)
        } else {
            e.start_byte
        };
        Some(start..end.max(start))
    }
}

/// Um gancho que o MOTOR chama, declarado global (`function init(self)`): o lint «nunca usada» do
/// servidor não sabe do motor (I0b §2, medido em todo script). Uma `local function init` não é
/// gancho e o aviso fica.
fn engine_hook(text: &Text, d: &Diag, bytes: &Range<usize>) -> bool {
    if d.code.as_deref() != Some(FUNCTION_UNUSED) {
        return false;
    }
    let name = text.slice(bytes.clone());
    let line = text.byte_of_line(text.line_of_byte(bytes.start));
    HOOKS.iter().any(|h| h.ident == name) && text.slice(line..bytes.start).trim() == "function"
}

/// As linhas (0-based) que uma mensagem do Luau cita: «… (to close 'function' at line 29) …; did you
/// forget to close 'then' at line 47?» (1-based no texto, I0b §2 e2).
fn cited_lines(message: &str) -> impl Iterator<Item = usize> + '_ {
    message.match_indices("at line ").filter_map(move |(i, m)| {
        let rest = &message[i + m.len()..];
        let n = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        rest[..n].parse::<usize>().ok()?.checked_sub(1)
    })
}

fn severity_of(lsp: u8) -> Severity {
    match lsp {
        1 => Severity::Error,
        2 => Severity::Warning,
        _ => Severity::Info,
    }
}

/// A tinta de cada tipo de realce (16 → 14: chamada = função, booleano = constante, atributo = palavra
/// reservada).
#[must_use]
pub fn ink_of(k: HighlightKind) -> Ink {
    match k {
        HighlightKind::Keyword | HighlightKind::Attribute => Ink::Keyword,
        HighlightKind::Function | HighlightKind::FunctionCall => Ink::Function,
        HighlightKind::Variable => Ink::Variable,
        HighlightKind::Parameter => Ink::Parameter,
        HighlightKind::Property => Ink::Property,
        HighlightKind::Type => Ink::Type,
        HighlightKind::String => Ink::String,
        HighlightKind::Number => Ink::Number,
        HighlightKind::Boolean | HighlightKind::Constant => Ink::Constant,
        HighlightKind::Builtin => Ink::Builtin,
        HighlightKind::Comment => Ink::Comment,
        HighlightKind::Operator => Ink::Operator,
        HighlightKind::Punctuation => Ink::Punctuation,
    }
}

/// O culpado da árvore numa linha (0-based): o primeiro erro que começa nela.
#[must_use]
pub fn culprit_on_line(
    text: &Text,
    errors: &[SyntaxError],
    line: usize,
) -> Option<std::ops::Range<usize>> {
    errors
        .iter()
        .find(|e| text.line_of_byte(e.range.start) == line)
        .map(|e| e.range.clone())
}

/// Os problemas da aba. `saved` = o diagnóstico do Luau para o texto salvo e a linha (0-based) onde
/// ele está AGORA no buffer — quem chama segue-a pelas edições, ou não o passa (ver
/// `CodeDoc::problems`). `assist` = os diagnósticos do assistente que a aba segura.
///
/// Fusão, por linha: um erro de sintaxe do Luau (`1014`) dá a MENSAGEM ao do tree-sitter da mesma
/// linha, ou de uma linha que ele cita (o primeiro que chega fica); sem par vivo, só sai se o texto for o que o servidor analisou
/// (`exact` — senão é o erro que a tecla acabou de corrigir). Numa linha com erro de sintaxe, o resto
/// do que o assistente diz é eco do parser (`SameLineStatement`, o global de um `if x = 1`) e não sai.
#[must_use]
pub(crate) fn problems(
    text: &Text,
    errors: &[SyntaxError],
    saved: Option<(&ScriptDiagnostic, usize)>,
    assist: &Held,
) -> Vec<Problem> {
    struct Row {
        p: Problem,
        syntax: bool,
        /// A mensagem já é a do Luau.
        luau: bool,
    }
    let mut rows: Vec<Row> = errors
        .iter()
        .map(|e| Row {
            p: Problem {
                line: text.line_of_byte(e.range.start),
                severity: Severity::Error,
                culprit: Some(e.range.clone()),
                message: match &e.missing {
                    Some(sym) => tr_with("code.problem.missing", &[("sym", sym)]),
                    None => tr("code.problem.syntax").to_owned(),
                },
            },
            syntax: true,
            luau: false,
        })
        .collect();
    // Um erro por linha: o do tree-sitter que começa primeiro.
    rows.dedup_by_key(|r| r.p.line);
    if let Some((d, line)) = saved {
        let line = line.min(text.line_count().saturating_sub(1));
        match rows.iter_mut().find(|r| r.p.line == line) {
            Some(r) => {
                d.message.clone_into(&mut r.p.message);
                r.luau = true;
            }
            None => rows.push(Row {
                p: Problem {
                    line,
                    severity: Severity::Error,
                    culprit: culprit_on_line(text, errors, line),
                    message: d.message.clone(),
                },
                syntax: false,
                luau: true,
            }),
        }
    }
    let clamp = |r: &Range<usize>| {
        let a = text.floor_char(r.start.min(text.len()));
        a..text.ceil_char(r.end.clamp(a, text.len()))
    };
    let (syntax, other): (Vec<&Diag>, Vec<&Diag>) = assist
        .list
        .iter()
        .partition(|d| d.code.as_deref() == Some(SYNTAX));
    for d in syntax {
        let bytes = clamp(&d.bytes);
        let line = text.line_of_byte(bytes.start);
        // O bloco por fechar: o Luau aponta o fim do ficheiro e CITA a linha que o abriu; o
        // tree-sitter marca essa linha.
        let mut lines = std::iter::once(line).chain(cited_lines(&d.message));
        let pair = lines.find_map(|l| rows.iter().position(|r| r.syntax && r.p.line == l));
        match pair.map(|i| &mut rows[i]) {
            Some(r) if !r.luau => {
                d.message.clone_into(&mut r.p.message);
                r.luau = true;
            }
            Some(_) => {}
            None if assist.exact => rows.push(Row {
                p: Problem {
                    line,
                    severity: Severity::Error,
                    culprit: Some(bytes),
                    message: d.message.clone(),
                },
                syntax: true,
                luau: true,
            }),
            None => {}
        }
    }
    for d in other {
        let bytes = clamp(&d.bytes);
        let line = text.line_of_byte(bytes.start);
        if rows.iter().any(|r| r.syntax && r.p.line == line) || engine_hook(text, d, &bytes) {
            continue;
        }
        rows.push(Row {
            p: Problem {
                line,
                severity: severity_of(d.severity),
                culprit: Some(bytes),
                message: d.message.clone(),
            },
            syntax: false,
            luau: true,
        });
    }
    let mut out: Vec<Problem> = rows.into_iter().map(|r| r.p).collect();
    out.sort_by_key(|p| (p.line, p.culprit.as_ref().map(|c| c.start)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cada tipo de realce tem tinta — um tipo novo sem braço não compila; e os três que juntam vão
    /// para onde a pintura os espera.
    #[test]
    fn todo_tipo_de_realce_tem_tinta() {
        for k in HighlightKind::ALL {
            let _ = ink_of(k);
        }
        assert_eq!(ink_of(HighlightKind::FunctionCall), Ink::Function);
        assert_eq!(ink_of(HighlightKind::Boolean), Ink::Constant);
        assert_eq!(ink_of(HighlightKind::Attribute), Ink::Keyword);
        assert_eq!(ink_of(HighlightKind::String), Ink::String);
    }

    fn edit(start: usize, old_end: usize, new_end: usize) -> TextEdit {
        TextEdit {
            start_byte: start,
            old_end_byte: old_end,
            new_end_byte: new_end,
            start: ph2d_code_buffer::Point::default(),
            old_end: ph2d_code_buffer::Point::default(),
            new_end: ph2d_code_buffer::Point::default(),
        }
    }

    fn diag(bytes: Range<usize>, code: &str, severity: u8, message: &str) -> Diag {
        Diag {
            bytes,
            severity,
            code: Some(code.to_owned()),
            message: message.to_owned(),
        }
    }

    /// ⭐ Um diagnóstico seguro segue o texto: uma edição que não lhe toca deixa-o sobre os MESMOS
    /// caracteres; uma que o apaga todo leva-o; uma que entra nele mantém-no sobre o que sobrou mais
    /// o que entrou. 3 000 edições ao acaso sobre `abc pirnt def`.
    #[test]
    fn um_diagnostico_seguro_segue_as_edicoes() {
        let mut rng = fastrand::Rng::with_seed(0x13);
        for _ in 0..3_000 {
            let mut s = String::from("abc pirnt def");
            let r = 4..9;
            let a = rng.usize(0..=s.len());
            let b = rng.usize(a..=s.len());
            let ins: String = (0..rng.usize(0..4)).map(|_| rng.alphabetic()).collect();
            let e = edit(a, b, a + ins.len());
            let antes = s[r.clone()].to_owned();
            s.replace_range(a..b, &ins);
            match shift(&r, &e) {
                None => assert!(a <= r.start && r.end <= b, "{a}..{b} não cobria {r:?}"),
                Some(n) => {
                    if b <= r.start || a >= r.end {
                        assert_eq!(s[n.clone()], antes, "{a}..{b} «{ins}» moveu o alvo");
                    } else {
                        assert!(n.start <= n.end && n.end <= s.len());
                        if r.start < a && b < r.end {
                            assert!(s[n.clone()].contains(&ins), "{a}..{b}: {:?}", &s[n]);
                        }
                    }
                }
            }
        }
    }

    /// As respostas do `luau-lsp` 1.70.1 fixado a `if x = 1 then x = 2 end` (sonda de 16/10: dois
    /// `1014`, o global `x` e o lint `SameLineStatement`, todos na linha 0).
    fn if_igual() -> (Text, Vec<Diag>) {
        let t = Text::new("if x = 1 then x = 2 end\n");
        let list = vec![
            diag(
                5..6,
                SYNTAX,
                1,
                "SyntaxError: Expected 'then' when parsing if statement, got '='",
            ),
            diag(
                9..13,
                SYNTAX,
                1,
                "SyntaxError: Expected identifier when parsing expression, got 'then'",
            ),
            diag(
                3..4,
                "1001",
                1,
                "TypeError: Unknown global 'x'; consider assigning to it first",
            ),
            diag(
                9..13,
                "5",
                2,
                "SameLineStatement: A new statement is on the same line",
            ),
        ];
        (t, list)
    }

    fn com(list: Vec<Diag>, exact: bool) -> Held {
        Held {
            version: Some(1),
            exact,
            list,
        }
    }

    /// ⭐ I3: o erro de sintaxe sai UMA vez — no lugar do tree-sitter, com a mensagem do 1.º `1014` do
    /// Luau — e o eco do parser na mesma linha não sai.
    #[test]
    fn o_erro_de_sintaxe_sai_uma_vez_com_a_mensagem_do_luau() {
        let (t, list) = if_igual();
        let errors = ph2d_code_syntax::Syntax::new(&t).errors();
        let arvore = culprit_on_line(&t, &errors, 0).expect("o tree-sitter vê o erro");
        for exact in [true, false] {
            let ps = problems(&t, &errors, None, &com(list.clone(), exact));
            assert_eq!(ps.len(), 1, "exact={exact}: {ps:#?}");
            assert_eq!(ps[0].culprit, Some(arvore.clone()));
            assert_eq!(
                ps[0].message,
                "SyntaxError: Expected 'then' when parsing if statement, got '='"
            );
        }
        // Controlo: sem o assistente, a mesma linha diz o texto do tree-sitter.
        let ps = problems(&t, &errors, None, &Held::default());
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].message, tr("code.problem.syntax"));
    }

    /// Falta um `end`: o Luau aponta o fim do ficheiro (faixa vazia, I0b §2 e2), e é o mesmo erro que
    /// o tree-sitter vê.
    #[test]
    fn o_end_em_falta_sai_uma_vez() {
        let t = Text::new("function update(self, dt)\n  if dt then\n    print(1)\nend\n");
        let errors = ph2d_code_syntax::Syntax::new(&t).errors();
        assert!(!errors.is_empty(), "o tree-sitter vê o end em falta");
        let held = com(
            vec![diag(
                t.len()..t.len(),
                SYNTAX,
                1,
                "SyntaxError: Expected 'end' (to close 'function' at line 1), got <eof>",
            )],
            true,
        );
        let ps = problems(&t, &errors, None, &held);
        assert_eq!(ps.len(), 1, "{errors:?} {ps:#?}");
        assert!(ps[0].message.starts_with("SyntaxError: Expected 'end'"));
    }

    /// Um `1014` sem par vivo na árvore sai só se o texto for o que o servidor analisou: depois de
    /// uma tecla, é o erro que ela acabou de corrigir.
    #[test]
    fn um_erro_de_sintaxe_velho_sem_par_na_arvore_nao_sai() {
        let t = Text::new("local a = 1\n");
        let errors = ph2d_code_syntax::Syntax::new(&t).errors();
        assert!(errors.is_empty());
        let l = vec![diag(6..7, SYNTAX, 1, "SyntaxError: x")];
        assert_eq!(problems(&t, &errors, None, &com(l.clone(), true)).len(), 1);
        assert!(problems(&t, &errors, None, &com(l, false)).is_empty());
    }

    /// Os ganchos globais que o motor chama não são «nunca usados»; uma `local function init` e um
    /// gancho com o nome errado são, e o aviso fica (com a gravidade do LSP).
    #[test]
    fn um_gancho_do_motor_nao_e_funcao_nunca_usada() {
        let t = Text::new(
            "function init(self)\nend\nlocal function update(self)\nend\nfunction updtae(self)\nend\n",
        );
        let at = |s: &str, nth: usize| t.slice(0..t.len()).match_indices(s).nth(nth).unwrap().0;
        let unused = |name: &str, at: usize| {
            diag(
                at..at + name.len(),
                FUNCTION_UNUSED,
                2,
                &format!("FunctionUnused: Function '{name}' is never used"),
            )
        };
        let list = vec![
            unused("init", at("init", 0)),
            unused("update", at("update", 0)),
            unused("updtae", at("updtae", 0)),
        ];
        let ps = problems(&t, &[], None, &com(list, true));
        let lines: Vec<(usize, Severity)> = ps.iter().map(|p| (p.line, p.severity)).collect();
        assert_eq!(lines, [(2, Severity::Warning), (4, Severity::Warning)]);
    }
}
