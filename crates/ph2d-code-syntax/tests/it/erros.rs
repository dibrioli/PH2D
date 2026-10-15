//! Erros de sintaxe na posição do símbolo culpado: o caso real do smoke da Onda 0.

use ph2d_code_buffer::Text;
use ph2d_code_syntax::{Syntax, SyntaxError};

/// O ficheiro do smoke, lido onde ele vive (se mudar de sítio, isto não compila — não fica calado).
const QUEBRADO: &str =
    include_str!("../../../../docs/CodeEditor/ferramentas/smoke/script_quebrado.luau");

#[test]
fn script_quebrado_aponta_a_segunda_igual_da_linha_3() {
    let t = Text::new(QUEBRADO);
    let errs = Syntax::new(&t).errors();
    assert_eq!(errs.len(), 1, "{errs:?}");
    let e = &errs[0];
    let p = t.point_of_byte(e.range.start);
    // O Luau diz «line 3 · Expected identifier when parsing expression, got '='» — a 2.ª `=`
    // de `local x = = 1`. O tree-sitter aponta o MESMO símbolo: linha 3 (base 1), colunas 10..11.
    assert_eq!((p.row + 1, p.col, e.range.len()), (3, 10, 1));
    assert_eq!(t.slice(e.range.clone()), "=");
    assert_eq!(t.line_string(p.row), "local x = = 1");
    assert_eq!(e.missing, None);
}

#[test]
fn texto_valido_nao_tem_erros_e_o_que_falta_e_nomeado() {
    let t = Text::new("function f()\n  return 1\nend\n");
    assert_eq!(Syntax::new(&t).errors(), Vec::<SyntaxError>::new());
    let t = Text::new("local x = f(1, 2\nlocal y = 3\n");
    let errs = Syntax::new(&t).errors();
    assert!(!errs.is_empty());
    for e in &errs {
        assert!(e.range.end <= t.len());
    }
}
