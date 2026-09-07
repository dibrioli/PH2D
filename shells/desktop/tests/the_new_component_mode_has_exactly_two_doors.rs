//! ⛔⛔ **O MODO NOVO DA SECÇÃO *Component* TEM DUAS PORTAS, E AS DUAS LÊEM A MESMA PERGUNTA**
//! (F4.6c, wave 1).
//!
//! # Porque é textual
//!
//! As duas decisões vivem dentro do laço de quadro da `render_loop`, cuja função tem ~35 argumentos
//! e um `AppGfx` com uma surface de janela real — um teste de integração ali seria uma montagem
//! maior do que a lei que ele mede.
//!
//! # ⚠️ A lei
//!
//! **O que a secção MOSTRA** e **o que o clique FAZ** têm de concordar sobre o motor. Se um lê o
//! interruptor e o outro não, a secção oferece verbos que o dreno recusa (ou o contrário) — e o
//! sintoma é um botão que não faz nada, que é o defeito que esta linha caçou três vezes.
//!
//! ⛔ Ele descasca comentários antes de varrer.

use std::path::Path;

fn code_of(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **As DUAS portas leem o `armed()`** — nem uma a mais, nem uma a menos.
///
/// **Mutação que deve sangrar:** apagar um dos dois `if`.
#[test]
fn both_doors_of_the_new_component_mode_read_the_same_switch() {
    let body = code_of("render_loop/mod.rs");
    assert_eq!(
        body.matches("vec_component_general::armed()").count(),
        2,
        "o interruptor do modo novo deixou de ser lido nos DOIS sitios que decidem — o que a \
         seccao MOSTRA e o que o clique FAZ"
    );
    // A porta que publica o estado, e a que despacha o verbo.
    assert!(
        body.contains("vec_component_general::state_of("),
        "a seccao deixou de poder descrever o modelo geral"
    );
    assert!(
        body.contains("vec_component_general::dispatch("),
        "o clique deixou de poder alcancar o modelo geral"
    );
}

/// ⛔⛔⛔ **As DUAS listas que o modo novo ainda não serve saem VAZIAS por DECLARAÇÃO.**
///
/// O comentário daquele bloco afirmava que elas *«saem vazias sozinhas»* porque lêem o
/// `VecInstance` — e isso **quebra num estado alcançável**: aquele componente é REGISTADO e não
/// está no `DROPPED`, logo a cópia profunda do *Make* leva-o, e uma instância vetorial promovida a
/// prefab geral dá uma cópia com os dois. Aí o painel pintava **peças e variants** cujo clique o
/// `general_verb` recusa devolvendo `None` — **em silêncio**.
///
/// ⚠️ **A régua é o interruptor no caminho das listas**, e não a ausência do sintoma: uma fixtura
/// que não produza a cópia mista lê verde sobre o defeito.
///
/// **Mutação que deve sangrar:** apagar qualquer um dos dois `filter`.
#[test]
fn the_lists_the_new_mode_does_not_serve_are_published_empty() {
    let body = code_of("render_loop/mod.rs");
    assert_eq!(
        body.matches("filter(|_| !general_prefabs)").count(),
        2,
        "uma das listas (pecas / variants) voltou a ser publicada no modo geral — o painel pinta \
         controlos que o dreno recusa em silencio"
    );
}

/// ⛔⛔ **O caminho de OMISSÃO fica intacto** — o motor velho continua a ser chamado no `else`.
///
/// É o que torna esta wave incapaz de regredir: sem a env var, o que corre é exactamente o que
/// corria antes. *Um modo novo que apaga o velho antes de o dono o aprovar não é reversível.*
#[test]
fn the_old_motor_is_still_the_default_path() {
    let body = code_of("render_loop/mod.rs");
    assert!(
        body.contains("vec_component_edit::selected_component("),
        "o publicador do motor VELHO desapareceu — o caminho de omissao deixou de existir"
    );
    assert!(
        body.contains("vec_component_edit::create_main(")
            && body.contains("vec_component_edit::place_instance("),
        "os verbos do motor VELHO desapareceram do dreno — a wave deixou de ser reversivel"
    );
}
