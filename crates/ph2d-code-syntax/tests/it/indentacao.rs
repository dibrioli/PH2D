//! Indentação: o Enter depois de quem abre um bloco põe +1; dentro do bloco mantém; `end`,
//! `else`, `until`, `}` no início da linha recuam. Texto MEIO ESCRITO (sem o `end`) é o caso normal.

use crate::{BOB, line_of};
use ph2d_code_buffer::Text;
use ph2d_code_syntax::Syntax;

/// O nível do Enter no fim do texto `s` (o cursor está no fim, como quem acabou de escrever).
fn enter_no_fim(s: &str) -> usize {
    let t = Text::new(s);
    Syntax::new(&t).indent_level_after(&t, t.len())
}

#[test]
fn enter_depois_de_quem_abre_um_bloco_meio_escrito() {
    let casos = [
        ("function f()", 1),
        ("local function f(a, b)", 1),
        ("local g = function()", 1),
        ("if x then", 1),
        ("while x do", 1),
        ("for i = 1, 3 do", 1),
        ("for k, v in pairs(t) do", 1),
        ("do", 1),
        ("repeat", 1),
        ("local t = {", 1),
        ("foo(", 1),
        ("foo(function()", 1),
        ("local x = 1", 0),
        ("print(x)", 0),
        ("", 0),
        ("function f()\n  if x then", 2),
        ("function f()\n  local t = {", 2),
        ("function f()\n  x = 1", 1),
    ];
    for (s, want) in casos {
        assert_eq!(enter_no_fim(s), want, "Enter depois de {s:?}");
    }
}

#[test]
fn enter_dentro_de_um_bloco_completo_mantem() {
    let text = Text::new(BOB);
    let syn = Syntax::new(&text);
    let fim = |needle: &str| text.line_end(line_of(BOB, needle));
    let casos = [
        ("function init(self)", 1),
        ("self.base_x = ph2d.get", 1),
        ("if self.wave == \"square\" then", 2),
        ("s = math.sin(self.t * self.speed) >= 0", 2),
        ("elseif self.wave == \"still\" then", 2),
        ("  else", 2),
        ("if up and not self.was_up", 2),
        ("ph2d.emit(self.top_signal)", 2),
        ("self.was_up = up", 1),
        ("if not self.active then return end", 1),
        ("ph2d.property(\"amplitude\"", 0),
    ];
    for (needle, want) in casos {
        assert_eq!(
            syn.indent_level_after(&text, fim(needle)),
            want,
            "Enter no fim de {needle:?}"
        );
    }
    // Depois do `end` que fecha `init`: de volta ao topo.
    let fim_init = text.line_end(line_of(BOB, "self.was_up = false") + 1);
    assert_eq!(syn.indent_level_after(&text, fim_init), 0);
}

#[test]
fn a_linha_que_comeca_por_end_else_until_chaveta_recua() {
    let s = "function f()\n  if a then\n    b()\n  elseif c then\n    d()\n  else\n    e()\n  end\n  repeat\n    g()\n  until h\n  local t = {\n    1,\n  }\nend\n";
    let t = Text::new(s);
    let syn = Syntax::new(&t);
    let quero = [0, 1, 2, 1, 2, 1, 2, 1, 1, 2, 1, 1, 2, 1, 0];
    let got: Vec<usize> = (0..quero.len())
        .map(|l| syn.line_indent_level(&t, l))
        .collect();
    assert_eq!(got, quero);
    // `dedents_at` é a pergunta do editor ao escrever: o token no início da linha recua?
    for (l, want) in [
        (0, false),
        (3, true),
        (5, true),
        (7, true),
        (10, true),
        (13, true),
        (14, true),
        (2, false),
    ] {
        let at = t.byte_of_line(l) + t.line_string(l).len() - t.line_string(l).trim_start().len();
        assert_eq!(
            syn.dedents_at(&t, at),
            want,
            "linha {l}: {:?}",
            t.line_string(l)
        );
    }
}

/// O caso real do editor: o utilizador acabou de escrever `end` numa linha ainda indentada.
#[test]
fn escrever_end_numa_linha_indentada_pede_o_recuo() {
    let t = Text::new("function f()\n  x()\n  end\n");
    let syn = Syntax::new(&t);
    assert_eq!(syn.line_indent_level(&t, 1), 1);
    assert_eq!(syn.line_indent_level(&t, 2), 0);
    let t = Text::new("if a then\n  b()\n  else");
    assert_eq!(Syntax::new(&t).line_indent_level(&t, 2), 0);
}

#[test]
fn enter_entre_chavetas_e_dentro_de_parenteses() {
    let t = Text::new("local t = {}");
    let syn = Syntax::new(&t);
    assert_eq!(
        syn.indent_level_after(&t, t.len() - 1),
        1,
        "Enter entre {{ e }}"
    );
    let t = Text::new("foo(a,");
    assert_eq!(Syntax::new(&t).indent_level_after(&t, t.len()), 1);
    let t = Text::new("foo(a, b)");
    assert_eq!(
        Syntax::new(&t).indent_level_after(&t, t.len()),
        0,
        "parêntese fechado não indenta"
    );
}
