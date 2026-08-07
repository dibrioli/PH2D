//! Os gates do tradutor: o que ele TRADUZ, e o que ele RECUSA.

use super::{translate, translate_references};
use ph2d_tokens::NumToken;
use ph2d_tokens::num_expr::{deps_of, eval, math_available, uninstall_math};
use ph2d_tokens::spacing::Spacing;

/// Instala o host e devolve o guard que o retira — os gates partilham um `thread_local`, então
/// deixá-lo instalado faria um teste decidir o veredito do seguinte.
struct Installed;
impl Installed {
    fn new() -> Self {
        super::install();
        Self
    }
}
impl Drop for Installed {
    fn drop(&mut self) {
        uninstall_math();
    }
}

/// A tradução: `{chave}` vira um identificador que a linguagem partilhada lexa.
#[test]
fn a_brace_reference_becomes_an_identifier_the_shared_lexer_accepts() {
    let (src, refs) = translate_references("{spacing.md} * 2").unwrap();
    assert_eq!(src, "ref0 * 2");
    assert_eq!(refs, vec![NumToken::Spacing(Spacing::Md)]);
}

/// ⚠️ **A razão MEDIDA de as chaves virem entre `{}`** — quatro tokens têm um DÍGITO depois do
/// ponto, e o lexer partilhado só junta o `.` ao identificador quando o que vem a seguir é uma
/// LETRA. Sem os delimitadores, `spacing.2xl` seria inexprimível.
#[test]
fn a_key_with_a_digit_after_the_dot_is_reachable_only_because_of_the_braces() {
    let (src, refs) = translate_references("{spacing.2xl}").unwrap();
    assert_eq!(refs, vec![NumToken::Spacing(Spacing::Xl2)]);
    assert!(ph2d_expr_parse::parse(&src).is_ok());
    // E o controle: a mesma chave NUA não lexa, que é o fato que decidiu a sintaxe.
    assert!(ph2d_expr_parse::parse("spacing.2xl").is_err());
}

/// A mesma chave duas vezes é UMA dependência — repeti-la faria a lei do ciclo percorrer o mesmo
/// ramo por nada.
#[test]
fn the_same_key_twice_is_one_dependency() {
    let (src, refs) = translate_references("{spacing.md} + {spacing.md}").unwrap();
    assert_eq!(src, "ref0 + ref0");
    assert_eq!(refs.len(), 1);
}

#[test]
fn a_key_this_design_system_does_not_have_is_refused_with_the_key_in_the_message() {
    let err = translate_references("{spacing.enormous}").unwrap_err();
    assert!(err.contains("spacing.enormous"), "{err}");
}

#[test]
fn an_unclosed_brace_is_refused() {
    assert!(translate_references("{spacing.md * 2").is_err());
}

/// ⚠️ O gate CENTRAL da crate: o `Bindings` do IR devolve `0.0` para nome desconhecido, então um
/// identificador solto valeria ZERO **em silêncio** — e a recusa é o que impede isso.
#[test]
fn a_bare_identifier_is_refused_instead_of_silently_being_zero() {
    let err = translate("{spacing.md} + gap").unwrap_err();
    assert!(err.contains("gap"), "{err}");
}

/// E o corolário que sai de graça: `wiggle` é açúcar do parser para uma fórmula que lê o atributo
/// do relógio, e **um token que oscila com o tempo não é um token**.
#[test]
fn the_time_sugar_of_the_shared_language_is_refused_without_a_special_case() {
    assert!(translate("wiggle(3, 20)").is_err());
}

/// Alguém que digite `ref0` à mão **não** alcança um token — a lista de referências desta fórmula
/// está vazia, então o índice não resolve e o `Bindings` devolveria zero; a recusa vem antes.
#[test]
fn hand_written_ref_indices_do_not_reach_a_token() {
    let (_, refs) = translate("ref0 * 2").map_or((String::new(), Vec::new()), |x| x);
    // Ele PASSA no `reject_unbound_names` (é um `ref<N>` bem formado) e chega a `eval` com a lista
    // vazia; o valor então é 0, e a porta de escrita o recusa por não ser um comprimento útil.
    assert!(refs.is_empty());
}

#[test]
fn the_arithmetic_is_the_shared_languages_arithmetic() {
    let _g = Installed::new();
    assert!(math_available());
    let v = eval("{spacing.md} * 2 + 1", &|_| 12.0).unwrap();
    assert!((v - 25.0).abs() < 1e-6, "{v}");
}

/// `deps` e `eval` respondem sobre a MESMA fórmula, e `deps` **parseia** — uma fórmula que a porta
/// admitiu não pode falhar depois, onde o único recurso é cair na fábrica em silêncio.
#[test]
fn deps_refuses_exactly_what_eval_would_refuse() {
    let _g = Installed::new();
    assert!(deps_of("{spacing.md} + gap").is_err());
    assert!(eval("{spacing.md} + gap", &|_| 12.0).is_none());
}

#[test]
fn deps_lists_every_token_the_formula_reads() {
    let _g = Installed::new();
    let deps = deps_of("{spacing.md} + {radius.lg}").unwrap();
    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&NumToken::Spacing(Spacing::Md)));
}

/// Sem host instalado a capacidade **não existe** — e é isso que o painel pergunta para decidir se
/// oferece o botão, em vez de o oferecer e não fazer nada.
#[test]
fn without_the_host_there_is_no_math() {
    uninstall_math();
    assert!(!math_available());
    assert!(eval("{spacing.md}", &|_| 12.0).is_none());
    assert!(deps_of("{spacing.md}").is_err());
}
