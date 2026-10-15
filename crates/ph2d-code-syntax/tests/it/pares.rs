//! O par de um parêntese pela árvore: aninhamento, string e par em falta.

use ph2d_code_buffer::Text;
use ph2d_code_syntax::Syntax;

#[test]
fn par_de_parenteses_colchetes_e_chavetas() {
    let s = "local t = { f(a[1], (b)) }";
    let t = Text::new(s);
    let syn = Syntax::new(&t);
    let at = |c: &str, n: usize| {
        s.match_indices(c)
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or(usize::MAX)
    };
    let pares = [
        (at("{", 0), at("}", 0)),
        (at("(", 0), at(")", 1)),
        (at("[", 0), at("]", 0)),
        (at("(", 1), at(")", 0)),
    ];
    for (a, b) in pares {
        assert_eq!(syn.matching_bracket(&t, a), Some(b), "abre em {a}");
        assert_eq!(syn.matching_bracket(&t, b), Some(a), "fecha em {b}");
    }
    assert_eq!(
        syn.matching_bracket(&t, at("f", 0)),
        None,
        "não é parêntese"
    );
    assert_eq!(syn.matching_bracket(&t, s.len()), None);
}

#[test]
fn parentese_dentro_de_string_e_par_em_falta() {
    let s = "f(\"(\")";
    let t = Text::new(s);
    let syn = Syntax::new(&t);
    assert_eq!(
        syn.matching_bracket(&t, 3),
        None,
        "o `(` da string não é parêntese"
    );
    assert_eq!(syn.matching_bracket(&t, 1), Some(5));
    let t = Text::new("print(1");
    assert_eq!(
        Syntax::new(&t).matching_bracket(&t, 5),
        None,
        "sem par escrito, não há par"
    );
}
