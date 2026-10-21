//! ⭐⭐ **O que um arrasto ESCREVE no script** (item 8, `docs/CodeEditor/07_arrastar_para_o_codigo.md` §3) —
//! a lei pura: o texto da soltura, o nome da variável e a reescrita do renomear. O gesto e o
//! oráculo (o editor do Godot 4.7.2 corrido, `ferramentas/oraculo_arrastar/saida.txt`) estão no doc.

use ph2d_code_buffer::Text;
use ph2d_code_syntax::{LUAU_KEYWORDS, ObjectCall, Syntax};

/// O que a soltura escreve: `text` em `at`, e o cursor fica em `caret` (bytes do texto DEPOIS).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Insertion {
    pub at: usize,
    pub text: String,
    pub caret: usize,
}

/// Uma string Luau entre aspas duplas, com `\` e `"` escapados.
#[must_use]
pub fn luau_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        if matches!(c, '"' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// `ph2d.object("<nome>")` — a referência que a soltura escreve (§3.1).
#[must_use]
pub fn reference(name: &str) -> String {
    format!("ph2d.object({})", luau_string(name))
}

/// O nome da variável do Ctrl: o `snake_case` do nome (o do Godot: `Caixa Grande` → `caixa_grande`,
/// `PlayerSprite` → `player_sprite`), feito identificador Luau — acentos tirados, o resto do que não
/// é letra/dígito vira `_`, dígito inicial com `_` à frente, palavra reservada com `_` atrás.
#[must_use]
pub fn variable_name(name: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for c in name.chars() {
        let c = unaccent(c);
        if c.is_ascii_alphanumeric() {
            if c.is_ascii_uppercase() && prev_lower {
                out.push('_');
            }
            prev_lower = c.is_ascii_lowercase();
            out.push(c.to_ascii_lowercase());
        } else {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            prev_lower = false;
        }
    }
    let mut out = out.trim_end_matches('_').to_owned();
    if out.is_empty() {
        out.push_str("object");
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    if LUAU_KEYWORDS.contains(&out.as_str()) {
        out.push('_');
    }
    out
}

fn unaccent(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        'ç' => 'c',
        'Ç' => 'C',
        'ñ' => 'n',
        'Ñ' => 'N',
        _ => c,
    }
}

/// O recuo (espaços/tabs do início) da linha `line`.
fn indent_of(text: &Text, line: usize) -> String {
    text.line_string(line)
        .chars()
        .take_while(|c| matches!(c, ' ' | '\t'))
        .collect()
}

/// O recuo para escrever na linha `line`: o dela; numa linha EM BRANCO (o formatador tira-lhe os
/// espaços), o maior das vizinhas com texto — a de cima abre o bloco, a de baixo pode ser o `end`.
fn indent_for(text: &Text, line: usize) -> String {
    if !text.line_string(line).trim().is_empty() {
        return indent_of(text, line);
    }
    let filled = |l: &usize| !text.line_string(*l).trim().is_empty();
    let up = (0..line).rev().find(filled).map(|l| indent_of(text, l));
    let down = (line + 1..text.line_count())
        .find(filled)
        .map(|l| indent_of(text, l));
    up.into_iter()
        .chain(down)
        .max_by_key(String::len)
        .unwrap_or_default()
}

/// ⭐ **A soltura de objectos** em `byte` (a posição do cursor fino).
///
/// - Sem modificador: `ph2d.object("A"), ph2d.object("B")` no cursor (§3.1, §3.3).
/// - Com Ctrl, dentro de uma função: um `local <var> = ph2d.object("…")` por objecto, no INÍCIO da
///   linha da soltura e com o recuo dela (o que lá estava desce) — §3.2.
/// - Com Ctrl, no topo: `self.<var> = ph2d.object("…")` no começo do corpo do `init(self)`; sem
///   `init`, um `function init(self) … end` novo na linha da soltura (o topo de um script corre ao
///   carregar, fora da cena: o `init` é o `@onready` do Godot).
///
/// `unit` é um nível de recuo do documento. `names` vem pela ordem da lista de objectos.
#[must_use]
pub fn insertion(
    text: &Text,
    syntax: &Syntax,
    byte: usize,
    names: &[String],
    ctrl: bool,
    unit: &str,
) -> Insertion {
    let byte = byte.min(text.len());
    if !ctrl {
        let s = names
            .iter()
            .map(|n| reference(n))
            .collect::<Vec<_>>()
            .join(", ");
        return Insertion {
            at: byte,
            caret: byte + s.len(),
            text: s,
        };
    }
    let line = text.line_of_byte(byte);
    let own = indent_of(text, line);
    let line_start = text.byte_of_line(line) + own.len();
    if !syntax.at_top_level(text, line_start) {
        let indent = indent_for(text, line);
        let pad = &indent[own.len().min(indent.len())..];
        let decls = names
            .iter()
            .map(|n| format!("local {} = {}", variable_name(n), reference(n)))
            .collect::<Vec<_>>()
            .join(&format!("\n{indent}"));
        let s = format!("{pad}{decls}\n{indent}");
        return Insertion {
            at: line_start,
            caret: line_start + pad.len() + decls.len(),
            text: s,
        };
    }
    let fields = |pad: &str| {
        names
            .iter()
            .map(|n| format!("{pad}self.{} = {}", variable_name(n), reference(n)))
            .collect::<Vec<_>>()
            .join("\n")
    };
    if let Some(body) = syntax.top_function_body(text, "init") {
        let body_line = text.line_of_byte(body);
        let pad = if body < text.len() && !indent_of(text, body_line).is_empty() {
            indent_of(text, body_line)
        } else {
            unit.to_owned()
        };
        let decls = fields(&pad);
        return Insertion {
            at: body,
            caret: body + decls.len(),
            text: format!("{decls}\n"),
        };
    }
    let at = text.byte_of_line(line);
    let head = "function init(self)\n";
    let decls = fields(unit);
    Insertion {
        at,
        caret: at + head.len() + decls.len(),
        text: format!("{head}{decls}\nend\n"),
    }
}

/// O texto `src` depois da soltura em `byte` — a lei inteira sem aba (as cenas de smoke e os testes
/// que correm o resultado).
#[must_use]
pub fn dropped_into(src: &str, byte: usize, names: &[String], ctrl: bool, unit: &str) -> String {
    let t = Text::new(src);
    let i = insertion(&t, &Syntax::new(&t), byte, names, ctrl, unit);
    let mut out = src.to_owned();
    out.insert_str(i.at, &i.text);
    out
}

/// ⭐ **O renomear reescreve** (§3.5): os bytes (dentro das aspas) a trocar por `new`, do fim para o
/// início. Um objecto da cena muda os `ph2d.object`/`ph2d.find_by_name`; um MESTRE muda os `ph2d.spawn`
/// — cada função só procura o que a sua porta vê (o `find` salta mestres).
#[must_use]
pub fn rename_edits(
    text: &Text,
    syntax: &Syntax,
    old: &str,
    new: &str,
    master: bool,
) -> Vec<(std::ops::Range<usize>, String)> {
    let inner = {
        let s = luau_string(new);
        s[1..s.len() - 1].to_owned()
    };
    let mut out: Vec<_> = syntax
        .object_references(text)
        .into_iter()
        .filter(|r| r.name == old)
        .filter(|r| match r.call {
            ObjectCall::Object | ObjectCall::FindByName => !master,
            ObjectCall::Spawn => master,
        })
        .map(|r| (r.range, inner.clone()))
        .collect();
    out.sort_by_key(|(r, _)| std::cmp::Reverse(r.start));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(src: &str, needle: &str) -> usize {
        src.find(needle).expect("agulha")
    }

    fn apply(src: &str, i: &Insertion) -> String {
        let mut s = src.to_owned();
        s.insert_str(i.at, &i.text);
        s
    }

    #[test]
    fn o_nome_da_variavel_e_o_snake_case_do_godot_feito_identificador() {
        for (nome, var) in [
            ("Porta", "porta"),
            ("Caixa Grande", "caixa_grande"),
            ("PlayerSprite", "player_sprite"),
            ("Portão", "portao"),
            ("Inimigo 2", "inimigo_2"),
            ("3D", "_3d"),
            ("End", "end_"),
            ("!!!", "object"),
            ("bala-azul", "bala_azul"),
        ] {
            assert_eq!(variable_name(nome), var, "{nome}");
            assert!(
                ph2d_code_syntax::is_identifier(&variable_name(nome)),
                "{nome}"
            );
        }
    }

    #[test]
    fn sem_modificador_a_referencia_no_cursor_e_varios_com_virgula() {
        let src = "function update(self)\n  ph2d.set(, \"x\", 1)\nend\n";
        let t = Text::new(src);
        let s = Syntax::new(&t);
        let b = at(src, ", \"x\"");
        let i = insertion(&t, &s, b, &["Porta".into()], false, "  ");
        assert_eq!(
            apply(src, &i),
            "function update(self)\n  ph2d.set(ph2d.object(\"Porta\"), \"x\", 1)\nend\n"
        );
        assert_eq!(i.caret, b + "ph2d.object(\"Porta\")".len());
        let dois = insertion(
            &t,
            &s,
            b,
            &["Porta".into(), "Diz \"oi\"".into()],
            false,
            "  ",
        );
        assert_eq!(
            dois.text,
            "ph2d.object(\"Porta\"), ph2d.object(\"Diz \\\"oi\\\"\")"
        );
    }

    #[test]
    fn com_ctrl_dentro_da_funcao_um_local_na_linha_com_o_recuo_dela() {
        let src = "function update(self)\n  ph2d.set(self.id, \"x\", 1)\nend\n";
        let t = Text::new(src);
        let s = Syntax::new(&t);
        let i = insertion(
            &t,
            &s,
            at(src, "self.id"),
            &["Porta".into(), "Caixa Grande".into()],
            true,
            "  ",
        );
        let got = apply(src, &i);
        assert_eq!(
            got,
            "function update(self)\n  local porta = ph2d.object(\"Porta\")\n  local caixa_grande = ph2d.object(\"Caixa Grande\")\n  ph2d.set(self.id, \"x\", 1)\nend\n"
        );
        assert_eq!(
            &got[..i.caret],
            &got[..at(&got, "\n  ph2d.set")],
            "o cursor no fim da última declaração"
        );
    }

    #[test]
    fn com_ctrl_numa_linha_em_branco_o_recuo_vem_das_vizinhas() {
        let src = "function update(self)\n  x = 1\n\n  y = 2\nend\n";
        let t = Text::new(src);
        let s = Syntax::new(&t);
        let i = insertion(&t, &s, at(src, "\n\n") + 1, &["Door".into()], true, "\t");
        let got = apply(src, &i);
        assert_eq!(
            got,
            "function update(self)\n  x = 1\n  local door = ph2d.object(\"Door\")\n  \n  y = 2\nend\n"
        );
        assert!(got[..i.caret].ends_with("ph2d.object(\"Door\")"));
        let antes_do_end = "function update(self)\n  x = 1\n\nend\n";
        let t = Text::new(antes_do_end);
        let i = insertion(
            &t,
            &Syntax::new(&t),
            at(antes_do_end, "\n\n") + 1,
            &["D".into()],
            true,
            "\t",
        );
        assert!(
            apply(antes_do_end, &i).contains("\n  local d = "),
            "a de cima ganha ao `end`"
        );
    }

    #[test]
    fn com_ctrl_no_topo_vai_para_self_no_init_que_existe() {
        let src = "local VEL = 2\n\nfunction init(self)\n\tself.vida = 3\nend\n";
        let t = Text::new(src);
        let s = Syntax::new(&t);
        let i = insertion(
            &t,
            &s,
            at(src, "\n\nfunction") + 1,
            &["Porta".into()],
            true,
            "  ",
        );
        assert_eq!(
            apply(src, &i),
            "local VEL = 2\n\nfunction init(self)\n\tself.porta = ph2d.object(\"Porta\")\n\tself.vida = 3\nend\n",
            "o recuo é o do corpo, não o unit"
        );
    }

    #[test]
    fn com_ctrl_no_topo_sem_init_nasce_um_init() {
        let src = "local VEL = 2\n\nfunction update(self)\nend\n";
        let t = Text::new(src);
        let s = Syntax::new(&t);
        let i = insertion(
            &t,
            &s,
            at(src, "\nfunction") + 1,
            &["Porta".into()],
            true,
            "\t",
        );
        let got = apply(src, &i);
        assert_eq!(
            got,
            "local VEL = 2\n\nfunction init(self)\n\tself.porta = ph2d.object(\"Porta\")\nend\nfunction update(self)\nend\n"
        );
        assert!(got[..i.caret].ends_with("ph2d.object(\"Porta\")"));
        assert!(
            Syntax::new(&Text::new(&got)).errors().is_empty(),
            "o script continua válido"
        );
    }

    #[test]
    fn renomear_reescreve_so_o_que_a_porta_ve() {
        let src = "local p = ph2d.object(\"Porta\")\nlocal q = ph2d.find_by_name('Porta')\nph2d.spawn(\"Porta\")\nlocal x = \"Porta\"\nph2d.object(\"Portas\")\n";
        let t = Text::new(src);
        let s = Syntax::new(&t);
        let mut objecto = src.to_owned();
        for (r, novo) in rename_edits(&t, &s, "Porta", "Portão \"A\"", false) {
            objecto.replace_range(r, &novo);
        }
        assert_eq!(
            objecto,
            "local p = ph2d.object(\"Portão \\\"A\\\"\")\nlocal q = ph2d.find_by_name('Portão \\\"A\\\"')\nph2d.spawn(\"Porta\")\nlocal x = \"Porta\"\nph2d.object(\"Portas\")\n",
            "um texto solto e um nome que só COMEÇA igual ficam"
        );
        let mestre = rename_edits(&t, &s, "Porta", "Bala", true);
        assert_eq!(mestre.len(), 1, "um mestre só muda o spawn");
        assert_eq!(&src[mestre[0].0.clone()], "Porta");
        assert!(mestre[0].0.start > at(src, "ph2d.spawn"));
    }
}
