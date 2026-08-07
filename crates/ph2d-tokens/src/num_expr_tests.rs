//! Os gates do CONTRATO — o que esta crate promete **sem** conhecer parser nenhum.

use super::{MathHost, deps_of, eval, install_math, math_available, uninstall_math};
use crate::num::NumToken;
use crate::spacing::Spacing;

/// Um host de brinquedo: `"md"` lê um token, qualquer outra coisa é recusada. Ele existe para
/// medir o CONTRATO — a tradução real vive na `ph2d-token-math`, e um gate que a chamasse estaria
/// a medir o parser em vez da fronteira.
fn toy() -> MathHost {
    MathHost {
        deps: |src| {
            if src == "md" {
                Ok(vec![NumToken::Spacing(Spacing::Md)])
            } else {
                Err("nope".to_string())
            }
        },
        eval: |src, value_of| {
            if src == "md" {
                Ok(value_of(NumToken::Spacing(Spacing::Md)) * 2.0)
            } else {
                Err("nope".to_string())
            }
        },
    }
}

#[test]
fn without_a_host_there_is_no_math_and_the_answers_say_so() {
    uninstall_math();
    assert!(!math_available());
    // ⚠️ `None`, nunca `0.0`: zero é um comprimento legítimo, e devolvê-lo tornaria *"não sei"*
    // indistinguível de *"vale zero"* — a rachura do `Bindings` que esta camada não herda.
    assert!(eval("md", &|_| 12.0).is_none());
    assert!(deps_of("md").is_err());
}

#[test]
fn an_installed_host_answers_both_questions() {
    install_math(toy());
    assert!(math_available());
    assert_eq!(deps_of("md").unwrap(), vec![NumToken::Spacing(Spacing::Md)]);
    assert_eq!(eval("md", &|_| 6.0), Some(12.0));
    uninstall_math();
}

/// A recusa do host **chega ao chamador com a frase**, que é o que um toast precisa.
#[test]
fn the_hosts_refusal_carries_its_sentence() {
    install_math(toy());
    assert_eq!(deps_of("something else").unwrap_err(), "nope");
    // Do lado do valor a frase não sobrevive de propósito: quem lê um número não tem onde a pôr, e
    // a porta de escrita — que TEM — pergunta pelo `deps_of`.
    assert!(eval("something else", &|_| 6.0).is_none());
    uninstall_math();
}

/// ⚠️ `value_of` é do CHAMADOR, e é isso que mantém esta crate ignorante da tabela: ela não sabe o
/// que um token vale, só sabe perguntar.
#[test]
fn the_caller_answers_what_each_token_is_worth() {
    install_math(toy());
    assert_eq!(eval("md", &|_| 1.0), Some(2.0));
    assert_eq!(eval("md", &|_| 50.0), Some(100.0));
    uninstall_math();
}

/// Instalar duas vezes é legal e a ÚLTIMA vence — é o que torna um gate capaz de trocar o host sem
/// um `clear` ao lado. O que não é legal é haver dois hosts vivos, e não há: um slot.
#[test]
fn the_last_host_installed_is_the_one_that_answers() {
    install_math(toy());
    install_math(MathHost {
        deps: |_| Ok(Vec::new()),
        eval: |_, _| Ok(99.0),
    });
    assert_eq!(eval("md", &|_| 6.0), Some(99.0));
    uninstall_math();
}
