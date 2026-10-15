//! E8 — fechamento automático e auto-indent contra o ORÁCULO Godot 4.7.2 (`CodeEdit` headless).
//! A fixtura é a saída gravada do Godot sobre entradas nossas (`docs/CodeEditor/ferramentas/
//! godot_oraculo/`); cada caso corre pela nossa lei e o texto + cursores têm de ser iguais.

use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_code_syntax::{AutoEdit, IndentStyle, Syntax};

const FIXTURA: &str = include_str!("fixtures/godot_autoedit.txt");

/// Divergências DE PROPÓSITO: (caso, a nossa saída, porquê).
const DIVERGE: &[(&str, &str, &str)] = &[
    (
        "end_luau_recua",
        "if x then\\n\\tfoo()\\nend|",
        "o Luau fecha blocos com `end`; o GDScript não tem fecho, o Godot não recua nada",
    ),
    (
        "else_luau_recua",
        "if x then\\n\\tfoo()\\nelse|",
        "`else` ramifica o bloco Luau e volta ao nível do `if` (no GDScript o `else:` é escrito já no nível)",
    ),
    (
        "chaveta_fecha_recua_luau",
        "x = {\\n\\t1,\\n}|",
        "`}` sozinho no início da linha fecha a tabela e volta ao nível de quem a abriu",
    ),
];

struct Case {
    name: String,
    keys: Vec<String>,
    text: String,
    indent: IndentStyle,
    maps: Vec<(String, String)>,
    godot: String,
}

fn unescape(s: &str) -> String {
    let mut o = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => o.push('\n'),
                Some('t') => o.push('\t'),
                Some(x) => o.push(x),
                None => o.push('\\'),
            }
        } else {
            o.push(c);
        }
    }
    o
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

fn cases() -> Vec<Case> {
    let mut out: Vec<Case> = Vec::new();
    for line in FIXTURA.lines() {
        if let Some(n) = line.strip_prefix("== ") {
            out.push(Case {
                name: n.into(),
                keys: Vec::new(),
                text: String::new(),
                indent: IndentStyle::Tabs,
                maps: Vec::new(),
                godot: String::new(),
            });
            continue;
        }
        let Some(c) = out.last_mut() else { continue };
        if let Some(k) = line.strip_prefix("keys: ") {
            c.keys = k
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        } else if let Some(t) = line.strip_prefix("text: ") {
            c.text = unescape(t);
        } else if let Some(i) = line.strip_prefix("indent: ") {
            c.indent = if i == "tab" {
                IndentStyle::Tabs
            } else {
                IndentStyle::Spaces(i.parse().expect("indent: n"))
            };
        } else if let Some(m) = line.strip_prefix("map: ") {
            let (a, b) = m.split_once("=>").expect("map: de=>para");
            c.maps.push((unescape(a), unescape(b)));
        } else if let Some(g) = line.strip_prefix("godot: ") {
            c.godot = unescape(g);
        }
    }
    out
}

fn map(s: &str, maps: &[(String, String)]) -> String {
    maps.iter()
        .fold(s.to_string(), |acc, (a, b)| acc.replace(a.as_str(), b))
}

/// Texto com `|` (cursor) e `^` (âncora) → buffer com essa seleção.
fn buffer_of(marked: &str) -> Buffer {
    let mut text = String::new();
    let mut ranges = Vec::new();
    let mut anchor = None;
    for c in marked.chars() {
        match c {
            '|' => ranges.push(Range::new(anchor.take().unwrap_or(text.len()), text.len())),
            '^' => anchor = Some(text.len()),
            _ => text.push(c),
        }
    }
    let mut b = Buffer::new(&text);
    b.set_selection(Selection::new(ranges, 0));
    b
}

fn marked(b: &Buffer) -> String {
    let s = b.to_string();
    let mut out = String::new();
    for (i, ch) in s
        .char_indices()
        .map(|(i, c)| (i, Some(c)))
        .chain([(s.len(), None)])
    {
        for r in b.selection().ranges() {
            if !r.is_empty() && r.anchor == i {
                out.push('^');
            }
        }
        for r in b.selection().ranges() {
            if r.head == i {
                out.push('|');
            }
        }
        if let Some(c) = ch {
            out.push(c);
        }
    }
    out
}

/// Corre as teclas do caso pela nossa lei; devolve o texto marcado.
fn run(c: &Case, close_pairs: bool) -> String {
    let mut buf = buffer_of(&map(&c.text, &c.maps));
    let mut syn = Syntax::new(buf.text());
    let ae = AutoEdit {
        indent: c.indent,
        close_pairs,
    };
    for k in &c.keys {
        match k.as_str() {
            "ENTER" => {
                ae.on_enter(&mut buf, &mut syn);
            }
            "BS" => {
                ae.on_backspace(&mut buf, &mut syn);
            }
            "SPACE" => {
                ae.on_char(&mut buf, &mut syn, ' ');
            }
            "UNDO" => {
                let e = buf.undo();
                syn.edit(&e, buf.text());
            }
            ch => {
                let mut it = ch.chars();
                let (Some(ch), None) = (it.next(), it.next()) else {
                    panic!("tecla desconhecida {k:?}")
                };
                ae.on_char(&mut buf, &mut syn, ch);
            }
        }
    }
    marked(&buf)
}

