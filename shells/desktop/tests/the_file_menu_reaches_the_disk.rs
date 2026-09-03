//! **A outra metade do menu Ficheiro** — o painel levanta a bandeira, e ELE tem de a drenar.
//!
//! ⚠️ Os gates do lado do painel vivem em
//! `crates/ph2d-editor-core/tests/the_file_menu_items_are_not_mute.rs`, e afirmam o que o clique
//! PROMETE. Este afirma que a promessa chega ao disco — e ele é um arch-gate de FONTE porque a
//! travessia real não é alcançável: o `drain_project_io` sai no `gfx.is_none()` de um `App` sem
//! janela, e o seletor nativo abre uma janela modal do sistema.
//!
//! ⛔ Sem ele, os três itens voltariam a ser mudos com um `git revert` de uma linha, e nada
//! reprovaria — foi exactamente esse o estado até 2026-08-23.

mod sculpt_source;
use sculpt_source::{function_body, source};

/// **A agulha casa como PALAVRA, nunca como SUBCADEIA.**
///
/// ⛔ **Sem isto o gate abaixo era verde pela razão errada, e a razão é uma letra:** `asked.save`
/// é prefixo de `asked.save_as`, e as três bandeiras vivem na MESMA linha
/// (`let (save, save_as, open) = (asked.save, asked.save_as, asked.open);`). Um `contains` puro
/// perguntava *«a linha menciona `asked.save`?»* e a resposta era sim mesmo depois de o `Save`
/// morrer — mutar o produto para `(false, asked.save_as, asked.open)` deixava o item *Save* do
/// menu **e** o `Ctrl+S` mudos com o gate VERDE. *Um botão que engole o clique é pior que um
/// ausente: o artista conclui que gravou.*
///
/// A fronteira é a de identificador (`[A-Za-z0-9_]`) dos DOIS lados: à direita ela separa
/// `asked.save` de `asked.save_as`, à esquerda ela impede que uma agulha curta (`save`) case
/// dentro de um nome maior (`autosave`). Um lado onde a própria agulha já traz pontuação
/// (`vec_text_ride::link(`) fica satisfeito por construção.
fn mentions(hay: &str, needle: &str) -> bool {
    fn is_word(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
    let starts_word = needle.chars().next().is_some_and(is_word);
    let ends_word = needle.chars().last().is_some_and(is_word);
    hay.match_indices(needle).any(|(at, _)| {
        let left = !starts_word || hay[..at].chars().next_back().is_none_or(|c| !is_word(c));
        let right = !ends_word
            || hay[at + needle.len()..]
                .chars()
                .next()
                .is_none_or(|c| !is_word(c));
        left && right
    })
}

/// **As bandeiras são DRENADAS** — e cada uma leva ao seu gesto.
///
/// ⚠️ A asserção é sobre todas juntas: drenar três e esquecer a quarta deixa uma bandeira **presa a
/// `true`**, e a partir daí o app grava (ou abre, ou exporta) uma vez por quadro.
///
/// ⛔ **A lista é LITERAL, e a dívida está nomeada** no irmão
/// `the_file_menu_items_are_not_mute.rs`: ela tinha três entradas e o *Export SVG…* (2026-09-02)
/// passou sem a acordar. Item novo no menu Ficheiro ⇒ entrada nova aqui.
///
/// ⚠️ **Toda agulha passa pela [`mentions`], não pelo `contains`** — o doc dela tem o mecanismo e
/// a mutação que o provou. Não é só a `asked.save`: um `project_save_gesture` que ganhasse um
/// irmão `project_save_gesture_as` teria a mesma doença, e a lei fica escrita uma vez.
#[test]
fn every_file_menu_flag_is_drained() {
    let body = function_body(&source("project_io.rs"), "drain_project_io");
    for flag in [
        "asked.save",
        "asked.save_as",
        "asked.open",
        "asked.export_svg",
    ] {
        assert!(
            mentions(&body, flag),
            "o `drain_project_io` nao le' `{flag}` — a bandeira fica presa e o gesto repete uma vez \
             por quadro"
        );
    }
    assert!(
        mentions(&body, "mem::take"),
        "as bandeiras tem de ser CONSUMIDAS (mem::take), nao so' lidas"
    );
    assert!(
        mentions(&body, "project_save_gesture")
            && mentions(&body, "project_open_gesture")
            && mentions(&body, "export_svg_gesture"),
        "…e cada uma tem de chamar o gesto, nao so' baixar a bandeira"
    );
}

/// **O controle da própria agulha** — sem ele a [`mentions`] podia ser um `|_, _| true`.
///
/// ⚠️ Ele afirma os DOIS sentidos: o que ela tem de aceitar (a bandeira que existe) e o que ela
/// tem de recusar (o prefixo que mora dentro da irmã). O caso do meio é a mutação real: um corpo
/// que só fala de `asked.save_as` **não** menciona `asked.save`.
#[test]
fn the_needle_does_not_match_inside_a_longer_name() {
    let alive = "let (save, save_as, open) = (asked.save, asked.save_as, asked.open);";
    assert!(mentions(alive, "asked.save"));
    assert!(mentions(alive, "asked.save_as"));

    let dead = "let (save, save_as, open) = (false, asked.save_as, asked.open);";
    assert!(
        !mentions(dead, "asked.save"),
        "o `Save` morto tem de ler-se como AUSENTE — e' esta a mutacao que o `contains` deixava \
         passar"
    );
    assert!(mentions(dead, "asked.save_as"));

    // …e a fronteira da ESQUERDA, que é o mesmo defeito espelhado.
    assert!(!mentions("self.autosave_tick();", "save"));
}

/// **O TECLADO e o MENU chamam a mesma função.**
///
/// ⚠️ Duas portas para o mesmo gesto é como o `.ase` ficou invisível no diálogo de import no mesmo
/// dia (`crate::import_router`), e aqui a divergência seria pior: um `Save` do menu a gravar
/// noutro sítio que o `Ctrl+S`. O gate lê o handler de teclado e exige que ele passe pelas mesmas
/// duas portas — não pelo `project_save`/`project_load` crus, que não perguntam.
#[test]
fn the_keyboard_goes_through_the_same_door_as_the_menu() {
    let src = source("input_dispatch/keyboard_files.rs");
    let body = function_body(&src, "file_chords");
    assert!(
        mentions(&body, "project_save_gesture"),
        "o Ctrl+S tem de passar pela porta que decide ONDE gravar"
    );
    assert!(
        mentions(&body, "project_open_gesture"),
        "e o Ctrl+O pela que PERGUNTA — abrir deita fora o trabalho nao gravado"
    );
    assert!(
        mentions(&body, "shift_key"),
        "o Ctrl+Shift+S (Save As) tem de perguntar o modificador, senao os dois gestos ficam \
         indistinguiveis"
    );
}