/// Os casos em que a nossa lei difere do Godot (fora da lista `DIVERGE`): (caso, Godot, nós).
fn mismatches(close_pairs: bool) -> Vec<(String, String, String)> {
    cases()
        .into_iter()
        .filter(|c| !DIVERGE.iter().any(|d| d.0 == c.name))
        .filter_map(|c| {
            let want = map(&c.godot, &c.maps);
            let got = run(&c, close_pairs);
            (got != want).then(|| (c.name, escape(&want), escape(&got)))
        })
        .collect()
}

#[test]
fn cada_caso_do_godot() {
    let all = cases();
    assert!(
        all.len() >= 80,
        "a fixtura tem {} casos — foi truncada?",
        all.len()
    );
    assert!(
        all.iter()
            .all(|c| !c.keys.is_empty() && !c.godot.is_empty()),
        "caso sem teclas ou sem saída"
    );
    let bad = mismatches(true);
    assert!(
        bad.is_empty(),
        "{} casos divergem do Godot:\n{}",
        bad.len(),
        bad.iter()
            .map(|(n, g, o)| format!("  {n}\n    godot: {g}\n    nós:   {o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn as_divergencias_de_proposito_continuam_a_divergir() {
    let all = cases();
    for (name, ours, why) in DIVERGE {
        let c = all
            .iter()
            .find(|c| c.name == *name)
            .unwrap_or_else(|| panic!("{name} não está na fixtura"));
        let got = escape(&run(c, true));
        assert_eq!(got, *ours, "{name}: {why}");
        assert_ne!(
            got,
            escape(&map(&c.godot, &c.maps)),
            "{name} já não diverge — tire-o da lista"
        );
    }
}

/// Controlo de cegueira: com o fecho automático desligado o mesmo comparador TEM de reprovar os
/// casos de par (senão o gate não vê o que diz ver).
#[test]
fn controle_sem_pares_o_gate_reprova() {
    let bad = mismatches(false);
    for want in [
        "par_parentese",
        "par_aspas_simples",
        "fecho_que_nao_foi_automatico",
        "backspace_par_vazio_carregado",
        "selecao_parentese",
        "multicursor_par",
    ] {
        assert!(
            bad.iter().any(|b| b.0 == want),
            "{want} devia reprovar sem pares; reprovaram {:?}",
            bad.iter().map(|b| &b.0).collect::<Vec<_>>()
        );
    }
}

fn typed(ae: &AutoEdit, buf: &mut Buffer, syn: &mut Syntax, s: &str) {
    for ch in s.chars() {
        ae.on_char(buf, syn, ch);
    }
}

#[test]
fn o_par_e_o_fecho_automatico_sao_um_passo_de_undo() {
    let ae = AutoEdit::new(IndentStyle::Tabs);
    let mut buf = buffer_of("x = |");
    let mut syn = Syntax::new(buf.text());
    ae.on_char(&mut buf, &mut syn, '(');
    assert_eq!(marked(&buf), "x = (|)");
    let before = buf.history().undo_len();
    buf.undo();
    assert_eq!(marked(&buf), "x = |");
    assert_eq!(before, 1);
}

#[test]
fn o_recuo_do_end_junta_se_ao_passo_da_palavra() {
    let ae = AutoEdit::new(IndentStyle::Spaces(2));
    let mut buf = buffer_of("if x then\n  foo()\n  |");
    let mut syn = Syntax::new(buf.text());
    buf.seal_history();
    typed(&ae, &mut buf, &mut syn, "end");
    assert_eq!(marked(&buf), "if x then\n  foo()\nend|");
    let e = buf.undo();
    syn.edit(&e, buf.text());
    assert_eq!(
        marked(&buf),
        "if x then\n  foo()\n  |",
        "um Ctrl+Z tira a palavra E o recuo"
    );
    // `end` → `endx`: deixou de fechar o bloco, a linha volta ao nível do corpo.
    let mut buf = buffer_of("if x then\n  foo()\n  |");
    let mut syn = Syntax::new(buf.text());
    typed(&ae, &mut buf, &mut syn, "endx");
    assert_eq!(marked(&buf), "if x then\n  foo()\n  endx|");
}

#[test]
fn backspace_logo_depois_do_par_tira_o_par_inteiro() {
    let ae = AutoEdit::new(IndentStyle::Tabs);
    let mut buf = buffer_of("f|\ng|");
    let mut syn = Syntax::new(buf.text());
    ae.on_char(&mut buf, &mut syn, '{');
    assert_eq!(marked(&buf), "f{|}\ng{|}");
    ae.on_backspace(&mut buf, &mut syn);
    assert_eq!(marked(&buf), "f|\ng|");
}

#[test]
fn enter_multicursor_e_um_passo_e_a_arvore_segue() {
    let ae = AutoEdit::new(IndentStyle::Tabs);
    let mut buf = buffer_of("function f()|\nend\nfunction g()|\nend");
    let mut syn = Syntax::new(buf.text());
    let n = buf.history().undo_len();
    ae.on_enter(&mut buf, &mut syn);
    assert_eq!(
        marked(&buf),
        "function f()\n\t|\nend\nfunction g()\n\t|\nend"
    );
    assert_eq!(buf.history().undo_len(), n + 1);
    assert_eq!(
        syn.sexp(),
        Syntax::new(buf.text()).sexp(),
        "a árvore incremental = parse do zero"
    );
}

#[test]
fn enter_antes_do_end_na_mesma_linha_abre_o_corpo() {
    let ae = AutoEdit::new(IndentStyle::Spaces(2));
    let mut buf = buffer_of("local g = function()|end");
    let mut syn = Syntax::new(buf.text());
    ae.on_enter(&mut buf, &mut syn);
    assert_eq!(marked(&buf), "local g = function()\n  |\nend");
}
